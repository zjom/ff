; Comments
(comment) @comment

; Literals
(number) @number
(string) @string
(bool) @constant.builtin.boolean
(atom) @constant
(wildcard) @variable.builtin

; Keywords
"if"     @keyword.control.conditional
"then"   @keyword.control.conditional
"else"   @keyword.control.conditional
"match"  @keyword.control
"import" @keyword.control.import
"export" @keyword.control.import

; Function literal arrow and match arrow
"=>" @operator
"->" @operator

; Assignment
"=" @operator

; Built-in operators
(power) @operator
(multiply) @operator
(divide) @operator
(modulo) @operator
(add) @operator
(subtract) @operator
(eq) @operator
(ne) @operator
(le) @operator
(ge) @operator
(lt) @operator
(gt) @operator
(not_match) @operator
(match_op) @operator
(logical_and) @operator
(logical_or) @operator
(neg) @operator
(logical_not) @operator
(cons_op) @operator

; Custom operators (user-defined)
(custom_op_pow)  @operator
(custom_op_mult) @operator
(custom_op_add)  @operator
(custom_op_cat)  @operator
(custom_op_comp) @operator
(custom_op_and)  @operator
(custom_op_or)   @operator
(custom_prefix)  @operator

; Range punctuation
"..=" @operator
".."  @operator

; Function and call structure
(function params: (params) @variable.parameter)
(call_expr fn: (ident) @function)
(juxt_expr fn: (ident) @function)
(dot_expr field: (ident) @property)
(dot_expr field: (dot_index) @property)

; Assignment of a function literal: highlight LHS as a function name
(assignment
  lhs: (ident) @function
  rhs: (function))

; Match arms
(match_arm pattern: (_) @variable)
(match_guard "if" @keyword.control.conditional)

; Identifiers default
(ident) @variable

; Pattern-rest binder
(pattern_rest (ident) @variable.parameter)

; Object/set/list punctuation
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket

","  @punctuation.delimiter
";"  @punctuation.delimiter
":"  @punctuation.delimiter
"."  @punctuation.delimiter

; Operator-as-value: `(++)` should look like a function name.
(op_paren) @function
