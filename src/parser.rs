use crate::ast::*;
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use pest::Parser;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

#[derive(pest_derive::Parser)]
#[grammar = "ff.pest"]
pub struct FFParser;

lazy_static! {
    static ref PRATT: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::logical_or, Assoc::Left))
        .op(Op::infix(Rule::logical_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::ne, Assoc::Left))
        .op(Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::le, Assoc::Left)
            | Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::ge, Assoc::Left)
            | Op::infix(Rule::match_op, Assoc::Left)
            | Op::infix(Rule::not_match, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::subtract, Assoc::Left))
        .op(Op::infix(Rule::multiply, Assoc::Left)
            | Op::infix(Rule::divide, Assoc::Left)
            | Op::infix(Rule::modulo, Assoc::Left))
        .op(Op::infix(Rule::power, Assoc::Right))
        .op(Op::prefix(Rule::neg) | Op::prefix(Rule::logical_not))
        .op(Op::postfix(Rule::call_args) | Op::postfix(Rule::dot_access));
}

pub fn parse(input: &str) -> Result<Program> {
    let mut pairs = FFParser::parse(Rule::program, input)?;
    let program = pairs.next().ok_or_else(|| anyhow!("empty parse tree"))?;
    build_program(program)
}

fn build_program(pair: Pair<Rule>) -> Result<Program> {
    let mut statements = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::assignment => statements.push(Statement::Assignment(build_assignment(inner)?)),
            Rule::expr => statements.push(Statement::Expr(build_expr(inner)?)),
            Rule::EOI => {}
            r => return Err(anyhow!("unexpected rule in program: {:?}", r)),
        }
    }
    Ok(Program { statements })
}

fn build_assignment(pair: Pair<Rule>) -> Result<Assignment> {
    let mut inner = pair.into_inner();
    let pattern = build_pattern(inner.next().ok_or_else(|| anyhow!("missing pattern"))?)?;
    let _assign = inner.next();
    let value = build_expr(inner.next().ok_or_else(|| anyhow!("missing value"))?)?;
    Ok(Assignment { pattern, value })
}

fn build_pattern(pair: Pair<Rule>) -> Result<Pattern> {
    match pair.as_rule() {
        Rule::wildcard => Ok(Pattern::Wildcard),
        Rule::ident => Ok(Pattern::Ident(pair.as_str().to_string())),
        Rule::number => Ok(Pattern::Number(pair.as_str().parse()?)),
        Rule::string => Ok(Pattern::String(unquote(pair.as_str()))),
        Rule::bool => Ok(Pattern::Bool(pair.as_str() == "true")),
        Rule::pattern_list => Ok(Pattern::List(
            pair.into_inner()
                .map(build_pattern_item)
                .collect::<Result<_>>()?,
        )),
        Rule::pattern_tuple => Ok(Pattern::Tuple(
            pair.into_inner()
                .map(build_pattern_item)
                .collect::<Result<_>>()?,
        )),
        Rule::pattern_dict => {
            let mut entries = Vec::new();
            for entry in pair.into_inner() {
                let mut inner = entry.into_inner();
                let key = build_expr(inner.next().ok_or_else(|| anyhow!("missing dict key"))?)?;
                let val =
                    build_pattern(inner.next().ok_or_else(|| anyhow!("missing dict value"))?)?;
                entries.push((key, val));
            }
            Ok(Pattern::Dict(entries))
        }
        Rule::pattern_set => Ok(Pattern::Set(
            pair.into_inner()
                .map(build_pattern)
                .collect::<Result<_>>()?,
        )),
        r => Err(anyhow!("unexpected pattern rule: {:?}", r)),
    }
}

fn build_pattern_item(pair: Pair<Rule>) -> Result<PatternItem> {
    match pair.as_rule() {
        Rule::pattern_rest => {
            let name = pair.into_inner().next().map(|p| p.as_str().to_string());
            Ok(PatternItem::Rest(name))
        }
        _ => Ok(PatternItem::Pattern(build_pattern(pair)?)),
    }
}

fn build_expr(pair: Pair<Rule>) -> Result<Expr> {
    PRATT
        .map_primary(build_primary)
        .map_prefix(|op, rhs| {
            let op = match op.as_rule() {
                Rule::neg => UnaryOp::Neg,
                Rule::logical_not => UnaryOp::Not,
                r => return Err(anyhow!("unexpected prefix: {:?}", r)),
            };
            Ok(Expr::Unary {
                op,
                operand: Box::new(rhs?),
            })
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::call_args => {
                let args = op
                    .into_inner()
                    .map(build_expr)
                    .collect::<Result<Vec<_>>>()?;
                Ok(curry_call(lhs?, args))
            }
            Rule::dot_access => {
                let inner = op.into_inner().next().ok_or_else(|| anyhow!("empty dot"))?;
                let key = match inner.as_rule() {
                    Rule::dot_index => AccessKey::Index(inner.as_str().parse()?),
                    Rule::ident => AccessKey::Field(inner.as_str().to_string()),
                    r => return Err(anyhow!("unexpected dot key: {:?}", r)),
                };
                Ok(Expr::Access {
                    target: Box::new(lhs?),
                    key,
                })
            }
            r => Err(anyhow!("unexpected postfix: {:?}", r)),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::power => BinaryOp::Pow,
                Rule::multiply => BinaryOp::Mul,
                Rule::divide => BinaryOp::Div,
                Rule::modulo => BinaryOp::Mod,
                Rule::add => BinaryOp::Add,
                Rule::subtract => BinaryOp::Sub,
                Rule::eq => BinaryOp::Eq,
                Rule::ne => BinaryOp::Ne,
                Rule::lt => BinaryOp::Lt,
                Rule::le => BinaryOp::Le,
                Rule::gt => BinaryOp::Gt,
                Rule::ge => BinaryOp::Ge,
                Rule::logical_and => BinaryOp::And,
                Rule::logical_or => BinaryOp::Or,
                Rule::match_op => BinaryOp::Match,
                Rule::not_match => BinaryOp::NotMatch,
                r => return Err(anyhow!("unexpected infix: {:?}", r)),
            };
            Ok(Expr::Binary {
                op,
                lhs: Box::new(lhs?),
                rhs: Box::new(rhs?),
            })
        })
        .parse(pair.into_inner())
}

fn build_primary(pair: Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::number => Ok(Expr::Number(pair.as_str().parse()?)),
        Rule::string => Ok(Expr::String(unquote(pair.as_str()))),
        Rule::bool => Ok(Expr::Bool(pair.as_str() == "true")),
        Rule::ident => Ok(Expr::Ident(pair.as_str().to_string())),
        Rule::list => Ok(Expr::List(
            pair.into_inner().map(build_expr).collect::<Result<_>>()?,
        )),
        Rule::tuple => Ok(Expr::Tuple(
            pair.into_inner().map(build_expr).collect::<Result<_>>()?,
        )),
        Rule::dict => {
            let mut entries = Vec::new();
            for entry in pair.into_inner() {
                let mut inner = entry.into_inner();
                let k = build_expr(inner.next().ok_or_else(|| anyhow!("missing dict key"))?)?;
                let v = build_expr(inner.next().ok_or_else(|| anyhow!("missing dict value"))?)?;
                entries.push((k, v));
            }
            Ok(Expr::Dict(entries))
        }
        Rule::set => Ok(Expr::Set(
            pair.into_inner().map(build_expr).collect::<Result<_>>()?,
        )),
        Rule::function => {
            let mut inner = pair.into_inner();
            let params_pair = inner.next().ok_or_else(|| anyhow!("missing params"))?;
            let body_pair = inner.next().ok_or_else(|| anyhow!("missing body"))?;
            let params: Vec<String> = params_pair
                .into_inner()
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(curry_function(params, build_expr(body_pair)?))
        }
        Rule::if_expr => {
            let mut inner = pair.into_inner();
            let cond = Box::new(build_expr(inner.next().unwrap())?);
            let then_branch = Box::new(build_expr(inner.next().unwrap())?);
            let else_branch = Box::new(build_expr(inner.next().unwrap())?);
            Ok(Expr::If {
                cond,
                then_branch,
                else_branch,
            })
        }
        Rule::match_expr => {
            let mut inner = pair.into_inner();
            let scrutinee = Box::new(build_expr(inner.next().unwrap())?);
            let arms = inner
                .map(|arm| {
                    let mut p = arm.into_inner();
                    let pattern = build_pattern(p.next().unwrap())?;
                    let body = build_expr(p.next().unwrap())?;
                    Ok(MatchArm { pattern, body })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Expr::Match { scrutinee, arms })
        }
        Rule::expr => build_expr(pair),
        r => Err(anyhow!("unexpected primary: {:?}", r)),
    }
}

fn unquote(s: &str) -> String {
    s[1..s.len() - 1].to_string()
}

// Multi-param `(x, y, z) -> body` desugars to `(x) -> (y) -> (z) -> body`.
// Zero-param functions are preserved as-is.
fn curry_function(params: Vec<String>, body: Expr) -> Expr {
    if params.len() <= 1 {
        return Expr::Function {
            params,
            body: Box::new(body),
        };
    }
    let mut acc = body;
    for p in params.into_iter().rev() {
        acc = Expr::Function {
            params: vec![p],
            body: Box::new(acc),
        };
    }
    acc
}

// Multi-arg `f(a, b, c)` desugars to `f(a)(b)(c)`. Zero-arg calls `f()`
// remain a single call so zero-param functions still fire.
fn curry_call(callee: Expr, args: Vec<Expr>) -> Expr {
    if args.is_empty() {
        return Expr::Call {
            callee: Box::new(callee),
            args,
        };
    }
    let mut acc = callee;
    for a in args {
        acc = Expr::Call {
            callee: Box::new(acc),
            args: vec![a],
        };
    }
    acc
}
