// ----------------------------------------------------
// callable.rs - defines one trait
// trait: a list of abilities a type promises to have
// ----------------------------------------------------

use crate::evaluator::{Interpreter, RuntimeError, Value};
use std::rc::Rc;                            // shared pointer
use crate::ast::FunctionDecl;               // AST node for a law
use crate::context::{Context, ContextRef};  // environments
use crate::evaluator::Interrupt;            // signal enum


// anything callable must have the ff:
pub trait Callable {
    fn arity(&self) -> usize;           // how many argument it expects
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>)
        -> Result<Value, RuntimeError>; // a way to run it;  gets the interpreter and
                                        // already evaluated args, returns value or error
    fn describe(&self) -> String;       // a printable name  
}

// a declared law: its declaration + closure (environment it was written in)
pub struct UserFunction {
    declaration: Rc<FunctionDecl>,  // name, params, body (shared with AST, not copied)
    closure: ContextRef             // where this law was declared
}

impl UserFunction {
    // constructor; used when a law decl is executed
    pub fn new(declaration: Rc<FunctionDecl>, closure: ContextRef) -> Self {
        UserFunction { declaration, closure }
    }
}

impl Callable for UserFunction {
    // arity = how many params the law was declared with
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interp: &mut Interpreter, args: Vec<Value>)
        -> Result<Value, RuntimeError> {
            // fresh scratch environment for this call, parented to closure
            // NOT the caller's environment so it's lexical
            let env = Context::new_enclosed(&self.closure);

            // bind each parameter to its argument inside env
            for (param, arg) in self.declaration.params.iter().zip(args) {
                env.borrow_mut().define(param, arg)?;
            }

            // run the body in that environment
            let outcome = interp.execute_block(&self.declaration.body, env);

            // match outcome and produce thil call's result
            match outcome {
                Ok(()) => Ok(Value::Nil),
                Err(Interrupt::Return {value, ..}) => Ok(value),
                Err(Interrupt::Runtime(e)) => Err(e)
            }
        }

    // printable nam, e.g. <law add>
    fn describe(&self) -> String {
        format!("<law {}>", self.declaration.name.lexeme)
    }
}