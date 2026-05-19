mod common;
use common::{eval_err, parse_err};

#[test]
fn undefined_variable() {
    let msg = eval_err("undef_var");
    assert!(msg.contains("undefined"), "got: {}", msg);
}

#[test]
fn type_error_arithmetic() {
    let msg = eval_err(r#"1 + "two""#);
    assert!(
        msg.contains("Add") || msg.contains("number"),
        "got: {}",
        msg
    );
}

#[test]
fn if_condition_must_be_bool() {
    let msg = eval_err("if 1 then 0 else 1");
    assert!(msg.contains("bool"), "got: {}", msg);
}

#[test]
fn call_non_function() {
    let msg = eval_err("42(1)");
    assert!(msg.contains("non-function"), "got: {}", msg);
}

#[test]
fn arity_mismatch() {
    let msg = eval_err("f = (x) => x\nf(1, 2)");
    // Currying means f(1, 2) = f(1)(2); the result is `1` and we call `1(2)`.
    assert!(
        msg.contains("non-function") || msg.contains("expects"),
        "got: {}",
        msg
    );
}

#[test]
fn list_index_out_of_range() {
    let msg = eval_err("[1, 2, 3].5");
    assert!(msg.contains("out of range"), "got: {}", msg);
}

#[test]
fn object_missing_key() {
    let msg = eval_err(r#"{"a": 1}.b"#);
    assert!(msg.contains("no key"), "got: {}", msg);
}

#[test]
fn no_match_arm_matches() {
    let msg = eval_err("match 99\n  0 -> 0,\n  1 -> 1");
    assert!(msg.contains("no match"), "got: {}", msg);
}

#[test]
fn destructure_shape_mismatch_is_runtime_error() {
    let msg = eval_err("[a, b, c] = [1, 2]\na");
    assert!(msg.contains("pattern"), "got: {}", msg);
}

#[test]
fn parse_error_on_dangling_operator() {
    let msg = parse_err("1 +");
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_on_unclosed_bracket() {
    let msg = parse_err("[1, 2");
    assert!(!msg.is_empty());
}
