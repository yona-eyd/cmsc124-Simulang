use crate::tokens::{Token, TokenList};
use crate::ast::{Expr,Stmt};
use crate::context::{Context, ContextRef};
use std::fmt;
use std::rc::Rc;
use crate::callable::{Callable, UserFunction};
use crate::natives::{self, NativeFn};

// in 5PR3: Debug + PartialEq removed
// compiler can't auto-write them for a trait object 
#[derive(Clone)]    
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Nil,
    Callable(Rc<dyn Callable>),
    Native(&'static NativeFn),
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

// two ways a statement can stop early: error or a return
pub enum Interrupt {
    Runtime(RuntimeError),                  // a real error
    Return { value: Value, line: usize }    // a return statement: line is for error msgs
}

// lets self.evaluate(e)? inside execute turn a RuntimeError into an Interrupt
impl From<RuntimeError> for Interrupt {
    fn from(e: RuntimeError) -> Self {
        Interrupt::Runtime(e)
    }
}


pub fn is_equal(a: &Value, b: &Value) -> bool {
    match(a,b) {
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        // returns true only when x and y are the very same object
        (Value::Callable(x), Value::Callable(y)) => Rc::ptr_eq(x, y),
        (Value::Native(x), Value::Native(y)) => std::ptr::eq(*x, *y),
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
            Value::Num(n)       => write!(f, "{}", n),
            Value::Str(s)       => write!(f, "{}", s),
            Value::Bool(b)      => write!(f, "{}", b),
            Value::Nil          => write!(f, "nil"),
            Value::Callable(c)  => write!(f, "{}", c.describe()),
            Value::Native(n) => write!(f, "<native {}>", n.name),
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

fn check_arity(paren: &Token, expected: usize, got: usize) -> Result<(), RuntimeError> {
    if expected != got {
        Err(RuntimeError::new(
            paren,
            &format!("Expected {expected} arguments but got {got}."),
        ))
    } else {
        Ok(())
    }
}

pub struct Interpreter {
    context: ContextRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let it = Interpreter {
            context: Context::new_global(),
            //add seed here
        };
        // natives exist before any user code runs
        for n in natives::NATIVES {
            it.context.borrow_mut().define_global(n.name, Value::Native(n));
        }
        it
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            // for literals, value is already known
            Expr::Number(n)     => Ok(Value::Num(*n)),
            Expr::Str(s)        => Ok(Value::Str(s.clone())),
            Expr::Bool(b)       => Ok(Value::Bool(*b)),
            Expr::Nil           => Ok(Value::Nil),

            // parentheses carry no meaning at runtime
            Expr::Grouping(inner)   => self.evaluate(inner),

            //operand first, check its type, then apply op
            Expr::Unary { op, right } => {
                let right = self.evaluate(right)?;
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
                let left    = self.evaluate(left)?;
                let right   = self.evaluate(right)?;
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
                let left    = self.evaluate(left)?;
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
                self.evaluate(right)
            }

            
            Expr::Variable(name) => self.context.borrow().get(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.context.borrow_mut().assign(name, value.clone())?;
                Ok(value)   // assignment is an expression: its value is the assigned value
            }

            // a call expression: evaluate callee & arguments, check them, then run the function
            Expr::Call { callee, paren, arguments } => {
                let callee_value = self.evaluate(callee)?;
                
                // evaluate arguments left to right into a Vec<Value>
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }

                // check if function is callable
                // only callables can be called
                match callee_value {
                    Value::Callable(function) => {
                        check_arity(paren, function.arity(), args.len())?;
                        function.call(self, args)
                    }
                    Value::Native(native) => {
                        check_arity(paren, native.arity, args.len())?;
                        (native.func)(self, &args, paren)
                    }
                    _ => Err(RuntimeError::new(paren, "Can only call functions.")),
                }
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), Interrupt> {
        match stmt {
            Stmt::Expression(e) => { self.evaluate(e)?; Ok(()) }
            Stmt::Print(e) => {
                let value = self.evaluate(e)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                // initializer first, so `var a = a;` sees an outer a
                let value = self.evaluate(initializer)?;
                self.context.borrow_mut().define(name, value)?;     // added ? to convert error into interrupt
                Ok(())
            }
            Stmt::Block(stmts) => {
                let inner = Context::new_enclosed(&self.context);
                self.execute_block(stmts, inner)
            }
            Stmt::When { condition, then_branch, else_branch } => {
                let cond = self.evaluate(condition)?;
                if is_truthy(&cond) {
                    self.execute(then_branch)?;
                } else if let Some(else_stmt) = else_branch {
                    self.execute(else_stmt)?;
                }
                Ok(())
            }
            Stmt::Evolve { body, condition, check_first } => {
                if *check_first  {
                    loop { 
                        let cond = self.evaluate(condition)?;
                        if is_truthy(&cond) {
                            break;
                        } else {
                            self.execute(body)?;
                        }
                    }
                } else {
                    loop {
                        self.execute(body)?;
                        let cond = self.evaluate(condition)?;
                        if is_truthy(&cond) {
                            break;
                        }
                    }
                }
                Ok(())
            }
            Stmt::Return { keyword, value } => {
                // decide what the function hands back
                let result = if let Some(expr) = value {
                    self.evaluate(expr)?        // return expr -> evaluate it
                } else {
                    Value::Nil                  // bare return -> nil
                };

                Err(Interrupt::Return { value: result, line: keyword.line })
            }
            Stmt::Function(decl) => {
                // builds function from declaration + CURRENT environment (interpreter's)
                let function = UserFunction::new(Rc::clone(decl), Rc::clone(&self.context));
                // wrap it so it counts as a value
                let value   = Value::Callable(Rc::new(function));
                // store it in current environment under the law's name
                self.context.borrow_mut().define(&decl.name, value)?;
                Ok(())
            }
        }
    }

    // installs `inner`, runs the statements, then always restores the previous scope
    // in 5PR3: made it reachable from other files (pub(crate))
    pub(crate) fn execute_block(&mut self, stmts: &[Stmt], inner: ContextRef) -> Result<(), Interrupt> {
        let previous = std::mem::replace(&mut self.context, inner);
        let result = stmts.iter().try_for_each(|s| self.execute(s));
        self.context = previous;
        result
    }

    // runs one statement for main.rs: turns an Interrupt back to a plain RuntimeError
    pub fn run(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match self.execute(stmt) {
            Ok(()) => Ok(()),
            Err(Interrupt::Runtime(e)) => Err(e),
            Err(Interrupt::Return { line, .. }) => Err(RuntimeError { line, message: "Can't return from top-level code.".to_string()})
        }
    }
}