/**
 * tree-sitter grammar for the ff (a.k.a. f2) language.
 *
 * Mirrors src/ff.pest. Operator precedence follows the Pratt table built
 * in src/parser.rs (OCaml-style: bucket by first character of the operator).
 *
 * Newlines are significant — they separate statements, and they are also
 * a valid collection separator inside [], {}, and call argument lists.
 * They are NOT included in `extras`; only horizontal whitespace and
 * comments are skipped automatically.
 */

const OP_CHAR_CLASS = '[!$%&*+\\-/<=>?@^|~]';
const OP_TAIL = `(?:${OP_CHAR_CLASS})+`;
const OP_TAIL_OPT = `(?:${OP_CHAR_CLASS})*`;

// Precedence table. Higher numbers bind tighter.
const PREC = {
  assignment: 1,
  function: 2,
  match_expr: 3,
  if_expr: 3,
  import: 3,
  range: 4,
  or: 5,         // ||  and custom_op_or  (left)
  and: 6,        // &&  and custom_op_and (left)
  eq: 7,         // == !=  and custom_op_comp (left)
  comp: 8,       // < <= > >= ~ !~  (left)
  cat: 9,        // custom_op_cat and ::  (right)
  add: 10,       // + -  and custom_op_add (left)
  mult: 11,      // * / %  and custom_op_mult (left)
  power: 12,     // ** ^  and custom_op_pow (right)
  prefix: 13,    // - ! and custom_prefix
  juxt: 14,      // f x
  postfix: 15,   // f(...), f.x
};

module.exports = grammar({
  name: 'ff',

  extras: $ => [
    /[ \t\r]/,
    $.comment,
  ],

  word: $ => $.ident,

  conflicts: $ => [
    // `{ k: v }` could be an object expression or an object pattern.
    [$.object, $.pattern_object],
    // `[a, b]` could be a list or a list pattern.
    [$.list, $.pattern_list],
    // `ident` could be a pattern or a primary expression.
    [$._pattern_atom, $._primary],
    // After `{ expr \n`, can't tell if the next `\n` continues the
    // collection separator (set / pattern_set) or is the `_nl` between an
    // object_entry's key and `:`.
    [$.object_entry, $.set, $.pattern_object_entry],
    // `(\n...` could begin a `unit`/`pattern_unit` or a multi-statement
    // `scope` — only the next non-newline token decides.
    [$.scope, $.unit, $.pattern_unit],
    // `()` itself: unit (expression) vs pattern_unit (pattern).
    [$.unit, $.pattern_unit],
    // `params` is `pattern (, pattern)*` — overlaps with comma-separated
    // pattern positions when reached without the leading `(`.
    [$.params, $._pattern_item],
    [$.params, $.pattern_set],
    [$.params, $.pattern_object_entry],
    // `{x}` is ambiguous between a one-element `set`, a one-element
    // `pattern_set`, and a `pattern_object` with shorthand entry `x`.
    [$._primary, $._pattern_atom, $.pattern_object_entry],
    [$._pattern_atom, $.pattern_object_entry],
    [$._primary, $.pattern_object_entry],
  ],

  rules: {
    program: $ => seq(
      optional($._stmt_sep),
      optional(seq(
        $._statement,
        repeat(seq($._stmt_sep, $._statement)),
        optional($._stmt_sep),
      )),
    ),

    // ----- Trivia -----
    comment: _ => token(seq('#', /[^\n]*/)),

    _stmt_sep: _ => repeat1(choice('\n', ';')),
    _nl: _ => repeat1('\n'),

    // Collection separator: any non-empty run of commas and/or newlines.
    _coll_sep: _ => repeat1(choice(',', '\n')),

    // ----- Statements -----
    _statement: $ => choice(
      $.export_stmt,
      $.assignment,
      $._expr,
    ),

    assignment: $ => prec(PREC.assignment, seq(
      field('lhs', $._pattern),
      '=',
      field('rhs', $._expr),
    )),

    export_stmt: $ => seq(
      'export',
      optional($._nl),
      choice($.export_all, $.export_names),
    ),
    export_all: _ => '*',
    export_names: $ => prec.right(seq(
      $.export_name,
      repeat(seq(',', optional($._nl), $.export_name)),
      optional(','),
    )),
    export_name: $ => choice($.op_paren, $.ident),

    // ----- Expressions -----
    _expr: $ => choice(
      $.function,
      $.match_expr,
      $.if_expr,
      $.import_expr,
      $.range_expr,
      $.binary_expr,
      $.unary_expr,
      $._postfix_expr,
    ),

    // Function literal: `params => body`. Parameters are space-separated;
    // commas are optional. `() => body` is the zero-arg form.
    function: $ => prec.right(PREC.function, seq(
      field('params', $.params),
      '=>',
      field('body', $._expr),
    )),

    params: $ => choice(
      // Explicit empty form `()`. Higher precedence so it wins over the
      // alternative where `()` would be a `pattern_unit` (as the sole pattern
      // of the no-paren form).
      prec(2, seq('(', ')')),
      seq(
        '(',
        $._pattern,
        repeat(seq(optional(','), $._pattern)),
        optional(','),
        ')',
      ),
      prec.right(seq(
        $._pattern,
        repeat(seq(optional(','), $._pattern)),
      )),
    ),

    // If: newlines allowed around all keywords.
    if_expr: $ => prec.right(PREC.if_expr, seq(
      'if',    optional($._nl), field('cond', $._expr),
      optional($._nl), 'then', optional($._nl), field('then', $._expr),
      optional($._nl), 'else', optional($._nl), field('else', $._expr),
    )),

    // Match: arms separated by commas (newlines allowed around them).
    // The scrutinee is optional; the multi-line form puts a newline between
    // scrutinee and the first arm, the single-line form puts `:`.
    match_expr: $ => prec.right(PREC.match_expr, choice(
      // multi-line: `match value\n  pat -> ...`
      seq(
        'match',
        optional(field('scrutinee', $._expr)),
        $._nl,
        $.match_arm,
        repeat(seq(',', optional($._nl), $.match_arm)),
      ),
      // single-line: `match value: pat -> ..., ...`
      seq(
        'match',
        optional(seq(field('scrutinee', $._expr), ':')),
        optional($._nl),
        $.match_arm,
        repeat(seq(',', optional($._nl), $.match_arm)),
      ),
    )),

    match_arm: $ => seq(
      field('pattern', $._pattern),
      optional($.match_guard),
      '->',
      field('body', $._expr),
    ),
    match_guard: $ => seq('if', $._expr),

    // `import <primary>`
    import_expr: $ => prec(PREC.import, seq(
      'import',
      optional($._nl),
      $._primary,
    )),

    // ----- Ranges -----
    // Distinguished as their own expression form (not infix) so they don't
    // need a precedence slot in the Pratt table.
    range_expr: $ => prec(PREC.range, choice(
      // [a..=b]
      seq($._postfix_expr, '..=', $._postfix_expr),
      // [a..b]
      seq($._postfix_expr, '..', $._postfix_expr),
      // [a..]
      prec(PREC.range - 1, seq($._postfix_expr, '..')),
    )),

    // ----- Binary (infix) expressions -----
    binary_expr: $ => choice(
      // Power: right-assoc
      prec.right(PREC.power, seq($._expr, choice($.power, $.custom_op_pow), $._expr)),
      // Multiplicative: left-assoc
      prec.left(PREC.mult, seq($._expr,
        choice($.multiply, $.divide, $.modulo, $.custom_op_mult),
        $._expr)),
      // Additive: left-assoc
      prec.left(PREC.add, seq($._expr,
        choice($.add, $.subtract, $.custom_op_add),
        $._expr)),
      // Cat / cons: right-assoc
      prec.right(PREC.cat, seq($._expr,
        choice($.custom_op_cat, $.cons_op),
        $._expr)),
      // Comparison: left-assoc
      prec.left(PREC.comp, seq($._expr,
        choice($.lt, $.le, $.gt, $.ge, $.match_op, $.not_match),
        $._expr)),
      // Equality: left-assoc
      prec.left(PREC.eq, seq($._expr,
        choice($.eq, $.ne, $.custom_op_comp),
        $._expr)),
      // Logical and: left-assoc
      prec.left(PREC.and, seq($._expr,
        choice($.logical_and, $.custom_op_and),
        $._expr)),
      // Logical or: left-assoc
      prec.left(PREC.or, seq($._expr,
        choice($.logical_or, $.custom_op_or),
        $._expr)),
    ),

    // ----- Prefix / unary -----
    unary_expr: $ => prec.right(PREC.prefix, seq(
      choice($.neg, $.logical_not, $.custom_prefix),
      $._expr,
    )),

    // ----- Postfix: call, dot, juxtaposition -----
    _postfix_expr: $ => choice(
      $.call_expr,
      $.dot_expr,
      $.juxt_expr,
      $._primary,
    ),

    call_expr: $ => prec(PREC.postfix, seq(
      field('fn', $._postfix_expr),
      $.call_args,
    )),

    call_args: $ => seq(
      '(',
      optional(seq(
        optional($._nl),
        $._expr,
        repeat(seq(optional($._nl), ',', optional($._nl), $._expr)),
        optional(','),
      )),
      optional($._nl),
      ')',
    ),

    dot_expr: $ => prec(PREC.postfix, seq(
      field('object', $._postfix_expr),
      '.',
      field('field', choice($.dot_index, $.ident)),
    )),
    dot_index: _ => /[0-9]+/,

    // Juxt's argument intentionally excludes `(...)`-shaped primaries so that
    // `f(x)` is always a `call_expr`, never a juxt with a paren_expr arg.
    // To pass a paren_expr by juxtaposition, write `f (x)` — same in pest,
    // since `postfix_op` lists `call_args` before `juxt_arg`.
    juxt_expr: $ => prec.left(PREC.juxt, seq(
      field('fn', $._postfix_expr),
      field('arg', $._juxt_arg),
    )),

    _juxt_arg: $ => choice(
      $.number,
      $.string,
      $.bool,
      $.atom,
      $.list,
      $.object,
      $.set,
      $.ident,
    ),

    // ----- Primary expressions -----
    _primary: $ => choice(
      $.number,
      $.string,
      $.bool,
      $.atom,
      $.list,
      $.object,
      $.set,
      $.op_paren,
      $.scope,
      $.unit,
      $.paren_expr,
      $.ident,
    ),

    // `(x)` is parsed here. Higher precedence than `scope` so a single
    // expression in parens is `paren_expr`, not a one-statement `scope`.
    paren_expr: $ => prec(1, seq('(', $._expr, ')')),

    // Multi-statement block. A `scope` is `(` followed by one-or-more
    // statements. `paren_expr` outranks scope for the single-expression case,
    // so a `scope` only wins when (a) there's an assignment or (b) there
    // are two or more statements.
    scope: $ => seq(
      '(',
      optional($._stmt_sep),
      $._statement,
      repeat(seq($._stmt_sep, $._statement)),
      optional($._stmt_sep),
      ')',
    ),

    unit: $ => seq('(', optional($._nl), ')'),

    // ----- Collections -----
    // Lists allow a `range_expr` element (e.g. `[0..10]`) — but since
    // `range_expr` is itself an `_expr`, no special-case is needed.
    list: $ => seq(
      '[',
      optional($._coll_sep),
      optional(seq(
        $._expr,
        repeat(seq($._coll_sep, $._expr)),
        optional($._coll_sep),
      )),
      ']',
    ),

    object: $ => seq(
      '{',
      optional($._coll_sep),
      optional(seq(
        $.object_entry,
        repeat(seq($._coll_sep, $.object_entry)),
        optional($._coll_sep),
      )),
      '}',
    ),
    object_entry: $ => seq(
      field('key', $._expr),
      optional($._nl),
      ':',
      optional($._nl),
      field('value', $._expr),
    ),

    set: $ => seq(
      '{',
      optional($._coll_sep),
      $._expr,
      repeat(seq($._coll_sep, $._expr)),
      optional($._coll_sep),
      '}',
    ),

    // ----- Patterns -----
    _pattern: $ => choice(
      $.pattern_cons,
      $._pattern_atom,
    ),

    pattern_cons: $ => prec.right(seq(
      $._pattern_atom,
      repeat1(seq($.cons_op, $._pattern_atom)),
    )),

    _pattern_atom: $ => choice(
      $.pattern_list,
      $.op_paren,
      $.pattern_unit,
      $.pattern_object,
      $.pattern_set,
      $.number,
      $.string,
      $.bool,
      $.atom,
      $.wildcard,
      $.ident,
    ),

    pattern_unit: $ => seq('(', optional($._nl), ')'),

    pattern_list: $ => seq(
      '[',
      optional($._coll_sep),
      optional(seq(
        $._pattern_item,
        repeat(seq($._coll_sep, $._pattern_item)),
        optional($._coll_sep),
      )),
      ']',
    ),
    _pattern_item: $ => choice($.pattern_rest, $._pattern),
    pattern_rest: $ => seq('..', optional($.ident)),

    pattern_object: $ => seq(
      '{',
      optional($._coll_sep),
      optional(seq(
        $.pattern_object_entry,
        repeat(seq($._coll_sep, $.pattern_object_entry)),
        optional($._coll_sep),
      )),
      '}',
    ),
    pattern_object_entry: $ => choice(
      seq(field('key', $._expr), optional($._nl), ':', optional($._nl), field('value', $._pattern)),
      $.ident,
    ),

    pattern_set: $ => seq(
      '{',
      optional($._coll_sep),
      $._pattern,
      repeat(seq($._coll_sep, $._pattern)),
      optional($._coll_sep),
      '}',
    ),

    // ----- Literals -----
    // `0..` must remain two tokens so the range syntax works. The fractional
    // part requires at least one digit, so `0.5` is a decimal but `0..5`
    // lexes as `0`, `..`, `5`.
    number: _ => token(/[0-9]+(\.[0-9]+)?/),
    string: _ => token(seq('"', /[^"]*/, '"')),
    bool: _ => token(prec(1, choice('true', 'false'))),
    ident: _ => token(/[A-Za-z_][A-Za-z0-9_]*/),
    wildcard: _ => token(prec(2, '_')),
    atom: _ => token(/:[A-Za-z_][A-Za-z0-9_]*/),

    // ----- Operator tokens -----
    // Built-in tokens are exact; if a longer operator-character sequence
    // is present (`+>`, `**+`, ...), tree-sitter's longest-match lexer
    // emits the corresponding `custom_op_*` token instead.
    power:        _ => choice('**', '^'),
    multiply:     _ => '*',
    divide:       _ => '/',
    modulo:       _ => '%',
    add:          _ => '+',
    subtract:     _ => '-',

    eq:           _ => '==',
    ne:           _ => '!=',
    le:           _ => '<=',
    ge:           _ => '>=',
    lt:           _ => '<',
    gt:           _ => '>',
    not_match:    _ => '!~',
    match_op:     _ => '~',

    logical_and:  _ => '&&',
    logical_or:   _ => '||',

    neg:          _ => '-',
    logical_not:  _ => '!',

    cons_op:      _ => '::',

    // Custom operator buckets. Regex is bounded so longest-match works:
    // each starts with the bucket's first-char set and continues with op_chars.
    // `=>` and `->` are reserved arrows so they're excluded from the `=`/`-`
    // buckets via a leading character constraint.
    custom_op_pow:  _ => token(new RegExp(`\\*\\*${OP_TAIL}`)),
    custom_op_mult: _ => token(new RegExp(`[*/%]${OP_TAIL}`)),
    custom_op_add:  _ => token(choice(
      new RegExp(`\\+${OP_TAIL}`),
      new RegExp(`-[!$%&*+\\-/<=?@^|~]${OP_TAIL_OPT}`), // `-` then non-`>` op_char
    )),
    custom_op_cat:  _ => token(new RegExp(`[\\^@]${OP_TAIL_OPT}`)),
    custom_op_comp: _ => token(choice(
      new RegExp(`[<>!$]${OP_TAIL}`),
      new RegExp(`=[!$%&*+\\-/<=?@^|~]${OP_TAIL_OPT}`), // `=` then non-`>` op_char
    )),
    custom_op_and:  _ => token(new RegExp(`&${OP_TAIL}`)),
    custom_op_or:   _ => token(new RegExp(`\\|${OP_TAIL}`)),

    // Prefix operator: starts with `?` or `~`, at least two chars.
    custom_prefix:  _ => token(new RegExp(`[?~]${OP_TAIL}`)),

    // Parenthesized operator name acts as an identifier.
    op_paren: $ => seq('(', $._op_paren_name, ')'),
    _op_paren_name: _ => token(choice('::', /[!$%&*+\-/<=>?@^|~]+/)),
  },
});
