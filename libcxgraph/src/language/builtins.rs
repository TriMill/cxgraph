use std::{collections::HashMap, ops, f64::consts::{TAU, E}};

use num_complex::Complex64 as Complex;

#[derive(Clone, Copy)]
pub enum Func {
	One(fn(Complex) -> Complex),
	Two(fn(Complex, Complex) -> Complex),
}

impl Func {
	pub fn argc(&self) -> usize {
		match self {
			Func::One(_) => 1,
			Func::Two(_) => 2,
		}
	}
}

fn neg(z: Complex) -> Complex { -z }
fn re(z: Complex) -> Complex { Complex::from(z.re) }
fn im(z: Complex) -> Complex { Complex::from(z.im) }
fn abs_sq(z: Complex) -> Complex { Complex::from(z.norm_sqr()) }
fn abs(z: Complex) -> Complex { Complex::from(z.norm()) }
fn arg(z: Complex) -> Complex { Complex::from(z.arg()) }
fn recip(z: Complex) -> Complex { Complex::new(1.0, 0.0) / z }
fn conj(z: Complex) -> Complex { z.conj() }


fn gamma(z: Complex) -> Complex {
	let reflect = z.re < 0.5;
	let zp = if reflect { 1.0 - z } else { z };
	let mut w = gamma_inner(zp + 3.0) / (zp*(zp+1.0)*(zp+2.0)*(zp+3.0));
	if reflect {
		w = TAU * 0.5 / ((TAU * 0.5 * z).sin() * w);
	}
	return w;
}

// Yang, ZH., Tian, JF. An accurate approximation formula for gamma function. J Inequal Appl 2018, 56 (2018).
// https://doi.org/10.1186/s13660-018-1646-6
fn gamma_inner(z: Complex) -> Complex {
	let z2 = z * z;
	let z3 = z2 * z;

	let a = (TAU * z).sqrt();
	let b = (1.0 / (E * E) * z3 * (1.0/z).sinh()).powc(0.5 * z);
	let c = ((7.0/324.0) / (z3 * (35.0 * z2 + 33.0))).exp();

	return a * b * c;
}

thread_local! {
	pub static BUILTIN_FUNCS: HashMap<&'static str, (&'static str, Func)> = {
		let mut m = HashMap::new();
		m.insert("pos",    ("c_pos",    Func::One(std::convert::identity)));
		m.insert("neg",    ("c_neg",    Func::One(neg)));
		m.insert("recip",  ("c_recip",  Func::One(recip)));
		m.insert("conj",   ("c_conj",   Func::One(conj)));

		m.insert("re",       ("c_re",     Func::One(re)));
		m.insert("im",       ("c_im",     Func::One(im)));
		m.insert("abs_sq",   ("c_abs_sq", Func::One(abs_sq)));
		m.insert("abs",      ("c_abs",    Func::One(abs)));
		m.insert("arg",      ("c_arg",    Func::One(arg)));

		m.insert("add",      ("c_add",    Func::Two(<Complex as ops::Add>::add)));
		m.insert("sub",      ("c_sub",    Func::Two(<Complex as ops::Sub>::sub)));
		m.insert("mul",      ("c_mul",    Func::Two(<Complex as ops::Mul>::mul)));
		m.insert("div",      ("c_div",    Func::Two(<Complex as ops::Div>::div)));
		m.insert("pow",      ("c_pow",    Func::Two(Complex::powc)));

		m.insert("exp",      ("c_exp",    Func::One(Complex::exp)));
		m.insert("log",      ("c_log",    Func::One(Complex::ln)));
		m.insert("sqrt",     ("c_sqrt",   Func::One(Complex::sqrt)));

		m.insert("sin",      ("c_sin",    Func::One(Complex::sin)));
		m.insert("cos",      ("c_cos",    Func::One(Complex::cos)));
		m.insert("tan",      ("c_tan",    Func::One(Complex::tan)));
		m.insert("sinh",     ("c_sinh",   Func::One(Complex::sinh)));
		m.insert("cosh",     ("c_cosh",   Func::One(Complex::cosh)));
		m.insert("tanh",     ("c_tanh",   Func::One(Complex::tanh)));

		m.insert("asin",     ("c_asin",   Func::One(Complex::asin)));
		m.insert("acos",     ("c_acos",   Func::One(Complex::acos)));
		m.insert("atan",     ("c_atan",   Func::One(Complex::atan)));
		m.insert("asinh",    ("c_asinh",  Func::One(Complex::asinh)));
		m.insert("acosh",    ("c_acosh",  Func::One(Complex::acosh)));
		m.insert("atanh",    ("c_atanh",  Func::One(Complex::atanh)));

		m.insert("gamma",    ("c_gamma",  Func::One(gamma)));
		m.insert("\u{0393}", ("c_gamma",  Func::One(gamma)));

		m
	};

	pub static BUILTIN_CONSTS: HashMap<&'static str, (&'static str, Complex)> = {
		let mut m = HashMap::new();
		m.insert("tau",      ("C_TAU", Complex::new(std::f64::consts::TAU, 0.0)));
		m.insert("\u{03C4}", ("C_TAU", Complex::new(std::f64::consts::TAU, 0.0)));
		m.insert("e",        ("C_E",   Complex::new(std::f64::consts::E, 0.0)));
		m.insert("i",        ("C_I",   Complex::new(0.0, 1.0)));
		m
	}
}
