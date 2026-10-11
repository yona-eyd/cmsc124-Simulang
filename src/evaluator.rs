use crate::tokens::Token;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> Self {
        RuntimeError { line: token.line, message: message.to_string() }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] Runtime error: {}", self.line, self.message)
    }
}

pub fn is_equal(a: &Value, b: &Value) -> bool {
    match(a,b) {
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    }
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Nil => false,
        _ => true,
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub fn check_number_operand(token: &Token, operand: &Value) -> Result<f64, RuntimeError> {
    if let Value::Num(n) = operand {
        Ok(*n)
    } else {
        Err(RuntimeError::new(token, "Operand must be a number."))
    }
}

pub fn check_number_operands(token: &Token, left: &Value, right: &Value) -> Result<(f64, f64), RuntimeError> {
    match (left, right) {
    (Value::Num(a), Value::Num(b)) => Ok((*a, *b)),
    _ => Err(RuntimeError::new(token, "Operands must be numbers.")),
}
}

