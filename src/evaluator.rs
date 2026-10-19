use crate::tokens::{Token, TokenList};
use crate::ast::Expr;
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

// evaluator
pub fn evaluate(expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        // for literals, value is already known
        Expr::Number(n)     => Ok(Value::Num(*n)),
        Expr::Str(s)        => Ok(Value::Str(s.clone())),
        Expr::Bool(b)       => Ok(Value::Bool(*b)),
        Expr::Nil           => Ok(Value::Nil),

        // parentheses carry no meaning at runtime
        Expr::Grouping(inner)   => evaluate(inner),

        //operand first, check its type, then apply op
        Expr::Unary { op, right } => {
            let right = evaluate(right)?;
            match op.token_type {
                TokenList::Minus => {
                    let n = check_number_operand(op, &right)?;
                    Ok(Value::Num(-n))
                }
                TokenList::Not => Ok(Value::Bool(!is_truthy(&right))),
                _ => Err(RuntimeError::new(op, "Unknown unary operator.")),
                }
            }

        // left operand, then right, then op
        Expr::Binary { left, op, right } => {
            let left    = evaluate(left)?;
            let right   = evaluate(right)?;
            match op.token_type {
                TokenList::Minus => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Num(a - b))
                }
                TokenList::Plus => match (&left, &right) {
                        (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                        _ => Err(RuntimeError::new(op, "Operands must be two numbers or two strings.")),
                }
                TokenList::Star => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Num(a * b))
                }
                TokenList::Slash => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    if b == 0.0 {
                        return Err(RuntimeError::new(op, "Division by zero."))
                    }
                    Ok(Value::Num(a / b))
                }
                TokenList::Modulo => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    if b == 0.0 {
                        return Err(RuntimeError::new(op, "Division by zero."))
                    }
                    Ok(Value::Num(a % b))
                }
                TokenList::Less => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Bool(a < b))
                }
                TokenList::LessEql => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Bool(a <= b))
                }
                TokenList::Greater => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Bool(a > b))
                }
                TokenList::GreaterEql => {
                    let (a, b) = check_number_operands(op, &left, &right)?;
                    Ok(Value::Bool(a >= b))
                }
                TokenList::EqualTo => Ok(Value::Bool(is_equal(&left, &right))),
                TokenList::NotEqual => Ok(Value::Bool(!is_equal(&left, &right))),
                
                _ => Err(RuntimeError::new(op, "Unknown binary operator.")),
            }
        }

        Expr::Logical { left, op, right} => {
            let left    = evaluate(left)?;
            match op.token_type  {
                TokenList::Or   => {
                    if is_truthy(&left) {
                        return Ok(left)
                    } 
                }
                TokenList::And => {
                    if !is_truthy(&left) {
                        return Ok(left)
                    }
                }
                _ => return Err(RuntimeError::new(op, "Unknown logical operator.")),
            }
            evaluate(right)
        }

        Expr::Program(exprs) => {
            let mut last = Value::Nil;
            for e in exprs {
                last = evaluate(e)?;
            }
            Ok(last)
        }
        
        Expr::Variable(name) => Err(RuntimeError::new(name, "Variables are not supported yet.")),
        Expr::Assign { name, .. } => Err(RuntimeError::new(name, "Variables are not supported yet.")),
    }
}
