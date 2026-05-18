mod common;
use common::eval;

// --- destructuring assignment -----------------------------------------------

#[test]
fn destructure_list_exact() {
    assert_eq!(eval("[a, b, c] = [1, 2, 3]\nb"), "2");
}

#[test]
fn destructure_with_rest() {
    assert_eq!(eval("[head, ..tail] = [1, 2, 3, 4]\ntail"), "[2, 3, 4]");
}

#[test]
fn destructure_rest_in_middle() {
    assert_eq!(eval("[first, ..mid, last] = [1, 2, 3, 4, 5]\nmid"), "[2, 3, 4]");
    assert_eq!(eval("[first, ..mid, last] = [1, 2, 3, 4, 5]\nlast"), "5");
}

#[test]
fn destructure_anonymous_rest() {
    assert_eq!(eval("[head, ..] = [10, 20, 30]\nhead"), "10");
}

#[test]
fn destructure_unit() {
    assert_eq!(eval("() = ()\n1"), "1");
}

#[test]
fn destructure_dict() {
    assert_eq!(eval(r#"{"name": who} = {"name": "ada", "age": 36}
who"#), r#""ada""#);
}

#[test]
fn destructure_nested() {
    assert_eq!(eval("[[a, b], [c, d]] = [[1, 2], [3, 4]]\na + d"), "5");
}

// --- match expression -------------------------------------------------------

#[test]
fn match_literal_arm() {
    assert_eq!(eval("match 1\n  0 -> \"zero\",\n  1 -> \"one\",\n  n -> \"other\""), "\"one\"");
}

#[test]
fn match_binding_arm() {
    assert_eq!(eval("match 99\n  0 -> 0,\n  n -> n * 2"), "198");
}

#[test]
fn match_wildcard_arm() {
    assert_eq!(eval("match 42\n  0 -> \"zero\",\n  _ -> \"other\""), "\"other\"");
}

#[test]
fn match_list_pattern() {
    assert_eq!(eval("match [1, 2, 3]\n  [] -> 0,\n  [x] -> 1,\n  _ -> 99"), "99");
}

#[test]
fn match_list_rest() {
    assert_eq!(
        eval("match [1, 2, 3, 4]\n  [x, ..rest] -> rest"),
        "[2, 3, 4]"
    );
}

#[test]
fn match_dict_pattern() {
    // Dict pattern matches if all required keys exist; extras allowed.
    assert_eq!(
        eval(r#"match {"name": "ada", "age": 36}
  {"name": n} -> n"#),
        r#""ada""#
    );
}

#[test]
fn match_trailing_comma() {
    assert_eq!(eval("match 1\n  0 -> \"zero\",\n  1 -> \"one\",\n"), "\"one\"");
}

#[test]
fn match_arms_inline_with_commas() {
    assert_eq!(eval("match 2\n  1 -> \"a\", 2 -> \"b\", _ -> \"c\""), "\"b\"");
}

#[test]
fn match_missing_comma_is_parse_error() {
    // Newlines no longer separate arms; missing comma must fail to parse.
    assert!(ff::parser::parse("match 1\n  0 -> \"zero\"\n  1 -> \"one\"").is_err());
}

#[test]
fn match_pattern_failure_is_runtime_error() {
    // None of the arms match → runtime error (we test the error path elsewhere).
    let src = "match 99\n  0 -> 0,\n  1 -> 1";
    let prog = ff::parser::parse(src).expect("parse");
    assert!(ff::interpreter::run(&prog).is_err());
}

// --- cons-pattern (`h :: t`) ------------------------------------------------

#[test]
fn cons_pattern_list_destructure() {
    assert_eq!(eval("h :: t = [1, 2, 3]\nh"), "1");
    assert_eq!(eval("h :: t = [1, 2, 3]\nt"), "[2, 3]");
}

#[test]
fn cons_pattern_string_destructure() {
    assert_eq!(eval(r#"h :: t = "abc"
h"#), r#""a""#);
    assert_eq!(eval(r#"h :: t = "abc"
t"#), r#""bc""#);
}

#[test]
fn cons_pattern_set_destructure() {
    assert_eq!(eval("h :: t = {1, 2, 3}\nh"), "1");
    assert_eq!(eval("h :: t = {1, 2, 3}\nt"), "{2, 3}");
}

#[test]
fn cons_pattern_dict_destructure() {
    // Head is the first entry as a [key, value] list — the inverse of how
    // `::` prepends a `[k, v]` pair onto a dict.
    assert_eq!(eval(r#"h :: t = {"a": 1, "b": 2}
h"#), r#"["a", 1]"#);
    assert_eq!(eval(r#"h :: t = {"a": 1, "b": 2}
t"#), r#"{"b": 2}"#);
}

#[test]
fn cons_pattern_in_match_list() {
    let src = "match [1, 2, 3]\n  [] -> \"empty\",\n  h :: t -> t";
    assert_eq!(eval(src), "[2, 3]");
}

#[test]
fn cons_pattern_chained_is_right_assoc() {
    // `a :: b :: rest` peels two elements: a=1, b=2, rest=[3, 4].
    assert_eq!(eval("a :: b :: rest = [1, 2, 3, 4]\nrest"), "[3, 4]");
    assert_eq!(eval("a :: b :: rest = [1, 2, 3, 4]\nb"), "2");
}

#[test]
fn cons_pattern_empty_fails_to_match() {
    let src = "match []\n  h :: t -> \"non-empty\"";
    let prog = ff::parser::parse(src).expect("parse");
    assert!(ff::interpreter::run(&prog).is_err());
}

#[test]
fn cons_pattern_literal_head() {
    // Head can be any pattern, including a literal — matches only when the
    // first element equals it.
    let src = "match [1, 2, 3]\n  1 :: rest -> rest,\n  _ -> [99]";
    assert_eq!(eval(src), "[2, 3]");
    let src2 = "match [9, 2, 3]\n  1 :: rest -> rest,\n  _ -> [99]";
    assert_eq!(eval(src2), "[99]");
}

// --- guard clauses ----------------------------------------------------------

#[test]
fn match_guard_picks_first_truthy_arm() {
    let src = "match 1\n  n if n < 2 -> \"lt 2\",\n  n -> \"ge 2\"";
    assert_eq!(eval(src), "\"lt 2\"");
}

#[test]
fn match_guard_falls_through_when_false() {
    let src = "match 5\n  n if n < 2 -> \"lt 2\",\n  n -> \"ge 2\"";
    assert_eq!(eval(src), "\"ge 2\"");
}

#[test]
fn match_guard_can_reference_bound_idents() {
    // Guard sees bindings produced by the pattern.
    let src = "match [1, 2, 3]\n  [x, y, ..] if x < y -> \"ascending\",\n  _ -> \"other\"";
    assert_eq!(eval(src), "\"ascending\"");
}

#[test]
fn match_guard_multiple_arms_same_pattern() {
    // Same `n` pattern, different guards — first truthy wins.
    let src = "f = match\n  n if n < 0 -> \"neg\",\n  n if n == 0 -> \"zero\",\n  n -> \"pos\"\nf(-3)";
    assert_eq!(eval(src), "\"neg\"");
    let src = "f = match\n  n if n < 0 -> \"neg\",\n  n if n == 0 -> \"zero\",\n  n -> \"pos\"\nf 0";
    assert_eq!(eval(src), "\"zero\"");
    let src = "f = match\n  n if n < 0 -> \"neg\",\n  n if n == 0 -> \"zero\",\n  n -> \"pos\"\nf 7";
    assert_eq!(eval(src), "\"pos\"");
}

#[test]
fn match_guard_failure_with_no_fallback_is_runtime_error() {
    let src = "match 5\n  n if n < 2 -> \"lt 2\"";
    let prog = ff::parser::parse(src).expect("parse");
    assert!(ff::interpreter::run(&prog).is_err());
}

#[test]
fn match_guard_non_bool_is_runtime_error() {
    let src = "match 1\n  n if 42 -> \"yes\"";
    let prog = ff::parser::parse(src).expect("parse");
    assert!(ff::interpreter::run(&prog).is_err());
}

#[test]
fn match_guard_can_use_outer_bindings() {
    let src = "lo = 10\nmatch 5\n  n if n < lo -> \"small\",\n  _ -> \"big\"";
    assert_eq!(eval(src), "\"small\"");
}

// --- match-as-function ------------------------------------------------------

#[test]
fn match_as_function_via_call() {
    assert_eq!(
        eval("f = match\n  0 -> \"zero\",\n  n -> \"other\"\nf(0)"),
        r#""zero""#
    );
}

#[test]
fn match_as_function_via_juxt() {
    assert_eq!(
        eval("f = match\n  0 -> \"zero\",\n  n -> \"other\"\nf 5"),
        r#""other""#
    );
}

#[test]
fn match_as_function_recursive() {
    assert_eq!(
        eval("fact = match\n  0 -> 1,\n  n -> n * fact(n - 1)\nfact 6"),
        "720"
    );
}

#[test]
fn match_as_function_list_dispatch() {
    let src = "describe = match\n  [] -> \"empty\",\n  [x] -> \"one\",\n  _ -> \"many\"\ndescribe [1, 2, 3]";
    assert_eq!(eval(src), r#""many""#);
}

#[test]
fn match_atom_literal() {
    let src = "name = match\n  :ok -> \"yes\",\n  :error -> \"no\",\n  _ -> \"?\"\nname :error";
    assert_eq!(eval(src), r#""no""#);
}

#[test]
fn match_tagged_pair() {
    let src = "result = [:ok, 42]\nmatch result\n  [:ok, v] -> v,\n  [:error, _] -> -1";
    assert_eq!(eval(src), "42");
}

#[test]
fn destructure_dict_with_atom_key() {
    assert_eq!(
        eval("{:name: who} = {:name: \"ada\", :age: 36}\nwho"),
        r#""ada""#
    );
}
