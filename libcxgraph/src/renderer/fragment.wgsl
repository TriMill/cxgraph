////////////////
//  uniforms  //
////////////////

struct Uniforms {
	variables: array<vec4f, 4>,
	resolution: vec2u,
	bounds_min: vec2f,
	bounds_max: vec2f,
	shading_intensity: f32,
	contour_intensity: f32,
	decoration: u32,
	coloring: u32,
}

@group(0) @binding(1) var<uniform> uniforms: Uniforms;

///////////////
//  utility  //
///////////////

fn remap(val: vec2f, a1: vec2f, b1: vec2f, a2: vec2f, b2: vec2f) -> vec2f {
	return a2 + (b2 - a2) * ((val - a1) / (b1 - a1));
}

fn correct_mod(x: f32, y: f32) -> f32 {
	return ((x % y) + y) % y;
}

fn correct_mod2(x: vec2f, y: vec2f) -> vec2f {
	return ((x % y) + y) % y;
}

fn vlength(v: vec2f) -> f32 {
	let l = length(v);
	if l != l || l <= 3.4e+38 {
		return l;
	}
	let a = max(abs(v.x), abs(v.y));
	let b = RECIP_SQRT2 * abs(v.x) + RECIP_SQRT2 * abs(v.y);
	let c = 2.0 * RECIP_SQRT29 * abs(v.x) + 5.0 * RECIP_SQRT29 * abs(v.y);
	let d = 5.0 * RECIP_SQRT29 * abs(v.x) + 2.0 * RECIP_SQRT29 * abs(v.y);
	return max(max(a, b), max(c, d));
}

/////////////////
//  constants  //
/////////////////

const TAU = 6.283185307179586;
const E = 2.718281828459045;
const RECIP_SQRT2 = 0.7071067811865475;
const LOG_TAU = 1.8378770664093453;
const LOG_2 = 0.6931471805599453;
const RECIP_SQRT29 = 0.18569533817705186;

const C_TAU = vec2f(TAU, 0.0);
const C_E = vec2f(E, 0.0);
const C_I = vec2f(0.0, 1.0);
const C_EMGAMMA = vec2f(0.5772156649015329, 0.0);
const C_PHI = vec2f(1.618033988749895, 0.0);

/////////////////////////
//  complex functions  //
/////////////////////////

fn c_re(z: vec2f) -> vec2f {
	return vec2(z.x, 0.0);
}

fn c_im(z: vec2f) -> vec2f {
	return vec2(z.y, 0.0);
}

fn c_signre(z: vec2f) -> vec2f {
	return vec2(sign(z.x), 0.0);
}

fn c_signim(z: vec2f) -> vec2f {
	return vec2(sign(z.y), 0.0);
}

fn c_ifgt(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x > q.x);
}

fn c_iflt(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x < q.x);
}

fn c_ifge(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x >= q.x);
}

fn c_ifle(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x <= q.x);
}

fn c_ifeq(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x == q.x);
}

fn c_ifne(p: vec2f, q: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x != q.x);
}

fn c_ifnan(p: vec2f, z: vec2f, w: vec2f) -> vec2f {
	return select(w, z, p.x != p.x && p.y != p.y);
}

fn c_conj(z: vec2f) -> vec2f {
	return z * vec2(1.0, -1.0);
}

fn c_abs_sq(z: vec2f) -> vec2f {
	return vec2(dot(z, z), 0.0);
}

fn c_abs(z: vec2f) -> vec2f {
	return vec2(vlength(z), 0.0);
}

fn c_arg(z: vec2f) -> vec2f {
	if z.x < 0.0 && z.y == 0.0 {
		return vec2(TAU/2.0, 0.0);
	}
	return vec2(atan2(z.y, z.x), 0.0);
}

fn c_argbr(z: vec2f, br: vec2f) -> vec2f {
	let r = vec2(cos(-br.x), sin(-br.x));
	let zr = c_mul(z, r);
	return c_arg(zr) + vec2(br.x, 0.0);
}

fn c_add(u: vec2f, v: vec2f) -> vec2f {
	return u + v;
}

fn c_sub(u: vec2f, v: vec2f) -> vec2f {
	return u - v;
}

fn c_mul(u: vec2f, v: vec2f) -> vec2f {
	return vec2(u.x*v.x - u.y*v.y, u.y*v.x + u.x*v.y);
}

fn c_div(u: vec2f, v: vec2f) -> vec2f {
	return vec2(u.x*v.x + u.y*v.y, u.y*v.x - u.x*v.y) / dot(v, v);
}

fn c_pos(v: vec2f) -> vec2f {
	return v;
}

fn c_neg(v: vec2f) -> vec2f {
	return -v;
}

fn c_recip(v: vec2f) -> vec2f {
	return vec2(v.x, -v.y) / dot(v, v);
}

fn c_exp(z: vec2f) -> vec2f {
	return exp(z.x) * vec2(cos(z.y), sin(z.y));
}

fn c_log(z: vec2f) -> vec2f {
	return vec2(0.5 * log(dot(z, z)), c_arg(z).x);
}

fn c_logbr(z: vec2f, br: vec2f) -> vec2f {
	return vec2(0.5 * log(dot(z, z)), c_argbr(z, br).x);
}

fn c_pow(u: vec2f, v: vec2f) -> vec2f {
	return c_exp(c_mul(c_log(u), v));
}

fn c_powbr(u: vec2f, v: vec2f, br: vec2f) -> vec2f {
	return c_exp(c_mul(c_logbr(u, br), v));
}

fn c_sqrt(z: vec2f) -> vec2f {
	return c_pow(z, vec2(0.5, 0.0));
}

fn c_sqrtbr(z: vec2f, br: vec2f) -> vec2f {
	return c_powbr(z, vec2(0.5, 0.0), br);
}

fn c_cbrt(z: vec2f, br: vec2f) -> vec2f {
	return c_pow(z, vec2(1.0/3.0, 0.0));
}

fn c_cbrtbr(z: vec2f, br: vec2f) -> vec2f {
	return c_powbr(z, vec2(1.0/3.0, 0.0), br);
}

fn c_sin(z: vec2f) -> vec2f {
	return vec2(sin(z.x)*cosh(z.y), cos(z.x)*sinh(z.y));
}

fn c_cos(z: vec2f) -> vec2f {
	return vec2(cos(z.x)*cosh(z.y), -sin(z.x)*sinh(z.y));
}

fn c_tan(z: vec2f) -> vec2f {
	return vec2(sin(2.0*z.x), sinh(2.0*z.y)) / (cos(2.0*z.x) + cosh(2.0*z.y));
}

fn c_sinh(z: vec2f) -> vec2f {
	return vec2(sinh(z.x)*cos(z.y), cosh(z.x)*sin(z.y));
}

fn c_cosh(z: vec2f) -> vec2f {
	return vec2(cosh(z.x)*cos(z.y), sinh(z.x)*sin(z.y));
}

fn c_tanh(z: vec2f) -> vec2f {
	return vec2(sinh(2.0*z.x), sin(2.0*z.y)) / (cosh(2.0*z.x) + cos(2.0*z.y));
}

fn c_asin(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2(1.0, 0.0) - c_mul(z, z));
	let v = c_log(u + vec2(-z.y, z.x));
	return vec2(v.y, -v.x);
}

fn c_acos(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2(1.0, 0.0) - c_mul(z, z));
	let v = c_log(u + vec2(-z.y, z.x));
	return vec2(TAU*0.25 - v.y, v.x);
}

fn c_atan(z: vec2f) -> vec2f {
	let u = vec2(1.0, 0.0) - vec2(-z.y, z.x);
	let v = vec2(1.0, 0.0) + vec2(-z.y, z.x);
	let w = c_log(c_div(u, v));
	return 0.5 * vec2(-w.y, w.x);
}

fn c_asinh(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2(1.0, 0.0) + c_mul(z, z));
	return c_log(u + z);
}

fn c_acosh(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2(-1.0, 0.0) + c_mul(z, z));
	return c_log(u + z);
}

fn c_atanh(z: vec2f) -> vec2f {
	let u = vec2(1.0, 0.0) + z;
	let v = vec2(1.0, 0.0) - z;
	return 0.5 * c_log(c_div(u, v));
}

// log gamma //

fn c_loggamma(z: vec2f) -> vec2f {
	let reflect = z.x < 0.5 && abs(z.y) < 13.0;
	var zp = z;
	if reflect {
		zp = vec2(1.0, 0.0) - z;
	}
	var w = c_loggamma_inner2(zp);
	if reflect {
		let br = 0.5 * TAU * (0.5 - z.x) * sign(z.y);
		w = vec2(LOG_TAU - LOG_2, 0.0) - c_logbr(c_sin(TAU/2.0 * z), vec2(br, 0.0)) - w;
	}
	return w;
}

fn c_loggamma_inner(z: vec2f) -> vec2f {
	return c_mul(z - vec2(0.5, 0.0), c_log(z)) - z + vec2(0.5*LOG_TAU, 0.0) + c_recip(12.0 * z);
}

fn c_loggamma_inner2(z: vec2f) -> vec2f {
	let w = c_loggamma_inner(z + vec2(3.0, 0.0));
	let l = c_log(z) + c_log(z + vec2(1.0, 0.0)) + c_log(z + vec2(2.0, 0.0));
	return w - l;
}

// gamma //

fn c_gamma(z: vec2f) -> vec2f {
	return c_exp(c_loggamma(z));
}

fn c_invgamma(z: vec2f) -> vec2f {
	return c_exp(-c_loggamma(z));
}

// digamma //

fn c_digamma(z: vec2f) -> vec2f {
	let reflect = z.x < 0.5 && abs(z.y) < 13.0;
	var zp = z;
	if reflect {
		zp = vec2(1.0, 0.0) - z;
	}
	var w = c_digamma_inner2(zp);
	if reflect {
		w -= TAU / 2.0 * c_recip(c_tan(TAU / 2.0 * z));
	}
	return w;
}

fn c_digamma_inner(z: vec2f) -> vec2f {
	let zr = c_recip(z);
	let zr2 = c_mul(zr, zr);
	let zr4 = c_mul(zr2, zr2);
	let zr6 = c_mul(zr2, zr4);
	return c_log(z) - 0.5*zr - (1.0/12.0)*zr2 + (1.0/120.0)*zr4 - (1.0/252.0)*zr6;
}

fn c_digamma_inner2(z: vec2f) -> vec2f {
	let w = c_digamma_inner(z + vec2(3.0, 0.0));
	let l = c_recip(z + vec2(2.0, 0.0)) + c_recip(z + vec2(1.0, 0.0)) + c_recip(z);
	return w - l;
}

/////////////////
//  rendering  //
/////////////////

fn hsv2rgb(c: vec3f) -> vec3f {
	let p = abs(fract(c.xxx + vec3f(1.0, 2.0/3.0, 1.0/3.0)) * 6.0 - vec3f(3.0));
	return c.z * mix(vec3f(1.0), clamp(p - vec3f(1.0), vec3f(0.0), vec3f(1.0)), c.y);
}

fn shademap(r: f32) -> f32 {
	if uniforms.shading_intensity == 0.0 {
		return 1.0;
	} else {
		let i = uniforms.shading_intensity * uniforms.shading_intensity * uniforms.shading_intensity;
		return r*inverseSqrt(r * r + 0.0625 * i);
	}
}

fn coloring_standard(z: vec2f) -> vec3f {
	if z.x != z.x || z.y != z.y {
		return vec3f(0.5, 0.5, 0.5);
	}

	let mag = vlength(z);
	if mag > 3.40282347E+38 {
		return vec3f(1.0, 1.0, 1.0);
	}
	if uniforms.shading_intensity > 0.0 && mag > 1.8446E+19 {
		return vec3f(1.0, 1.0, 1.0);
	}
	if uniforms.shading_intensity == 0.0 && mag < 1.0E-38 {
		return vec3f(0.0, 0.0, 0.0);
	}

	let arg = c_arg(z).x;

	let hsv = vec3f(arg / TAU + 1.0, shademap(1.0/mag), shademap(mag));
	return hsv2rgb(hsv);
}

fn coloring_uniform(z: vec2f) -> vec3f {
	if z.x == 0.0 && z.y == 0.0 {
		return vec3f(0.0, 0.0, 0.0);
	}
	if z.x != z.x || z.y != z.y {
		return vec3f(0.5, 0.5, 0.5);
	}

	let mag = vlength(z);
	if mag > 3.40282347E+38 {
		return vec3f(1.0, 1.0, 1.0);
	}
	if uniforms.shading_intensity > 0.0 && mag > 1.8446E+19 {
		return vec3f(1.0, 1.0, 1.0);
	}
	if uniforms.shading_intensity == 0.0 && mag < 1.0E-38 {
		return vec3f(0.0, 0.0, 0.0);
	}

	let arg = c_arg(z).x;

	let r = cos(arg - 0.0*TAU/3.0)*0.5 + 0.5;
	let g = cos(arg - 1.0*TAU/3.0)*0.5 + 0.5;
	let b = cos(arg - 2.0*TAU/3.0)*0.5 + 0.5;
	let hue = vec3(r, g, b);
	let s = 1.0 - shademap(1.0/mag);
	let v = 1.0 - shademap(mag);
	return hue * (1.0 - s - v) + s;
}

fn coloring_none(z: vec2f) -> vec3f {
	return vec3f(0.5, 0.5, 0.5);
}

fn decoration_contour_re(z: vec2f) -> f32 {
	return correct_mod(floor(z.x), 2.0) * 2.0 - 1.0;
}

fn decoration_contour_im(z: vec2f) -> f32 {
	return correct_mod(floor(z.y), 2.0) * 2.0 - 1.0;
}

fn decoration_contour_arg(z: vec2f) -> f32 {
	let arg = c_arg(z).x;
	return round(correct_mod(arg + TAU, TAU/8.0) * 8.0/TAU) * 2.0 - 1.0;
}

fn decoration_contour_mag(z: vec2f) -> f32 {
	let logmag = 0.5 * log2(z.x*z.x + z.y*z.y);
	return round(correct_mod(0.5 * logmag, 1.0)) * 2.0 - 1.0;
}

@fragment
fn main(@builtin(position) in: vec4f) -> @location(0) vec4f {
	let pos = vec2(in.x, f32(uniforms.resolution.y) - in.y);
	let w = remap(pos, vec2(0.0, 0.0), vec2f(uniforms.resolution), uniforms.bounds_min, uniforms.bounds_max);

	let z = func_plot(w);

	var col = vec3f();
	switch uniforms.coloring {
		case 0u, default: {
			col = coloring_standard(z);
		}
		case 1u: {
			col = coloring_uniform(z);
		}
		case 2u: {
			col = coloring_none(z);
		}
	}

	var contours = 1.0;

	if (uniforms.decoration & 0x01u) != 0u {
		contours *= decoration_contour_re(z);
	}

	if (uniforms.decoration & 0x02u) != 0u {
		contours *= decoration_contour_im(z);
	}

	if (uniforms.decoration & 0x04u) != 0u {
		contours *= decoration_contour_arg(z);
	}

	if (uniforms.decoration & 0x08u) != 0u {
		contours *= decoration_contour_mag(z);
	}

	if(contours != contours) {
		contours = 0.0;
	}

	let final_col = mix(col, vec3f(contours * 0.5 + 0.5), uniforms.contour_intensity);

	return vec4f(pow(final_col, vec3(1.68)), 1.0);
}
