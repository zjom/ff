#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Expr(Expr),
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
}
