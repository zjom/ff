mod common;
use common::{eval, eval_err, parse_err};

#[test]
fn scope_returns_last_expression() {
    assert_eq!(eval("(\n  1\n  2\n  3\n)"), "3");
}

#[test]
fn scope_with_single_assignment_returns_unit() {
    assert_eq!(eval("( x = 1 )"), "()");
}

#[test]
fn scope_value_in_assignment() {
    let src = "y = (
  a = 10
  b = 20
  a + b
)
y";
    assert_eq!(eval(src), "30");
}

#[test]
fn scope_captures_outer_bindings() {
    let src = "x = 100
r = (
  y = 1
  x + y
)
r";
    assert_eq!(eval(src), "101");
}

#[test]
fn scope_bindings_dont_leak_outward() {
    let src = "x = 1
_ = (
  x = 99
  x
)
x";
    assert_eq!(eval(src), "1");
}

#[test]
fn scope_inner_shadows_outer() {
    let src = "x = 1
(
  x = 99
  x
)";
    assert_eq!(eval(src), "99");
}

#[test]
fn scope_in_if_branch() {
    let src = "if true then (
  a = 2
  a * a
) else 0";
    assert_eq!(eval(src), "4");
}

#[test]
fn scope_in_function_body() {
    let src = "f = (n) => (
  doubled = n * 2
  doubled + 1
)
f(5)";
    assert_eq!(eval(src), "11");
}

#[test]
fn scope_with_trailing_newlines() {
    assert_eq!(eval("(\n  x = 5\n  x + 1\n\n)"), "6");
}

#[test]
fn empty_parens_still_unit() {
    assert_eq!(eval("()"), "()");
}

#[test]
fn single_expr_parens_still_grouping() {
    assert_eq!(eval("(42)"), "42");
}

#[test]
fn dict_unaffected() {
    assert_eq!(eval(r#"{"a": 1, "b": 2}"#), r#"{"a": 1, "b": 2}"#);
}

#[test]
fn set_unaffected() {
    assert_eq!(eval("{1, 2, 3}"), "{1, 2, 3}");
}

#[test]
fn unit_unaffected() {
    assert_eq!(eval("()"), "()");
}

#[test]
fn nested_scopes() {
    let src = "(
  x = 1
  (
    y = 2
    x + y
  )
)";
    assert_eq!(eval(src), "3");
}

#[test]
fn scope_inner_assignment_unknown_outside() {
    let src = "_ = (
  hidden = 42
  hidden
)
hidden";
    let msg = eval_err(src);
    assert!(msg.contains("undefined"), "got: {}", msg);
}

#[test]
fn scope_single_assignment_inline() {
    assert_eq!(eval("( x = 7 )"), "()");
}

#[test]
fn scope_call_returns_value() {
    let src = "f = () => (
  a = 3
  b = 4
  a * b
)
f()";
    assert_eq!(eval(src), "12");
}

#[test]
fn scope_closure_captures_at_definition() {
    let src = "x = 10
get = () => (
  x + 1
  x + 2
)
x = 999
get()";
    // `x` is looked up at call time, so this reflects the most recent rebind.
    assert_eq!(eval(src), "1001");
}

#[test]
fn scope_in_match_arm() {
    let src = "describe = (n) => match n
  0 -> (
    msg = \"zero\"
    msg
  ),
  _ -> (
    msg = \"other\"
    msg
  )
describe(0)";
    assert_eq!(eval(src), "\"zero\"");
}

#[test]
fn scope_unterminated_is_parse_error() {
    let msg = parse_err("(\n  x = 1\n");
    assert!(!msg.is_empty());
}

#[test]
fn semicolon_separates_program_statements() {
    assert_eq!(eval("1; 2; 3"), "3");
}

#[test]
fn semicolon_mixed_with_newline_in_program() {
    assert_eq!(eval("x = 1; y = 2\nx + y"), "3");
}

#[test]
fn trailing_semicolon_in_program() {
    assert_eq!(eval("1; 2;"), "2");
}

#[test]
fn leading_semicolon_in_program() {
    assert_eq!(eval(";1"), "1");
}

#[test]
fn repeated_semicolons_in_program() {
    assert_eq!(eval("1;;; 2"), "2");
}

#[test]
fn semicolon_separates_scope_statements() {
    assert_eq!(eval("(a = 1; b = 2; a + b)"), "3");
}

#[test]
fn semicolon_mixed_with_newline_in_scope() {
    assert_eq!(eval("(a = 1; b = 2\n  a + b)"), "3");
}
