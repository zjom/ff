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

// --- pattern matching in function parameters --------------------------------

#[test]
fn param_list_destructure() {
    assert_eq!(eval("f = [a, b] => a + b\nf([3, 4])"), "7");
}

#[test]
fn param_list_with_rest() {
    assert_eq!(
        eval("f = [a, ..rest] => rest\nf([1, 2, 3, 4])"),
        "[2, 3, 4]"
    );
}

#[test]
fn param_cons_destructure() {
    assert_eq!(eval("f = h :: t => h\nf([1, 2, 3])"), "1");
    assert_eq!(eval("f = h :: t => t\nf([1, 2, 3])"), "[2, 3]");
}

#[test]
fn param_chained_cons() {
    // `a :: b :: rest => ...` peels two elements.
    assert_eq!(
        eval("f = a :: b :: rest => rest\nf([1, 2, 3, 4])"),
        "[3, 4]"
    );
    assert_eq!(eval("f = a :: b :: rest => b\nf([1, 2, 3, 4])"), "2");
}

#[test]
fn param_atom_tagged_pair() {
    assert_eq!(eval("f = [:ok, v] => v\nf([:ok, 42])"), "42");
}

#[test]
fn param_object_destructure() {
    assert_eq!(
        eval(
            r#"f = {"name": n} => n
f({"name": "ada", "age": 36})"#
        ),
        r#""ada""#
    );
}

#[test]
fn param_object_atom_key_destructure() {
    assert_eq!(
        eval("f = {:name: n} => n\nf({:name: \"ada\", :age: 36})"),
        r#""ada""#
    );
}

#[test]
fn param_object_shorthand() {
    assert_eq!(
        eval(
            r#"f = {name} => name
f({:name: "ada"})"#
        ),
        r#""ada""#
    );
}

#[test]
fn param_cons_with_list_head() {
    // `[key, value] :: rest` peels an entry of an object as a [k, v] pair.
    assert_eq!(
        eval(
            r#"f = [k, v] :: rest => k
f({"a": 1})"#
        ),
        r#""a""#
    );
}

#[test]
fn param_wildcard() {
    assert_eq!(eval("f = _ => 42\nf(99)"), "42");
}

#[test]
fn param_literal_pattern_failure_is_runtime_error() {
    // Calling a function whose param pattern doesn't match should error.
    let src = "f = [:ok, v] => v\nf([:error, \"oops\"])";
    let prog = f2::parser::parse(src).expect("parse");
    assert!(f2::interpreter::run(&prog).is_err());
}

#[test]
fn param_mixed_destructure_multi_arg() {
    // Each curried param can be its own pattern.
    assert_eq!(eval("f = ([a, b], c) => a + b + c\nf([1, 2], 3)"), "6");
}

#[test]
fn param_nested_destructure() {
    assert_eq!(
        eval("f = [[a, b], [c, d]] => a + b + c + d\nf([[1, 2], [3, 4]])"),
        "10"
    );
}

#[test]
fn param_cons_in_recursive_function() {
    // Recursive sum via cons-pattern param plus a fallback arm.
    let src = "sum = match\n  h :: t -> h + sum(t),\n  _ -> 0\nsum([1, 2, 3, 4])";
    assert_eq!(eval(src), "10");
}
