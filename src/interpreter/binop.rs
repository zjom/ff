use anyhow::{Result, bail};
use rug::Rational;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr};

use super::expr::eval_expr;
use super::number::{rat_mod, rat_pow};
use super::scope::Env;
use super::value::{Value, type_name};

pub(super) fn eval_binary(op: BinaryOp, lhs: &Expr, rhs: &Expr, env: &Env) -> Result<Value> {
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
        (BinaryOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Rc::new(
            Rational::from(a.as_ref() + b.as_ref()),
        ))),
        (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Rc::new(
            Rational::from(a.as_ref() - b.as_ref()),
        ))),
        (BinaryOp::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(Rc::new(
            Rational::from(a.as_ref() * b.as_ref()),
        ))),
        (BinaryOp::Div, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                bail!("division by zero");
            }
            Ok(Value::Number(Rc::new(Rational::from(
                a.as_ref() / b.as_ref(),
            ))))
        }
        (BinaryOp::Mod, Value::Number(a), Value::Number(b)) => {
            if b.cmp0() == std::cmp::Ordering::Equal {
                bail!("modulo by zero");
            }
            Ok(Value::Number(Rc::new(rat_mod(a, b))))
        }
        (BinaryOp::Pow, Value::Number(a), Value::Number(b)) => {
            Ok(Value::Number(Rc::new(rat_pow(a, b)?)))
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
        (op, a, b) => bail!(
            "cannot apply {:?} to {} and {}",
            op,
            type_name(a),
            type_name(b)
        ),
    }
}
