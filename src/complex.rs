#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
	pub re: f64,
	pub im: f64
}

impl Complex {
	pub const fn new(re: f64, im: f64) -> Self {
		Self { re, im }
	}
}

impl From<f64> for Complex {
	fn from(value: f64) -> Self {
		Self { re: value, im: 0.0 }
	}
}

pub mod cxfn {
	use super::Complex;

	pub type Function = fn(args: &[Complex]) -> Complex;

	pub fn pos(a: &[Complex]) -> Complex {
		a[0]
	}

	pub fn neg(a: &[Complex]) -> Complex {
		Complex::new(-a[0].re, -a[0].im)
	}

	pub fn recip(a: &[Complex]) -> Complex {
		let a = a[0];
		let d = a.re*a.re + a.im*a.im;
		Complex::new(a.re / d, -a.im / d)
	}

	pub fn re(a: &[Complex]) -> Complex {
		Complex::new(a[0].re, 0.0)
	}

	pub fn im(a: &[Complex]) -> Complex {
		Complex::new(a[0].im, 0.0)
	}

	pub fn conj(a: &[Complex]) -> Complex {
		Complex::new(a[0].re, -a[0].im)
	}

	pub fn abs_sq(a: &[Complex]) -> Complex {
		Complex::new(a[0].re*a[0].re + a[0].im*a[0].im, 0.0)
	}

	pub fn abs(a: &[Complex]) -> Complex {
		Complex::new((a[0].re*a[0].re + a[0].im*a[0].im).sqrt(), 0.0)
	}

	pub fn arg(a: &[Complex]) -> Complex {
		Complex::new(f64::atan2(a[0].im, a[0].re), 0.0)
	}

	pub fn add(a: &[Complex]) -> Complex {
		let (a, b) = (a[0], a[1]);
		Complex::new(a.re + b.re, a.im + b.im)
	}

	pub fn sub(a: &[Complex]) -> Complex {
		let (a, b) = (a[0], a[1]);
		Complex::new(a.re - b.re, a.im - b.im)
	}

	pub fn mul(a: &[Complex]) -> Complex {
		let (a, b) = (a[0], a[1]);
		Complex::new(a.re*b.re - a.im*b.im, a.im*b.re + a.re*b.im)
	}

	pub fn div(a: &[Complex]) -> Complex {
		let (a, b) = (a[0], a[1]);
		let d = b.re*b.re + b.im*b.im;
		Complex::new((a.re*b.re + a.im*b.im)/d, (a.im*b.re - a.re*b.im)/d)
	}

	pub fn exp(a: &[Complex]) -> Complex {
		let e = a[0].re.exp();
		Complex::new(e * a[0].im.cos(), e * a[0].im.sin())
	}

	pub fn log(a: &[Complex]) -> Complex {
		let a = a[0];
		Complex::new(0.5 * (a.re*a.re + a.im*a.im).ln(), f64::atan2(a.im, a.re))
	}

	pub fn pow(a: &[Complex]) -> Complex {
		exp(&[mul(&[log(&[a[0]]), a[1]])])
	}

	pub fn sqrt(a: &[Complex]) -> Complex {
		pow(&[a[0],Complex::new(0.5, 0.0)])
	}

	pub fn sin(a: &[Complex]) -> Complex {
        let a = a[0];
		Complex::new(a.re.sin()*a.im.cosh(), a.re.cos()*a.im.sinh())
	}

	pub fn cos(a: &[Complex]) -> Complex {
        let a = a[0];
		Complex::new(a.re.cos()*a.im.cosh(), -a.re.sin()*a.re.sinh())
	}

	pub fn tan(a: &[Complex]) -> Complex {
        let a = a[0];
        let d = (2.0*a.re).cos() + (2.0*a.im).cosh();
		Complex::new((2.0*a.re).sin() / d, (2.0*a.im).sinh() / d)
	}

	pub fn sinh(a: &[Complex]) -> Complex {
        let a = a[0];
		Complex::new(a.re.sinh()*a.im.cos(), a.re.cosh()*a.re.sin())
	}

	pub fn cosh(a: &[Complex]) -> Complex {
        let a = a[0];
		Complex::new(a.re.cosh()*a.im.cos(), a.re.sinh()*a.re.sin())
	}

	pub fn tanh(a: &[Complex]) -> Complex {
        let a = a[0];
        let d = (2.0*a.re).cosh() + (2.0*a.im).cos();
		Complex::new((2.0*a.re).sinh() / d, (2.0*a.im).sin() / d)
	}
}
