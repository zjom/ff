mod common;
use common::eval;

#[test]
fn function_value_display() {
    assert_eq!(eval("(x) => x + 1"), "<fn (x)>");
}

#[test]
fn call_one_arg_parens() {
    assert_eq!(eval("inc = (x) => x + 1\ninc(5)"), "6");
}

#[test]
fn call_one_arg_juxt() {
    assert_eq!(eval("inc = (x) => x + 1\ninc 5"), "6");
}

#[test]
fn bare_ident_param() {
    assert_eq!(eval("inc = x => x + 1\ninc(5)"), "6");
}

#[test]
fn arrow_right_associative() {
    // `x => y => body` should be `x => (y => body)`, a curried 2-arg function.
    assert_eq!(eval("add = x => y => x + y\nadd(3)(4)"), "7");
}

#[test]
fn multi_param_desugar() {
    // `(x, y) => body` is sugar for `(x) => (y) => body`.
    assert_eq!(eval("add = (x, y) => x + y\nadd(3, 4)"), "7");
    assert_eq!(eval("add = (x, y) => x + y\nadd(3)(4)"), "7");
}

#[test]
fn space_separated_bare_params() {
    // `a b c => ...` is equivalent to `(a, b, c) => ...`.
    assert_eq!(eval("add = a b c => a + b + c\nadd(1, 2, 3)"), "6");
    assert_eq!(eval("add = a b c => a + b + c\nadd 1 2 3"), "6");
}

#[test]
fn space_separated_params_in_parens() {
    assert_eq!(eval("add = (a b c) => a + b + c\nadd(1, 2, 3)"), "6");
}

#[test]
fn mixed_comma_and_space_params() {
    assert_eq!(eval("add = (a, b c) => a + b + c\nadd(1, 2, 3)"), "6");
    assert_eq!(eval("add = a, b c => a + b + c\nadd(1, 2, 3)"), "6");
}

#[test]
fn juxt_call_chain() {
    assert_eq!(eval("add = x => y => x + y\nadd 3 4"), "7");
}

#[test]
fn partial_application() {
    assert_eq!(eval("add = (x, y) => x + y\ninc = add(1)\ninc(99)"), "100");
}

#[test]
fn partial_application_via_juxt() {
    assert_eq!(eval("add = x => y => x + y\ninc = add 1\ninc 41"), "42");
}

#[test]
fn zero_arg_function_preserved() {
    assert_eq!(eval("f = () => 42\nf()"), "42");
}

#[test]
fn closure_captures_outer_variable() {
    assert_eq!(eval("n = 10\nf = (x) => x + n\nf(5)"), "15");
}

#[test]
fn closure_captures_value_at_definition() {
    // Reassigning `n` after the closure is built must still use the live scope
    // value, since the closure captures the scope by shared handle.
    assert_eq!(eval("n = 10\nf = (x) => x + n\nn = 100\nf(0)"), "100");
}

#[test]
fn recursion_works() {
    assert_eq!(
        eval("fact = (n) => if n == 0 then 1 else n * fact(n - 1)\nfact(6)"),
        "720"
    );
}

#[test]
fn juxt_tighter_than_infix() {
    // `inc 5 + 2` must parse as `(inc 5) + 2`.
    assert_eq!(eval("inc = x => x + 1\ninc 5 + 2"), "8");
}

#[test]
fn function_as_argument() {
    assert_eq!(
        eval("apply = (f, x) => f(x)\napply((y) => y * 2, 21)"),
        "42"
    );
}

#[test]
fn function_returned_from_function() {
    assert_eq!(
        eval("mk = (n) => (x) => x + n\nadd5 = mk(5)\nadd5(10)"),
        "15"
    );
}
