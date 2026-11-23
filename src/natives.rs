use crate::callable::Callable;
use crate::evaluator::{Interpreter, RuntimeError, Value};

//implements Callable, so calls, arity checks and first-class use go through the same path as laws.
#[derive(Debug, Clone, Copy)]
pub struct NativeFn {
    pub name: &'static str,
    pub arity: usize,
    pub func: fn(&mut Interpreter, &[Value]) -> Result<Value, String>,
}

impl Callable for NativeFn {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        // natives don't know the call's line, so they report line 0;
        (self.func)(interpreter, &args)
            .map_err(|message| RuntimeError { line: 0, message })
    }

    fn describe(&self) -> String {
        format!("<native {}>", self.name)
    }
}

pub const DEFAULT_SEED: u64 = 0x2545_F491_4F6C_DD1D;

pub static NATIVES: &[NativeFn] = &[
    NativeFn { name: "pow",    arity: 2, func: native_pow },
    NativeFn { name: "bound",  arity: 3, func: native_bound },
    NativeFn { name: "seed",   arity: 1, func: native_seed },
    NativeFn { name: "random", arity: 0, func: native_random },
];

fn num(name: &str, args: &[Value], i: usize) -> Result<f64, String> {
    match &args[i] {
        Value::Num(n) => Ok(*n),
        _ => Err(format!("{name}: argument {} must be a number.", i + 1)),
    }
}

// pow(base, exp) -> number
fn native_pow(_: &mut Interpreter, args: &[Value]) -> Result<Value, String> {
    let base = num("pow", args, 0)?;
    let exp = num("pow", args, 1)?;
    let result = base.powf(exp);
    if !result.is_finite() {
        return Err("pow: result is not a finite number.".to_string());
    }
    Ok(Value::Num(result))
}

// bound(x, lo, hi) -> number in [lo, hi]
fn native_bound(_: &mut Interpreter, args: &[Value]) -> Result<Value, String> {
    let x = num("bound", args, 0)?;
    let lo = num("bound", args, 1)?;
    let hi = num("bound", args, 2)?;
    if lo > hi {
        return Err("bound: lo must not be greater than hi.".to_string());
    }
    Ok(Value::Num(if x < lo { lo } else if x > hi { hi } else { x }))
}

//scrambles the user's seed so nearby seeds (1, 2, 3) give unrelated starting states
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// seed(n) -> nil
fn native_seed(it: &mut Interpreter, args: &[Value]) -> Result<Value, String> {
    let n = num("seed", args, 0)?;
    if !n.is_finite() {
        return Err("seed: argument must be finite.".to_string());
    }
    let state = splitmix64(n.to_bits());
    // xorshift gets stuck forever at state 0
    it.rng_state = if state == 0 { DEFAULT_SEED } else { state };
    Ok(Value::Nil)
}

// random() -> number in [0, 1); same seed gives the same sequence
fn native_random(it: &mut Interpreter, _: &[Value]) -> Result<Value, String> {
    let mut x = it.rng_state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    it.rng_state = x;
    // top 53 bits fill an f64 mantissa exactly, so the result is < 1.0
    Ok(Value::Num((x >> 11) as f64 / (1u64 << 53) as f64))
}