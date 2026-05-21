use std::sync::{Arc, Mutex};

use crate::ast::{BinaryOp, Expr};

use super::error::{RuntimeError, RuntimeResult};
use super::expr::eval_expr;
use super::scope::Env;
use super::value::{LazyState, Value, type_name};

pub(super) fn eval_binary(
    op: BinaryOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &Env,
) -> RuntimeResult<Value> {
    match op {
        // `a :: b` is non-strict in `b`: evaluate the head eagerly and capture
        // the rhs as a thunk so recursive stdlib builders (e.g. `f(x) ::
        // map(f, rest)`) don't bottom out before pattern-matching the tail.
        BinaryOp::Cons => {
            let head = eval_expr(lhs, env)?;
            let tail = Arc::new(Mutex::new(LazyState::Pending {
                body: rhs.clone(),
                env: env.clone(),
            }));
            Ok(Value::Cons {
                head: Arc::new(head),
                tail,
            })
        }
        BinaryOp::And | BinaryOp::Or => {
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
            Ok(Value::Bool(rb))
        }
    }
}
