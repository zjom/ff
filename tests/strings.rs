mod common;
use common::{eval, eval_err};

fn with_string(src: &str) -> String {
    eval(&format!("String = import \"String\"\n{}", src))
}

fn with_string_err(src: &str) -> String {
    eval_err(&format!("String = import \"String\"\n{}", src))
}

#[test]
fn format_basic_substitution() {
    assert_eq!(
        with_string(r#"String.format("hello, {}!", ["world"])"#),
        r#""hello, world!""#
    );
}

#[test]
fn format_multiple_sequential_args() {
    assert_eq!(
        with_string(r#"String.format("{} + {} = {}", [1, 2, 3])"#),
        r#""1 + 2 = 3""#
    );
}

#[test]
fn format_positional_index() {
    // `{N}` picks by index and can repeat.
    assert_eq!(
        with_string(r#"String.format("{0} {1} {0}", ["ping", "pong"])"#),
        r#""ping pong ping""#
    );
}

#[test]
fn format_positional_does_not_advance_sequential() {
    // Mirrors Rust: explicit `{0}` doesn't consume the implicit counter,
    // so the following `{}` is still the first sequential arg.
    assert_eq!(
        with_string(r#"String.format("{0} {}", ["a"])"#),
        r#""a a""#
    );
}

#[test]
fn format_debug_spec_quotes_strings() {
    // `{}` splices the bare string; `{:?}` falls back to ff's default
    // display, which quotes strings.
    assert_eq!(
        with_string(r#"String.format("{:?} vs {}", ["hi", "hi"])"#),
        r#""\"hi\" vs hi""#
    );
}

#[test]
fn format_indexed_debug_spec() {
    assert_eq!(
        with_string(r#"String.format("{0:?}", ["hi"])"#),
        r#""\"hi\"""#
    );
}

#[test]
fn format_literal_braces() {
    assert_eq!(
        with_string(r#"String.format("{{}} and {{{}}}", ["x"])"#),
        r#""{} and {x}""#
    );
}

#[test]
fn format_empty_template_and_no_args() {
    assert_eq!(with_string(r#"String.format("", [])"#), r#""""#);
}

#[test]
fn format_template_without_holes_ignores_args() {
    assert_eq!(
        with_string(r#"String.format("static text", [1, 2, 3])"#),
        r#""static text""#
    );
}

#[test]
fn format_atom_renders_with_colon() {
    assert_eq!(
        with_string(r#"String.format("status: {}", [:ok])"#),
        r#""status: :ok""#
    );
}

#[test]
fn format_number_and_list() {
    assert_eq!(
        with_string(r#"String.format("n={}, xs={}", [1/3, [1, 2, 3]])"#),
        r#""n=1/3, xs=[1, 2, 3]""#
    );
}

#[test]
fn format_pipe_composition_data_last() {
    // args-last lets data flow through `|>` naturally.
    let src = r#"["alice", 30] |> String.format("name={}, age={}")"#;
    assert_eq!(with_string(src), r#""name=alice, age=30""#);
}

#[test]
fn format_accepts_cons_spine_from_map() {
    // `map` yields a lazy cons spine; format must flatten it like List.* does.
    let src = r#"
        ages = map(x => x * 10, [1, 2, 3])
        String.format("{} {} {}", ages)
    "#;
    assert_eq!(with_string(src), r#""10 20 30""#);
}

// ---- error paths ----------------------------------------------------------

#[test]
fn format_unclosed_brace_errors() {
    let msg = with_string_err(r#"String.format("oops {", [])"#);
    assert!(msg.contains("unclosed"), "got: {}", msg);
}

#[test]
fn format_stray_close_brace_errors() {
    let msg = with_string_err(r#"String.format("hi }", [])"#);
    assert!(msg.contains("stray") || msg.contains("}"), "got: {}", msg);
}

#[test]
fn format_missing_arg_errors() {
    let msg = with_string_err(r#"String.format("{}", [])"#);
    assert!(msg.contains("missing argument"), "got: {}", msg);
}

#[test]
fn format_out_of_range_index_errors() {
    let msg = with_string_err(r#"String.format("{5}", ["only-one"])"#);
    assert!(msg.contains("missing argument 5"), "got: {}", msg);
}

#[test]
fn format_invalid_index_errors() {
    let msg = with_string_err(r#"String.format("{abc}", [])"#);
    assert!(msg.contains("invalid argument index"), "got: {}", msg);
}

#[test]
fn format_unsupported_spec_errors() {
    let msg = with_string_err(r#"String.format("{:x}", [42])"#);
    assert!(msg.contains("unsupported spec"), "got: {}", msg);
}

#[test]
fn format_non_string_template_errors() {
    let msg = with_string_err(r#"String.format(42, [])"#);
    assert!(
        msg.contains("expected a string") || msg.contains("string"),
        "got: {}",
        msg
    );
}

#[test]
fn format_non_list_args_errors() {
    let msg = with_string_err(r#"String.format("{}", "not a list")"#);
    assert!(
        msg.contains("expected a list") || msg.contains("list"),
        "got: {}",
        msg
    );
}
