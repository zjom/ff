#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Export(ExportKind),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportKind {
    All,
    Names(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub pattern: Pattern,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(rug::Rational),
    String(String),
    Bool(bool),
    Ident(String),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Set(Vec<Expr>),
    Function {
        params: Vec<String>,
        body: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },
    Import(Box<Expr>),
    Scope(Vec<Statement>),
    Access {
        target: Box<Expr>,
        key: AccessKey,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Number(rug::Rational),
    String(String),
    Bool(bool),
    List(Vec<PatternItem>),
    Tuple(Vec<PatternItem>),
    Dict(Vec<(Expr, Pattern)>),
    Set(Vec<Pattern>),
    // `head ++ tail`. Type-polymorphic destructuring: splits off the first
    // element of any value the `++` operator can build (list, string, tuple,
    // set, dict). Right-associative — `a ++ b ++ rest` nests as `Cons(a, Cons(b, rest))`.
    Cons {
        head: Box<Pattern>,
        tail: Box<Pattern>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternItem {
    Pattern(Pattern),
    Rest(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessKey {
    Index(usize),
    Field(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Pow,
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Match,
    NotMatch,
    // `a :: b` — non-strict in `b`. Evaluates lhs eagerly and captures rhs as
    // a thunk so recursive stdlib builders like `f(x) :: map(f, rest)` don't
    // force their tail until pattern-matched.
    Cons,
}
