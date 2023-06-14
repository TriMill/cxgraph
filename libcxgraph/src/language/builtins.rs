use std::collections::HashMap;

use num_complex::Complex64 as Complex;

thread_local! {
	pub static BUILTIN_FUNCS: HashMap<&'static str, (&'static str, usize)> = {
		let mut m = HashMap::new();
		m.insert("pos",    ("c_pos",    1));
		m.insert("neg",    ("c_neg",    1));
		m.insert("recip",  ("c_recip",  1));
		m.insert("conj",   ("c_conj",   1));

		m.insert("re",       ("c_re",     1));
		m.insert("im",       ("c_im",     1));
		m.insert("signre",   ("c_signre", 1));
		m.insert("signim",   ("c_signim", 1));
		m.insert("abs_sq",   ("c_abs_sq", 1));
		m.insert("abs",      ("c_abs",    1));
		m.insert("arg",      ("c_arg",    1));
		m.insert("argbr",    ("c_argbr",  2));

		m.insert("add",      ("c_add",    2));
		m.insert("sub",      ("c_sub",    2));
		m.insert("mul",      ("c_mul",    2));
		m.insert("div",      ("c_div",    2));
		m.insert("pow",      ("c_pow",    2));

		m.insert("exp",      ("c_exp",    1));
		m.insert("log",      ("c_log",    1));
		m.insert("logbr",    ("c_logbr",  2));
		m.insert("sqrt",     ("c_sqrt",   1));

		m.insert("sin",      ("c_sin",    1));
		m.insert("cos",      ("c_cos",    1));
		m.insert("tan",      ("c_tan",    1));
		m.insert("sinh",     ("c_sinh",   1));
		m.insert("cosh",     ("c_cosh",   1));
		m.insert("tanh",     ("c_tanh",   1));

		m.insert("asin",     ("c_asin",   1));
		m.insert("acos",     ("c_acos",   1));
		m.insert("atan",     ("c_atan",   1));
		m.insert("asinh",    ("c_asinh",  1));
		m.insert("acosh",    ("c_acosh",  1));
		m.insert("atanh",    ("c_atanh",  1));

		m.insert("gamma",       ("c_gamma",    1));
		m.insert("\u{0393}",    ("c_gamma",    1));
		m.insert("loggamma",    ("c_loggamma", 1));
		m.insert("log\u{0393}", ("c_loggamma", 1));
		m.insert("digamma",     ("c_digamma",  1));
		m.insert("\u{03C8}",    ("c_digamma",  1));

		m
	};

	pub static BUILTIN_CONSTS: HashMap<&'static str, (&'static str, Complex)> = {
		let mut m = HashMap::new();
		m.insert("i",        ("C_I",       Complex::new(0.0, 1.0)));
		m.insert("e",        ("C_E",       Complex::new(std::f64::consts::E, 0.0)));
		m.insert("tau",      ("C_TAU",     Complex::new(std::f64::consts::TAU, 0.0)));
		m.insert("\u{03C4}", ("C_TAU",     Complex::new(std::f64::consts::TAU, 0.0)));
		m.insert("emgamma",  ("C_EMGAMMA", Complex::new(0.5772156649015329, 0.0)));
		m.insert("\u{03B3}", ("C_EMGAMMA", Complex::new(0.5772156649015329, 0.0)));
		m.insert("phi",      ("C_PHI",     Complex::new(1.618033988749895, 0.0)));
		m.insert("\u{03C6}", ("C_PHI",     Complex::new(1.618033988749895, 0.0)));
		m
	}
}
