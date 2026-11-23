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

pub const DEFAULT_SEED: u64 = 0x2545_F491_4F6C_DD1D;

pub static NATIVES: &[NativeFn] = &[
    NativeFn { name: "pow",    arity: 2, func: native_pow },
    NativeFn { name: "seed",   arity: 1, func: native_seed },
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

// scrambles the user's seed so nearby seeds (1, 2, 3) give unrelated starting states
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// seed(n) -> nil
fn native_seed(it: &mut Interpreter, args: &[Value], tok: &Token) -> Result<Value, RuntimeError> {
    let n = num("seed", args, 0, tok)?;
    if !n.is_finite() {
        return Err(RuntimeError::new(tok, "seed: argument must be finite."));
    }
    let state = splitmix64(n.to_bits());
    // xorshift gets stuck forever if its state is 0
    it.rng_state = if state == 0 { DEFAULT_SEED } else { state };
    Ok(Value::Nil)
}