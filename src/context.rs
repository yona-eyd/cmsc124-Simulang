// ---------------------------------------------------------------------------
// context.rs - interpreter's memory, defines the environment
// environment: data structure that stores the languge's variables and 
//              decides which one a name refers to
// ---------------------------------------------------------------------------

use crate::evaluator::{RuntimeError, Value};
use crate::tokens::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type ContextRef = Rc<RefCell<Context>>;

// one scope: its own bindings plus a link to the scope it is written inside.
pub struct Context {
    values: HashMap<String, Value>,
    enclosing: Option<ContextRef>,
}

fn undefined(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}

impl Context {
    pub fn new_global() -> ContextRef {
        Rc::new(RefCell::new(Context { values: HashMap::new(), enclosing: None }))
    }

    pub fn new_enclosed(outer: &ContextRef) -> ContextRef {
        Rc::new(RefCell::new(Context {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(outer)),
        }))
    }

    // declares in this scope only and never walks outward.
    // redeclaring in the same scope is an error; an inner scope may shadow.
    pub fn define(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            return Err(RuntimeError::new(
                name,
                &format!("Variable '{}' already declared in this scope.", name.lexeme),
            ));
        }
        self.values.insert(name.lexeme.clone(), value);
        Ok(())
    }

    // walks outward; the innermost binding wins.
    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(v) = self.values.get(&name.lexeme) {
            return Ok(v.clone());
        }
        match &self.enclosing {
            Some(outer) => outer.borrow().get(name),
            None => Err(undefined(name)),
        }
    }

    // walks outward to an existing binding and never creates one.
    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if let Some(slot) = self.values.get_mut(&name.lexeme) {
            *slot = value;
            return Ok(());
        }
        match &self.enclosing {
            Some(outer) => outer.borrow_mut().assign(name, value),
            None => Err(undefined(name)),
        }
    }
}