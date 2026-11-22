// ----------------------------------------------------
// callable.rs - defines one trait
// trait: a list of abilities a type promises to have
// ----------------------------------------------------

use crate::evaluator::{Interpreter, RuntimeError, Value};

// anything callable must have the ff:
pub trait Callable {
    fn arity(&self) -> usize;           // how many argument it expects
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>)
        -> Result<Value, RuntimeError>; // a way to run it;  gets the interpreter and
                                        // already evaluated args, returns value or error
    fn describe(&self) -> String;       // a printable name  
}