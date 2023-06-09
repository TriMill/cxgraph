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

/////////////////
//  constants  //
/////////////////

const TAU = 6.283185307179586;
const E = 2.718281828459045;
const RECIP_SQRT2 = 0.7071067811865475;

const C_TAU = vec2f(TAU, 0.0);
const C_E = vec2f(E, 0.0);
const C_I = vec2f(0.0, 1.0);

/////////////////////////
//  complex functions  //
/////////////////////////

fn c_re(z: vec2f) -> vec2f {
	return vec2(z.x, 0.0);
}

fn c_im(z: vec2f) -> vec2f {
	return vec2(z.y, 0.0);
}

fn c_conj(z: vec2f) -> vec2f {
	return z * vec2(1.0, -1.0);
}

fn c_abs_sq(z: vec2f) -> vec2f {
	return vec2(dot(z, z), 0.0);
}

fn c_abs(z: vec2f) -> vec2f {
	return vec2(length(z), 0.0);
}

fn c_arg(z: vec2f) -> vec2f {
	return vec2(atan2(z.y, z.x), 0.0);
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
	return vec2(0.5 * log(dot(z, z)), atan2(z.y, z.x));
}

fn c_pow(u: vec2f, v: vec2f) -> vec2f {
	return c_exp(c_mul(c_log(u), v));
}

fn c_sqrt(z: vec2f) -> vec2f {
	return c_pow(z, vec2(0.5, 0.0));
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
	let u = c_sqrt(vec2f(1.0, 0.0) - c_mul(z, z));
	let v = c_log(u + vec2(-z.y, z.x));
	return vec2(v.y, -v.x);
}

fn c_acos(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2f(1.0, 0.0) - c_mul(z, z));
	let v = c_log(u + vec2f(-z.y, z.x));
	return vec2f(TAU*0.25 - v.y, v.x);
}

fn c_atan(z: vec2f) -> vec2f {
	let u = vec2f(1.0, 0.0) - vec2f(-z.y, z.x);
	let v = vec2f(1.0, 0.0) + vec2f(-z.y, z.x);
	let w = c_log(c_div(u, v));
	return 0.5 * vec2f(-w.y, w.x);
}

fn c_asinh(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2f(1.0, 0.0) + c_mul(z, z));
	return c_log(u + z);
}

fn c_acosh(z: vec2f) -> vec2f {
	let u = c_sqrt(vec2f(-1.0, 0.0) + c_mul(z, z));
	return c_log(u + z);
}

fn c_atanh(z: vec2f) -> vec2f {
	let u = vec2f(1.0, 0.0) + z;
	let v = vec2f(1.0, 0.0) - z;
	return 0.5 * c_log(c_div(u, v));
}

fn c_gamma(z: vec2f) -> vec2f {
	let reflect = z.x < 0.5;
	var zp = z;
	if reflect {
		zp = vec2(1.0, 0.0) - z;
	}
	var w = c_gamma_inner2(zp);
	if reflect {
		w = TAU * 0.5 * c_recip(c_mul(c_sin(TAU * 0.5 * z), w));
	}
	return w;
}

// Yang, ZH., Tian, JF. An accurate approximation formula for gamma function. J Inequal Appl 2018, 56 (2018).
// https://doi.org/10.1186/s13660-018-1646-6
fn c_gamma_inner(z: vec2f) -> vec2f {
	let z2 = c_mul(z, z);
	let z3 = c_mul(z2, z);

	let a = c_sqrt(TAU * z);
	let b = c_pow(1.0 / (E * E) * c_mul(z3, c_sinh(c_recip(z))), 0.5 * z);
	let c = c_exp(7.0/324.0 * c_recip(c_mul(z3, 35.0 * z2 + 33.0)));

	return c_mul(c_mul(a, b), c);
}

fn c_gamma_inner2(z: vec2f) -> vec2f {
	let w = c_gamma_inner(z + vec2(3.0, 0.0));
	return c_div(w, c_mul(c_mul(z, z + vec2(1.0, 0.0)), c_mul(z + vec2(2.0, 0.0), z + vec2(3.0, 0.0))));
}

/////////////////
//  rendering  //
/////////////////

fn hsv2rgb(c: vec3f) -> vec3f {
    let p = abs(fract(c.xxx + vec3f(1.0, 2.0/3.0, 1.0/3.0)) * 6.0 - vec3f(3.0));
    return c.z * mix(vec3f(1.0), clamp(p - vec3f(1.0), vec3f(0.0), vec3f(1.0)), c.y);
}

fn shademap(r: f32) -> f32 {
	let i = uniforms.shading_intensity * uniforms.shading_intensity * uniforms.shading_intensity;
    return r*inverseSqrt(r * r + 0.0625 * i);
}

fn coloring_standard(z: vec2f) -> vec3f {
	if z.x != z.x || z.y != z.y {
		return vec3f(0.5, 0.5, 0.5);
	}

	let r = length(z);
	let arg = atan2(z.y, z.x);
	let hsv = vec3f(arg / TAU + 1.0, shademap(1.0/r), shademap(r));
	return hsv2rgb(hsv);
}

fn coloring_uniform(z: vec2f) -> vec3f {
	if z.x != z.x || z.y != z.y {
		return vec3f(0.5, 0.5, 0.5);
	}

	let arg = atan2(z.y, z.x);
	let mag = length(z);

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
	let arg = atan2(z.y, z.x);
	return round(correct_mod(arg + TAU, TAU/8.0) * 8.0/TAU) * 2.0 - 1.0;
}

fn decoration_contour_mag(z: vec2f) -> f32 {
	let logmag = 0.5 * log2(z.x*z.x + z.y*z.y);
	return round(correct_mod(0.5 * logmag, 1.0)) * 2.0 - 1.0;
}

@fragment
fn main(@builtin(position) in: vec4f) -> @location(0) vec4f {
	let pos = vec2(in.x, f32(uniforms.resolution.y) - in.y);
	let w = remap(pos, vec2(0.0), vec2f(uniforms.resolution), uniforms.bounds_min, uniforms.bounds_max);

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

	let final_col = mix(col, vec3f(contours * 0.5 + 0.5), uniforms.contour_intensity);

	return vec4f(pow(final_col, vec3(1.68)), 1.0);
}
