//! Standardized error type emitted by the interpreter.
//!
//! Every interpreter and prelude entry point returns `RuntimeResult<T>`.
//! Outer drivers (`eval_file`, the REPL) convert these into `anyhow::Error`
//! at the boundary via `?`.

use thiserror::Error;

use crate::ast::{BinaryOp, UnaryOp};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Error)]
pub enum RuntimeError {
    // ---- name resolution ----------------------------------------------------
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("export: undefined variable `{0}`")]
    ExportUndefined(String),

    // ---- call dispatch ------------------------------------------------------
    #[error("function expects {expected} arg(s), got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("native `{name}` expects {expected} arg(s), got {got}")]
    NativeArity {
        name: &'static str,
        expected: usize,
        got: usize,
    },

    #[error("native `{name}` over-applied (arity {arity})")]
    NativeOverApplied { name: &'static str, arity: usize },

    #[error("cannot call non-function: {0}")]
    NotCallable(&'static str),

    // ---- control flow -------------------------------------------------------
    #[error("if condition must be Bool, got {0}")]
    IfConditionNotBool(&'static str),

    #[error("match guard must be Bool, got {0}")]
    MatchGuardNotBool(&'static str),

    #[error("no match arm matched")]
    NoMatchArm,

    // ---- patterns -----------------------------------------------------------
    #[error("pattern match failed in assignment")]
    AssignmentPatternFailed,

    #[error("multiple `..` patterns in one sequence")]
    MultipleRestPatterns,

    // ---- ranges -------------------------------------------------------------
    #[error("Range start must be a Number, got {0}")]
    RangeStartNotNumber(&'static str),

    #[error("Range end must be a Number, got {0}")]
    RangeEndNotNumber(&'static str),

    // ---- access -------------------------------------------------------------
    #[error("index {index} out of range (len {len})")]
    IndexOutOfRange { index: usize, len: usize },

    #[error("Object has no key {0:?}")]
    ObjectMissingField(String),

    #[error("Object has no key :{0}")]
    ObjectMissingAtom(String),

    #[error("Module has no member `.{0}`")]
    ModuleMissingMember(String),

    #[error("cannot index into {0}")]
    CannotIndex(&'static str),

    #[error("cannot read field .{field} from {type_name}")]
    CannotReadField {
        field: String,
        type_name: &'static str,
    },

    #[error("cannot read field .:{atom} from {type_name}")]
    CannotReadAtomField {
        atom: String,
        type_name: &'static str,
    },

    // ---- operators ----------------------------------------------------------
    #[error("cannot apply {op:?} to {type_name}")]
    UnaryTypeError {
        op: UnaryOp,
        type_name: &'static str,
    },

    #[error("{op:?} expects bool, got {type_name}")]
    LogicalOperandNotBool {
        op: BinaryOp,
        type_name: &'static str,
    },

    #[error("cannot apply {op:?} to {lhs} and {rhs}")]
    BinaryTypeError {
        op: BinaryOp,
        lhs: &'static str,
        rhs: &'static str,
    },

    #[error("division by zero")]
    DivisionByZero,

    #[error("modulo by zero")]
    ModuloByZero,

    #[error("** requires an integer exponent, got {0}")]
    PowNonIntegerExponent(String),

    #[error("** exponent out of range: {0}")]
    PowExponentOutOfRange(String),

    #[error("0 cannot be raised to a negative power")]
    ZeroToNegativePower,

    // ---- imports ------------------------------------------------------------
    #[error("import expects String path, got {0}")]
    ImportPathNotString(&'static str),

    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("parse error: {0}")]
    Parse(String),

    // ---- natives ------------------------------------------------------------
    #[error("panic: {0}")]
    Panic(String),

    #[error("{native} expected a {expected}, got {got}")]
    NativeTypeError {
        native: &'static str,
        expected: &'static str,
        got: &'static str,
    },

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // ---- interop (serde) ----------------------------------------------------
    #[error("serialize: {0}")]
    Serialize(String),

    #[error("deserialize: {0}")]
    Deserialize(String),

    #[error("cannot deserialize Object with non-string/atom key: {0}")]
    NonStringObjectKey(String),

    #[error("cannot deserialize lazy values; collect into a List first")]
    LazyDeserialize,

    #[error("cannot deserialize a Function or Module")]
    FunctionDeserialize,

    #[error("cannot represent {0} as a float")]
    CannotRepresentAsFloat(String),
}
