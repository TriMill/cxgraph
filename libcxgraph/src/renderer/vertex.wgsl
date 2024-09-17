@vertex
fn main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4f {
	var pos = array<vec2f, 3>(
		vec2(-1.0,-1.0),
		vec2( 3.0,-1.0),
		vec2(-1.0, 3.0),
	);

	return vec4f(pos[in_vertex_index], 0.0, 1.0);
}
