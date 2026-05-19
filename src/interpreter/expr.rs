use im::{HashMap, HashSet};
use rug::Rational;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{AccessKey, ExportKind, Expr, Program, Statement, UnaryOp};

use super::binop::eval_binary;
use super::error::{RuntimeError, RuntimeResult};
use super::pattern::match_pattern;
use super::scope::{Env, Scope, ctx_of, define, lookup};
use super::value::{LazyState, Value, type_name};

/// Force a lazy thunk to a concrete value. Memoizes via the shared RefCell so
/// re-forcing is cheap. Used by pattern matching, equality, display, and the
/// `force_cons` helper that walks a stream's spine.
pub fn force_tail(tail: &Rc<RefCell<LazyState>>) -> RuntimeResult<Value> {
    if let LazyState::Forced(v) = &*tail.borrow() {
        return Ok(v.clone());
    }
    let pending = std::mem::replace(&mut *tail.borrow_mut(), LazyState::Forced(Value::Unit));
    let v = match pending {
        LazyState::Pending { body, env } => eval_expr(&body, &env)?,
        LazyState::Native(thunk) => thunk()?,
        LazyState::Forced(v) => {
            *tail.borrow_mut() = LazyState::Forced(v.clone());
            return Ok(v);
        }
    };
    *tail.borrow_mut() = LazyState::Forced(v.clone());
    Ok(v)
}

fn upsert_export(table: &mut Vec<(String, Value)>, name: String, value: Value) {
    if let Some(slot) = table.iter_mut().find(|(k, _)| *k == name) {
        slot.1 = value;
    } else {
        table.push((name, value));
    }
}

pub fn run(program: &Program) -> RuntimeResult<Value> {
    let env = Scope::new();
    crate::prelude::install(&env);
    eval_program(program, &env)
}

/// Apply a value (function or native) to a list of already-evaluated args.
/// Calls are unary after the parser's curry desugar, so `arg_vals` is either
/// empty (zero-arg call: `f()`) or a single value.
pub fn apply(env: &Env, callee: Value, arg_vals: Vec<Value>) -> RuntimeResult<Value> {
    match callee {
        Value::Function {
            params,
            body,
            env: fn_env,
        } => {
            if params.len() != arg_vals.len() {
                return Err(RuntimeError::ArityMismatch {
                    expected: params.len(),
                    got: arg_vals.len(),
                });
            }
            let scope = Scope::child(fn_env);
            for (p, a) in params.iter().zip(arg_vals) {
                let bindings = match_pattern(p, &a, &scope)?
                    .ok_or(RuntimeError::AssignmentPatternFailed)?;
                for (k, v) in bindings {
                    define(&scope, &k, v);
                }
            }
            eval_expr(&body, &scope)
        }
        Value::Native {
            name,
            arity,
            mut applied,
            f,
        } => {
            if arg_vals.is_empty() {
                if arity == 0 && applied.is_empty() {
                    return (f.0)(env, &[]);
                }
                return Err(RuntimeError::NativeArity {
                    name,
                    expected: arity,
                    got: applied.len(),
                });
            }
            applied.extend(arg_vals);
            if applied.len() == arity {
                (f.0)(env, &applied)
            } else if applied.len() < arity {
                Ok(Value::Native {
                    name,
                    arity,
                    applied,
                    f,
                })
            } else {
                Err(RuntimeError::NativeOverApplied { name, arity })
            }
        }
        v => Err(RuntimeError::NotCallable(type_name(&v))),
    }
}

pub fn eval_program(program: &Program, env: &Env) -> RuntimeResult<Value> {
    let mut last = Value::Unit;
    for stmt in &program.statements {
        last = eval_statement(stmt, env)?;
    }
    Ok(last)
}

fn eval_statement(stmt: &Statement, env: &Env) -> RuntimeResult<Value> {
    match stmt {
        Statement::Assignment(a) => {
            let val = eval_expr(&a.value, env)?;
            let bindings = match_pattern(&a.pattern, &val, env)?
                .ok_or(RuntimeError::AssignmentPatternFailed)?;
            for (k, v) in bindings {
                define(env, &k, v);
            }
            Ok(Value::Unit)
        }
        Statement::Export(kind) => {
            let ctx = ctx_of(env);
            let mut exports_slot = ctx.current_exports.borrow_mut();
            // Top-level scripts have no exports table; `export` is a no-op
            // there so a module file can still be run directly.
            let Some(table) = exports_slot.as_mut() else {
                return Ok(Value::Unit);
            };
            match kind {
                ExportKind::All => {
                    for (k, v) in env.borrow().vars.iter() {
                        upsert_export(table, k.clone(), v.clone());
                    }
                }
                ExportKind::Names(names) => {
                    for name in names {
                        let v = lookup(env, name)
                            .ok_or_else(|| RuntimeError::ExportUndefined(name.clone()))?;
                        upsert_export(table, name.clone(), v);
                    }
                }
            }
            Ok(Value::Unit)
        }
        // Bare `import "x"` statement splats the module's atom-keyed members
        // into the current scope. An `import` used as part of a larger
        // expression (`x = import "x"`, `foo(import "x")`) just produces the
        // Object value.
        Statement::Expr(Expr::Import(path)) => {
            let val = eval_expr(&Expr::Import(path.clone()), env)?;
            if let Value::Object(entries) = val {
                for (k, v) in entries.iter() {
                    if let Value::Atom(name) = k {
                        define(env, name, v.clone());
                    }
                }
            }
            Ok(Value::Unit)
        }
        Statement::Expr(e) => eval_expr(e, env),
    }
}

pub fn eval_expr(expr: &Expr, env: &Env) -> RuntimeResult<Value> {
    match expr {
        Expr::Unit => Ok(Value::Unit),
        Expr::Number(n) => Ok(Value::Number(Rc::new(n.clone()))),
        Expr::String(s) => Ok(Value::String(s.as_str().into())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Atom(name) => Ok(Value::Atom(name.as_str().into())),
        Expr::Ident(name) => {
            lookup(env, name).ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))
        }
        Expr::List(items) => Ok(Value::List(
            items
                .iter()
                .map(|e| eval_expr(e, env))
                .collect::<RuntimeResult<_>>()?,
        )),
        Expr::Object(entries) => {
            let mut out: HashMap<Value, Value> = HashMap::new();
            for (k, v) in entries {
                let kv = eval_expr(k, env)?;
                let vv = eval_expr(v, env)?;
                out.insert(kv, vv);
            }
            Ok(Value::Object(out))
        }
        Expr::Set(items) => {
            let mut out: HashSet<Value> = HashSet::new();
            for e in items {
                let v = eval_expr(e, env)?;
                out.insert(v);
            }
            Ok(Value::Set(out))
        }
        Expr::Function { params, body } => Ok(Value::Function {
            params: params.clone(),
            body: (**body).clone(),
            env: env.clone(),
        }),
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_expr(cond, env)? {
            Value::Bool(true) => eval_expr(then_branch, env),
            Value::Bool(false) => eval_expr(else_branch, env),
            v => Err(RuntimeError::IfConditionNotBool(type_name(&v))),
        },
        Expr::Match { scrutinee, arms } => {
            let val = eval_expr(scrutinee, env)?;
            for arm in arms {
                if let Some(bindings) = match_pattern(&arm.pattern, &val, env)? {
                    let scope = Scope::child(env.clone());
                    for (k, v) in bindings {
                        define(&scope, &k, v);
                    }
                    if let Some(guard) = &arm.guard {
                        match eval_expr(guard, &scope)? {
                            Value::Bool(true) => {}
                            Value::Bool(false) => continue,
                            v => return Err(RuntimeError::MatchGuardNotBool(type_name(&v))),
                        }
                    }
                    return eval_expr(&arm.body, &scope);
                }
            }
            Err(RuntimeError::NoMatchArm)
        }
        Expr::Call { callee, args } => {
            let callee_val = eval_expr(callee, env)?;
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|a| eval_expr(a, env))
                .collect::<RuntimeResult<_>>()?;
            apply(env, callee_val, arg_vals)
        }
        Expr::Range {
            start,
            end,
            inclusive,
        } => {
            let start_v = eval_expr(start, env)?;
            let Value::Number(s) = start_v else {
                return Err(RuntimeError::RangeStartNotNumber(type_name(&start_v)));
            };
            let end_v = match end {
                None => None,
                Some(e) => {
                    let v = eval_expr(e, env)?;
                    let Value::Number(n) = v else {
                        return Err(RuntimeError::RangeEndNotNumber(type_name(&v)));
                    };
                    Some(n)
                }
            };
            Ok(Value::Range {
                start: s,
                end: end_v,
                inclusive: *inclusive,
            })
        }
        Expr::Import(path) => {
            let v = eval_expr(path, env)?;
            let Value::String(s) = v else {
                return Err(RuntimeError::ImportPathNotString(type_name(&v)));
            };
            crate::prelude::import_module(env, &s)
        }
        Expr::Access { target, key } => {
            let t = eval_expr(target, env)?;
            match (&t, key) {
                (Value::List(xs), AccessKey::Index(i)) => {
                    xs.get(*i).cloned().ok_or(RuntimeError::IndexOutOfRange {
                        index: *i,
                        len: xs.len(),
                    })
                }
                (Value::Object(es), AccessKey::Name(name)) => {
                    let k = Value::Atom(name.as_str().into());
                    es.get(&k)
                        .cloned()
                        .ok_or_else(|| RuntimeError::ObjectMissingKey(name.clone()))
                }
                (v, AccessKey::Index(_)) => Err(RuntimeError::CannotIndex(type_name(v))),
                (v, AccessKey::Name(name)) => Err(RuntimeError::CannotReadAtomField {
                    atom: name.clone(),
                    type_name: type_name(v),
                }),
            }
        }
        Expr::Unary { op, operand } => {
            let v = eval_expr(operand, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Number(n)) => {
                    Ok(Value::Number(Rc::new(Rational::from(-n.as_ref()))))
                }
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (op, v) => Err(RuntimeError::UnaryTypeError {
                    op: *op,
                    type_name: type_name(&v),
                }),
            }
        }
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env),
        Expr::Scope(stmts) => {
            let scope = Scope::child(env.clone());
            let mut last = Value::Unit;
            for stmt in stmts {
                last = eval_statement(stmt, &scope)?;
            }
            Ok(last)
        }
    }
}
