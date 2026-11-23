use crate::evaluator::{Interpreter, RuntimeError, Value};
use crate::tokens::Token;

#[derive(Debug)]
pub struct NativeFn {
    pub name: &'static str,
    pub arity: usize,
    pub func: fn(&mut Interpreter, &[Value], &Token) -> Result<Value, RuntimeError>,
}

// two natives are equal only if they are the same registered entry
impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self, other) }
}

pub static NATIVES: &[NativeFn] = &[
    NativeFn { name: "pow",    arity: 2, func: native_pow },
];

// the error cites the call's ")" token so the line number is the call's line
fn num(name: &str, args: &[Value], i: usize, tok: &Token) -> Result<f64, RuntimeError> {
    match &args[i] {
        Value::Num(n) => Ok(*n),
        _ => Err(RuntimeError::new(
            tok,
            &format!("{name}: argument {} must be a number.", i + 1),
        )),
    }
}

// pow(base, exp) -> number
fn native_pow(_: &mut Interpreter, args: &[Value], tok: &Token) -> Result<Value, RuntimeError> {
    let base = num("pow", args, 0, tok)?;
    let exp = num("pow", args, 1, tok)?;
    let result = base.powf(exp);
    if !result.is_finite() {
        return Err(RuntimeError::new(tok, "pow: result is not a finite number."));
    }
    Ok(Value::Num(result))
}