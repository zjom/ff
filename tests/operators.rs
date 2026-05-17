mod common;
use common::{eval, eval_err, parse_err};

// --- defining and using infix operators -------------------------------------

#[test]
fn define_and_use_infix() {
    assert_eq!(eval("(++) = (x, y) => x + y\n1 ++ 2"), "3");
}

#[test]
fn operator_as_value() {
    // `(++)` is just an identifier; binding it to another name works.
    assert_eq!(eval("(++) = (x, y) => x + y\nplus = (++)\nplus(2, 3)"), "5");
}

#[test]
fn operator_curried_via_partial_application() {
    assert_eq!(eval("(++) = (x, y) => x + y\ninc = (++)(1)\ninc(10)"), "11");
}

#[test]
fn operator_call_with_juxtaposition() {
    assert_eq!(eval("(++) = (x, y) => x + y\n(++) 4 5"), "9");
}

// --- precedence buckets (OCaml-style by first char) -------------------------

#[test]
fn mult_bucket_binds_tighter_than_add() {
    // `*^` starts with `*` -> multiplicative level. `1 + 2 *^ 3` = 1 + 6 = 7.
    assert_eq!(eval("(*^) = (x, y) => x * y\n1 + 2 *^ 3"), "7");
}

#[test]
fn add_bucket_binds_looser_than_mult() {
    // `++` is additive level: `1 ++ 2 * 3` = 1 ++ (2*3) = 7.
    assert_eq!(eval("(++) = (x, y) => x + y\n1 ++ 2 * 3"), "7");
}

#[test]
fn comp_bucket_below_additive() {
    // `<=>` is comparison level: `1 + 2 <=> 3` = 3 <=> 3.
    assert_eq!(eval("(<=>) = (x, y) => x == y\n1 + 2 <=> 3"), "true");
}

#[test]
fn cat_bucket_between_comparison_and_additive() {
    // `^^` is at "cat" level — tighter than comparison, looser than additive.
    // `1 + 2 ^^ 3 + 4 == 12` -> (1+2) ^^ (3+4) == 12. With ^^ as *, that's 21 == 12 -> false.
    assert_eq!(
        eval("(^^) = (x, y) => x * y\n1 + 2 ^^ 3 + 4 == 12"),
        "false"
    );
    // Comparison happens last: (3 ^^ 7) == 12 -> 21 == 12.
    assert_eq!(eval("(^^) = (x, y) => x * y\n1 + 2 ^^ 3 + 4 == 21"), "true");
}

// --- associativity ----------------------------------------------------------

#[test]
fn additive_custom_op_is_left_assoc() {
    // `(--)` = x - y, left-assoc at additive level: (10 -- 5) -- 2 = 3.
    assert_eq!(eval("(--) = (x, y) => x - y\n10 -- 5 -- 2"), "3");
}

#[test]
fn cat_custom_op_is_right_assoc() {
    // `(^^)` = x - y, right-assoc at cat level: 2 ^^ (1 ^^ 3) = 2 - (1-3) = 4.
    assert_eq!(eval("(^^) = (x, y) => x - y\n2 ^^ 1 ^^ 3"), "4");
}

// --- prefix operators -------------------------------------------------------

#[test]
fn custom_prefix_operator() {
    assert_eq!(eval("(~~) = x => x + 100\n~~5"), "105");
}

#[test]
fn custom_prefix_binds_tighter_than_infix() {
    // `~~5 + 1` -> (~~5) + 1 -> 105 + 1.
    assert_eq!(eval("(~~) = x => x + 100\n~~5 + 1"), "106");
}

#[test]
fn custom_prefix_question_mark() {
    assert_eq!(eval("(?+) = x => x + 1\n?+10"), "11");
}

// --- interactions with built-ins --------------------------------------------

#[test]
fn builtin_arithmetic_unchanged() {
    assert_eq!(eval("1 + 2 * 3"), "7");
    assert_eq!(eval("!!true"), "true");
    assert_eq!(eval("-(-5)"), "5");
}

#[test]
fn match_arrow_still_works_with_dash_chars() {
    // `->` must remain reserved for match arms even though `--`, `-<` etc. are
    // valid custom operators.
    let src = "describe = match\n  0 -> \"zero\",\n  _ -> \"other\"\ndescribe(0)";
    assert_eq!(eval(src), "\"zero\"");
}

#[test]
fn fat_arrow_still_works_for_functions() {
    // `=>` must remain reserved for function literals; `==>` etc. are custom.
    assert_eq!(eval("f = x => x + 1\nf(2)"), "3");
}

#[test]
fn custom_op_with_equals_prefix() {
    // `==>` starts with `=` but is not `=>`; comp-level left assoc.
    assert_eq!(eval("(==>) = (x, y) => x + y\n2 ==> 3"), "5");
}

// --- error paths ------------------------------------------------------------

#[test]
fn undefined_operator_is_runtime_error() {
    let err = eval_err("1 +++ 2");
    assert!(err.contains("undefined"), "unexpected error: {}", err);
}

#[test]
fn op_paren_requires_op_chars() {
    // `(abc)` is a grouped ident expression, not an op_paren.
    parse_err("(abc) = 1");
}
