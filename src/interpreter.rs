use crate::ast::*;
use anyhow::{Result, anyhow, bail};
use rug::{Integer, Rational};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Number(Rational),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Function {
        params: Vec<String>,
        body: Expr,
        env: Env,
    },
    Native {
        name: &'static str,
        arity: usize,
        applied: Vec<Value>,
        f: NativeFn,
    },
    Module {
        name: String,
        members: HashMap<String, Value>,
    },
}

pub type NativeFunction = Rc<dyn Fn(&Env, &[Value]) -> Result<Value>>;
#[derive(Clone)]
pub struct NativeFn(pub NativeFunction);

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<native fn>")
    }
}

pub struct Ctx {
    pub out: RefCell<Box<dyn Write>>,
    pub current_file: RefCell<Option<PathBuf>>,
    pub current_exports: RefCell<Option<HashMap<String, Value>>>,
    pub is_interactive: bool,
}

impl Ctx {
    pub fn stdio() -> Rc<Self> {
        Rc::new(Ctx {
            out: RefCell::new(Box::new(std::io::stdout())),
            current_file: RefCell::new(None),
            current_exports: RefCell::new(None),
            is_interactive: true,
        })
    }

    pub fn stdio_with_file(path: PathBuf) -> Rc<Self> {
        Rc::new(Ctx {
            out: RefCell::new(Box::new(std::io::stdout())),
            current_file: RefCell::new(Some(path)),
            current_exports: RefCell::new(None),
            is_interactive: false,
        })
    }
}

impl std::fmt::Debug for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ctx")
    }
}

pub type Env = Rc<RefCell<Scope>>;

#[derive(Debug)]
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
    ctx: Rc<Ctx>,
}

impl Scope {
    pub fn new() -> Env {
        Self::with_ctx(Ctx::stdio())
    }

    pub fn with_ctx(ctx: Rc<Ctx>) -> Env {
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: None,
            ctx,
        }))
    }

    pub fn child(parent: Env) -> Env {
        let ctx = parent.borrow().ctx.clone();
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: Some(parent),
            ctx,
        }))
    }
}

pub fn ctx_of(env: &Env) -> Rc<Ctx> {
    env.borrow().ctx.clone()
}

pub fn local_vars(env: &Env) -> HashMap<String, Value> {
    env.borrow().vars.clone()
}

pub fn lookup(env: &Env, name: &str) -> Option<Value> {
    if let Some(v) = env.borrow().vars.get(name).cloned() {
        return Some(v);
    }
    let parent = env.borrow().parent.clone();
    parent.and_then(|p| lookup(&p, name))
}

pub fn define(env: &Env, name: &str, val: Value) {
    env.borrow_mut().vars.insert(name.to_string(), val);
}

pub fn run(program: &Program) -> Result<Value> {
    let env = Scope::new();
    crate::prelude::install(&env);
    eval_program(program, &env)
}

/// Apply a value (function or native) to a list of already-evaluated args.
/// Calls are unary after the parser's curry desugar, so `arg_vals` is either
/// empty (zero-arg call: `f()`) or a single value.
pub fn apply(env: &Env, callee: Value, arg_vals: Vec<Value>) -> Result<Value> {
    match callee {
        Value::Function {
            params,
            body,
            env: fn_env,
        } => {
            if params.len() != arg_vals.len() {
                bail!(
                    "function expects {} arg(s), got {}",
                    params.len(),
                    arg_vals.len()
                );
            }
            let scope = Scope::child(fn_env);
            for (p, a) in params.iter().zip(arg_vals) {
                define(&scope, p, a);
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
                bail!(
                    "native `{}` expects {} arg(s), got {}",
                    name,
                    arity,
                    applied.len()
                );
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
                bail!("native `{}` over-applied (arity {})", name, arity)
            }
        }
        v => bail!("cannot call non-function: {}", type_name(&v)),
    }
}

pub fn eval_program(program: &Program, env: &Env) -> Result<Value> {
    let mut last = Value::Unit;
    for stmt in &program.statements {
        last = eval_statement(stmt, env)?;
    }
    Ok(last)
}

fn eval_statement(stmt: &Statement, env: &Env) -> Result<Value> {
    match stmt {
        Statement::Assignment(a) => {
            let val = eval_expr(&a.value, env)?;
            let bindings = match_pattern(&a.pattern, &val, env)?
                .ok_or_else(|| anyhow!("pattern match failed in assignment"))?;
            for (k, v) in bindings {
                define(env, &k, v);
            }
            Ok(Value::Unit)
        }
        Statement::Expr(e) => eval_expr(e, env),
    }
}

fn eval_expr(expr: &Expr, env: &Env) -> Result<Value> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(n.clone())),
        Expr::String(s) => Ok(Value::String(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => {
            lookup(env, name).ok_or_else(|| anyhow!("undefined variable: {}", name))
        }
        Expr::List(items) => Ok(Value::List(
            items
                .iter()
                .map(|e| eval_expr(e, env))
                .collect::<Result<_>>()?,
        )),
        Expr::Tuple(items) => Ok(Value::Tuple(
            items
                .iter()
                .map(|e| eval_expr(e, env))
                .collect::<Result<_>>()?,
        )),
        Expr::Dict(entries) => {
            let mut out: Vec<(Value, Value)> = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                let kv = eval_expr(k, env)?;
                let vv = eval_expr(v, env)?;
                if let Some(slot) = out.iter_mut().find(|(ek, _)| value_eq(ek, &kv)) {
                    slot.1 = vv;
                } else {
                    out.push((kv, vv));
                }
            }
            Ok(Value::Dict(out))
        }
        Expr::Set(items) => {
            let mut out = Vec::with_capacity(items.len());
            for e in items {
                let v = eval_expr(e, env)?;
                if !out.iter().any(|x| value_eq(x, &v)) {
                    out.push(v);
                }
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
            v => bail!("if condition must be bool, got {}", type_name(&v)),
        },
        Expr::Match { scrutinee, arms } => {
            let val = eval_expr(scrutinee, env)?;
            for arm in arms {
                if let Some(bindings) = match_pattern(&arm.pattern, &val, env)? {
                    let scope = Scope::child(env.clone());
                    for (k, v) in bindings {
                        define(&scope, &k, v);
                    }
                    return eval_expr(&arm.body, &scope);
                }
            }
            bail!("no match arm matched")
        }
        Expr::Call { callee, args } => {
            let callee_val = eval_expr(callee, env)?;
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|a| eval_expr(a, env))
                .collect::<Result<_>>()?;
            apply(env, callee_val, arg_vals)
        }
        Expr::Access { target, key } => {
            let t = eval_expr(target, env)?;
            match (&t, key) {
                (Value::List(xs), AccessKey::Index(i))
                | (Value::Tuple(xs), AccessKey::Index(i)) => xs
                    .get(*i)
                    .cloned()
                    .ok_or_else(|| anyhow!("index {} out of range (len {})", i, xs.len())),
                (Value::Dict(es), AccessKey::Field(name)) => {
                    let k = Value::String(name.clone());
                    es.iter()
                        .find(|(ek, _)| value_eq(ek, &k))
                        .map(|(_, v)| v.clone())
                        .ok_or_else(|| anyhow!("dict has no key {:?}", name))
                }
                (Value::Module { members, .. }, AccessKey::Field(name)) => members
                    .get(name)
                    .cloned()
                    .ok_or_else(|| anyhow!("module has no member `.{}`", name)),
                (v, AccessKey::Index(_)) => {
                    bail!("cannot index into {}", type_name(v))
                }
                (v, AccessKey::Field(name)) => {
                    bail!("cannot read field .{} from {}", name, type_name(v))
                }
            }
        }
        Expr::Unary { op, operand } => {
            let v = eval_expr(operand, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (op, v) => bail!("cannot apply {:?} to {}", op, type_name(&v)),
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

fn eval_binary(op: BinaryOp, lhs: &Expr, rhs: &Expr, env: &Env) -> Result<Value> {
    if matches!(op, BinaryOp::And | BinaryOp::Or) {
        let l = eval_expr(lhs, env)?;
        let Value::Bool(lb) = l else {
            bail!("{:?} expects bool, got {}", op, type_name(&l));
        };
        match (op, lb) {
            (BinaryOp::And, false) => return Ok(Value::Bool(false)),
            (BinaryOp::Or, true) => return Ok(Value::Bool(true)),
            _ => {}
        }
        let r = eval_expr(rhs, env)?;
        let Value::Bool(rb) = r else {
            bail!("{:?} expects bool, got {}", op, type_name(&r));
        };
        return Ok(Value::Bool(rb));
    }

    let l = eval_expr(lhs, env)?;
    let r = eval_expr(rhs, env)?;
    match (op, &l, &r) {
        (BinaryOp::Add, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Rational::from(a + b)))
        }
        (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Rational::from(a - b)))
        }
        (BinaryOp::Mul, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Rational::from(a * b)))
        }
        (BinaryOp::Div, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                bail!("division by zero");
            }
            Ok(Value::Number(Rational::from(a / b)))
        }
        (BinaryOp::Mod, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                bail!("modulo by zero");
            }
            Ok(Value::Number(rat_mod(a, b)))
        }
        (BinaryOp::Pow, Value::Number(a), Value::Number(b)) => Ok(Value::Number(rat_pow(a, b)?)),
        (BinaryOp::Add, Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b)))
        }
        (BinaryOp::Eq, a, b) => Ok(Value::Bool(value_eq(a, b))),
        (BinaryOp::Ne, a, b) => Ok(Value::Bool(!value_eq(a, b))),
        (BinaryOp::Lt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
        (BinaryOp::Le, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
        (BinaryOp::Gt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
        (BinaryOp::Ge, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
        (BinaryOp::Lt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        (BinaryOp::Le, Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
        (BinaryOp::Gt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
        (BinaryOp::Ge, Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
        (BinaryOp::Match, Value::String(a), Value::String(b)) => {
            Ok(Value::Bool(a.contains(b.as_str())))
        }
        (BinaryOp::NotMatch, Value::String(a), Value::String(b)) => {
            Ok(Value::Bool(!a.contains(b.as_str())))
        }
        (op, a, b) => bail!(
            "cannot apply {:?} to {} and {}",
            op,
            type_name(a),
            type_name(b)
        ),
    }
}

fn match_pattern(pat: &Pattern, val: &Value, env: &Env) -> Result<Option<HashMap<String, Value>>> {
    let mut bindings = HashMap::new();
    if match_into(pat, val, env, &mut bindings)? {
        Ok(Some(bindings))
    } else {
        Ok(None)
    }
}

fn match_into(
    pat: &Pattern,
    val: &Value,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool> {
    match pat {
        Pattern::Wildcard => Ok(true),
        Pattern::Ident(name) => {
            bindings.insert(name.clone(), val.clone());
            Ok(true)
        }
        Pattern::Number(n) => Ok(matches!(val, Value::Number(m) if m == n)),
        Pattern::String(s) => Ok(matches!(val, Value::String(t) if t == s)),
        Pattern::Bool(p) => Ok(matches!(val, Value::Bool(b) if b == p)),
        Pattern::List(items) => match_seq(items, val, env, bindings, true),
        Pattern::Tuple(items) => match_seq(items, val, env, bindings, false),
        Pattern::Dict(entries) => {
            let Value::Dict(d) = val else {
                return Ok(false);
            };
            for (key_expr, sub_pat) in entries {
                let key = eval_expr(key_expr, env)?;
                let Some((_, found)) = d.iter().find(|(k, _)| value_eq(k, &key)) else {
                    return Ok(false);
                };
                if !match_into(sub_pat, found, env, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Pattern::Set(items) => {
            let Value::Set(s) = val else {
                return Ok(false);
            };
            for p in items {
                let mut tmp = HashMap::new();
                let any = s
                    .iter()
                    .any(|el| match_into(p, el, env, &mut tmp).unwrap_or(false));
                if !any {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

fn match_seq(
    items: &[PatternItem],
    val: &Value,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
    list_like: bool,
) -> Result<bool> {
    let elems = match (list_like, val) {
        (true, Value::List(xs)) => xs,
        (false, Value::Tuple(xs)) => xs,
        _ => return Ok(false),
    };
    let rest_idx = items.iter().position(|i| matches!(i, PatternItem::Rest(_)));
    match rest_idx {
        None => {
            if elems.len() != items.len() {
                return Ok(false);
            }
            for (p, v) in items.iter().zip(elems.iter()) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Some(idx) => {
            if items[idx + 1..]
                .iter()
                .any(|i| matches!(i, PatternItem::Rest(_)))
            {
                bail!("multiple `..` patterns in one sequence");
            }
            let before = &items[..idx];
            let after = &items[idx + 1..];
            if elems.len() < before.len() + after.len() {
                return Ok(false);
            }
            for (p, v) in before.iter().zip(elems.iter()) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            let after_start = elems.len() - after.len();
            for (p, v) in after.iter().zip(elems[after_start..].iter()) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            if let PatternItem::Rest(Some(name)) = &items[idx] {
                let middle = elems[before.len()..after_start].to_vec();
                let bound = if list_like {
                    Value::List(middle)
                } else {
                    Value::Tuple(middle)
                };
                bindings.insert(name.clone(), bound);
            }
            Ok(true)
        }
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Unit, Value::Unit) => true,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::List(x), Value::List(y)) | (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| value_eq(a, b))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.iter().any(|(k2, v2)| value_eq(k, k2) && value_eq(v, v2)))
        }
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|a| y.iter().any(|b| value_eq(a, b)))
        }
        _ => false,
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => Ok(()),
            Value::Number(n) => write!(f, "{}", format_rational(n)),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "]")
            }
            Value::Tuple(xs) => {
                write!(f, "(")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                if xs.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Dict(es) => {
                write!(f, "{{")?;
                for (i, (k, v)) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(xs) => {
                write!(f, "{{")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "}}")
            }
            Value::Function { params, .. } => write!(f, "<fn ({})>", params.join(", ")),
            Value::Native { name, .. } => write!(f, "<native {}>", name),
            Value::Module { name, .. } => write!(f, "<module {}>", name),
        }
    }
}

// Truncated-toward-zero modulo on rationals: a - b * trunc(a / b).
fn rat_mod(a: &Rational, b: &Rational) -> Rational {
    let q = Rational::from(a / b);
    let (num, den) = q.into_numer_denom();
    let trunc = Integer::from(&num / &den);
    a - Rational::from(b * trunc)
}

// `**` requires an integer exponent; non-integer exponents would produce
// irrationals that don't fit in Rational.
fn rat_pow(base: &Rational, exp: &Rational) -> Result<Rational> {
    if exp.denom() != &Integer::from(1) {
        bail!(
            "** requires an integer exponent, got {}",
            format_rational(exp)
        );
    }
    let e_int = exp.numer();
    let e: i32 = e_int
        .to_i32()
        .ok_or_else(|| anyhow!("** exponent out of range: {}", e_int))?;
    if e == 0 {
        return Ok(Rational::from(1));
    }
    if base.cmp0() == std::cmp::Ordering::Equal && e < 0 {
        bail!("0 cannot be raised to a negative power");
    }
    use rug::ops::Pow;
    Ok(base.clone().pow(e))
}

// Display a rational as the most natural decimal form:
// - integer when denom == 1
// - terminating decimal (e.g. 5/2 -> "2.5") when denom is 2^a * 5^b
// - otherwise the canonical "n/d" form
fn format_rational(r: &Rational) -> String {
    let num = r.numer();
    let den = r.denom();
    if den == &Integer::from(1) {
        return num.to_string();
    }

    let mut d = den.clone();
    let mut twos: u32 = 0;
    while d.is_divisible_u(2) {
        d /= 2u32;
        twos += 1;
    }
    let mut fives: u32 = 0;
    while d.is_divisible_u(5) {
        d /= 5u32;
        fives += 1;
    }
    if d != 1 {
        return format!("{}/{}", num, den);
    }

    let power = twos.max(fives);
    let mut scaled = num.clone().abs();
    let mut extra_twos = power - twos;
    while extra_twos > 0 {
        scaled *= 2u32;
        extra_twos -= 1;
    }
    let mut extra_fives = power - fives;
    while extra_fives > 0 {
        scaled *= 5u32;
        extra_fives -= 1;
    }

    let sign = if num.cmp0() == std::cmp::Ordering::Less {
        "-"
    } else {
        ""
    };
    let digits = scaled.to_string();
    let p = power as usize;
    let padded = if digits.len() <= p {
        let pad = "0".repeat(p - digits.len() + 1);
        format!("{}{}", pad, digits)
    } else {
        digits
    };
    let split = padded.len() - p;
    format!("{}{}.{}", sign, &padded[..split], &padded[split..])
}

pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Unit => "unit",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::List(_) => "list",
        Value::Tuple(_) => "tuple",
        Value::Dict(_) => "dict",
        Value::Set(_) => "set",
        Value::Function { .. } => "function",
        Value::Native { .. } => "native",
        Value::Module { .. } => "module",
    }
}
