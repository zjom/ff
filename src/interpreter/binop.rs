use rug::Rational;
use std::sync::{Arc, Mutex};

use crate::ast::{BinaryOp, Expr};

use super::error::{RuntimeError, RuntimeResult};
use super::expr::eval_expr;
use super::number::{rat_mod, rat_pow};
use super::scope::Env;
use super::value::{LazyState, Value, type_name};

pub(super) fn eval_binary(
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &Env,
) -> RuntimeResult<Value> {
    // `a :: b` is non-strict in `b`: evaluate the head eagerly and capture the
    // rhs as a thunk so recursive stdlib builders (e.g. `f(x) :: map(f, rest)`)
    // don't bottom out before pattern-matching the tail.
    if matches!(op, BinaryOp::Cons) {
        let head = eval_expr(lhs, env)?;
        let tail = Arc::new(Mutex::new(LazyState::Pending {
            body: rhs.clone(),
            env: env.clone(),
        }));
        return Ok(Value::Cons {
            head: Arc::new(head),
            tail,
        });
    }

    if matches!(op, BinaryOp::And | BinaryOp::Or) {
        let l = eval_expr(lhs, env)?;
        let Value::Bool(lb) = l else {
            return Err(RuntimeError::LogicalOperandNotBool {
                op,
                type_name: type_name(&l),
            });
        };
        match (op, lb) {
            (BinaryOp::And, false) => return Ok(Value::Bool(false)),
            (BinaryOp::Or, true) => return Ok(Value::Bool(true)),
            _ => {}
        }
        let r = eval_expr(rhs, env)?;
        let Value::Bool(rb) = r else {
            return Err(RuntimeError::LogicalOperandNotBool {
                op,
                type_name: type_name(&r),
            });
        };
        return Ok(Value::Bool(rb));
    }

    let l = eval_expr(lhs, env)?;
    let r = eval_expr(rhs, env)?;
    match (op, &l, &r) {
        (BinaryOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Arc::new(
            Rational::from(a.as_ref() + b.as_ref()),
        ))),
        (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Arc::new(
            Rational::from(a.as_ref() - b.as_ref()),
        ))),
        (BinaryOp::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Arc::new(
            Rational::from(a.as_ref() * b.as_ref()),
        ))),
        (BinaryOp::Div, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                return Err(RuntimeError::DivisionByZero);
            }
            Ok(Value::Number(Arc::new(Rational::from(
                a.as_ref() / b.as_ref(),
            ))))
        }
        (BinaryOp::Mod, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                return Err(RuntimeError::ModuloByZero);
            }
            Ok(Value::Number(Arc::new(rat_mod(a, b))))
        }
        (BinaryOp::Pow, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Arc::new(rat_pow(a, b)?)))
        }
        (BinaryOp::Add, Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b).into()))
        }
        (BinaryOp::Eq, a, b) => Ok(Value::Bool(a == b)),
        (BinaryOp::Ne, a, b) => Ok(Value::Bool(a != b)),
        (BinaryOp::Lt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
        (BinaryOp::Le, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
        (BinaryOp::Gt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
        (BinaryOp::Ge, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
        (BinaryOp::Lt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        (BinaryOp::Le, Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
        (BinaryOp::Gt, Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
        (BinaryOp::Ge, Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
        (BinaryOp::Match, Value::String(a), Value::String(b)) => Ok(Value::Bool(a.contains(&**b))),
        (BinaryOp::NotMatch, Value::String(a), Value::String(b)) => {
            Ok(Value::Bool(!a.contains(&**b)))
        }
        (op, a, b) => Err(RuntimeError::BinaryTypeError {
            op,
            lhs: type_name(a),
            rhs: type_name(b),
        }),
    }
}
