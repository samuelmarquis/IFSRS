const PI: f32  = 3.14159265358f;
const TAU: f32 = 6.28318530718f;
const DEGTORAD: f32 = 0.0174532925f;

const MAX_ITERATORS : u32 =	100;
const MAX_PALETTE_COLORS : u32 = 256;
const MAX_PARAMS : u32 = (2 * MAX_ITERATORS);
const MAX_XAOS : u32 = (MAX_ITERATORS * MAX_ITERATORS);

struct camera_params
{
    view_proj_mat: mat4x4<f32>,

    position: vec4<f32>,
    forward: vec4<f32>,
    focus_point: vec4<f32>,

    aperture: f32,
    focus_distance: f32,
    depth_of_field: f32,
    projection_type: i32,
}

struct Iterator {
    color_speed: f32,
    color_index: f32,
    opacity: f32,
    reset_prob: f32,
    reset_alias: i32,
    tf_id: i32,
    real_params_index: i32,
    vec3_params_index: i32,
    shading_mode: i32,//0: default, 1: delta_p
    tf_mix: f32,
    tf_add: f32,
    padding2: i32
}

struct P_State {
    pos: vec4<f32>,
    color_index: f32,
    dummy0: f32,
    iterator_index: i32,
    iteration_depth: i32,
}

struct Settings {
    camera: camera_params,
    fog_effect: f32,
    itnum : u32, //number of iterators
    palettecnt : i32,
    mark_area_in_focus: i32,

    warmup: u32,
    entropy: f32,
    max_filter_radius: i32,
    padding0: i32,

    filter_method: i32,
    filter_param0: f32,
    filter_param1: f32,
    filter_param2: f32,
}

struct Parameters {
    seed: u32,
    width: u32,
    height: u32,
    dispatch_cnt: i32,

    reset_points_state: i32,
    invocation_iters: i32,
    padding_1: u32,
    padding_2: u32,
}


struct RealParam {
    @align(16) val: f32,
}

// we would want this to be coherent, but wgsl doesn't know what that means yet

@group(0) @binding(0) var<storage, read_write> histogram: array<vec4<f32>>;

@group(0) @binding(1) var<storage, read_write> state: array<P_State>;

@group(0) @binding(2) var<uniform> settings: Settings; // filled from cpu

@group(0) @binding(3) var<uniform> iterators: array<Iterator, MAX_ITERATORS>;  // filled from cpu

@group(0) @binding(4) var<uniform> alias_tables: array<vec4<f32>, MAX_ITERATORS>; // filled from cpu

@group(0) @binding(5) var<uniform> palette: array<vec4<f32>, MAX_PALETTE_COLORS>; // filled from cpu

@group(0) @binding(6) var<uniform> real_params: array<RealParam, MAX_PARAMS>; // filled from cpu

@group(0) @binding(7) var<uniform> vec3_params: array<vec4<f32>, MAX_PARAMS>; // filled from cpu

@group(0) @binding(8) var<uniform> parameters: Parameters; // filled from cpu

@group(0) @binding(9) var<storage, read_write> next_sample: u32;

fn check_float_bits(f: f32) -> i32 { //dumb func to check inf/nan
    let bits = bitcast<u32>(f);
    let exponent = (bits >> 23) & 0xFF;
    //let sign = (bits >> 31) & 0x1;
    let mantissa = bits & 0x7FFFFF;

    if (exponent == 0xFF) {
        if (mantissa != 0) {
            return -1; // NaN
        } else {
            return -1; // Inf
        }
    }
    return 0; // Normal
}

fn rotmat(v: vec3<f32>, arad: f32) -> mat3x3<f32>{
    let c : f32 = cos(arad);
    let s : f32 = sin(arad);
    return mat3x3(
        c + (1.0 - c) * v.x * v.x, (1.0 - c) * v.x * v.y - s * v.z, (1.0 - c) * v.x * v.z + s * v.y,
        (1.0 - c) * v.x * v.y + s * v.z, c + (1.0 - c) * v.y * v.y, (1.0 - c) * v.y * v.z - s * v.x,
        (1.0 - c) * v.x * v.z - s * v.y, (1.0 - c) * v.y * v.z + s * v.x, c + (1.0 - c) * v.z * v.z
    );
}

fn rotate_euler(euler_angles : vec3<f32>) -> mat3x3<f32> {
    return rotmat(vec3(1.0, 0.0, 0.0), euler_angles.x)
         * rotmat(vec3(0.0, 1.0, 0.0), euler_angles.y)
         * rotmat(vec3(0.0, 0.0, 1.0), euler_angles.z);
}


fn pcg_hash(x: u32) -> u32 {
	let state = x * 747796405u + 2891336453u;
	let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
	return (word >> 22u) ^ word;
}

fn pcg_hash_2d(v_param : vec2<u32>) -> vec2<u32>{
    var v = v_param;

    v = v * 1664525u + 1013904223u;

    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;

    v ^= v >> vec2(16u);

    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;

    v ^= v >> vec2(16u);

    return v;
}

fn pcg_hash_3d(v_param : vec3<u32>) -> vec3<u32> {
    var v = v_param;

    v = v * 1664525u + 1013904223u;

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    v ^= v >> vec3(16u);

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    return v;
}


fn pcg_hash_4d(v_param: vec4<u32>) -> vec4<u32> {
    var v = v_param;
    v = v * 1664525u + 1013904223u;

    v.x += v.y*v.w;
    v.y += v.z*v.x;
    v.z += v.x*v.y;
    v.w += v.y*v.z;

    v ^= v >> vec4(16u);

    v.x += v.y*v.w;
    v.y += v.z*v.x;
    v.z += v.x*v.y;
    v.w += v.y*v.z;

    return v;
}

fn xorshift128(v_param : vec4<u32>) -> u32 {
    var v = v_param;
    v.w ^= v.w << 11u;
    v.w ^= v.w >> 8u;
    v = v.wxyz;
    v.x ^= v.y;
    v.x ^= v.y >> 19u;
    return v.x;
}

fn f_hash(h_param: u32) -> f32 {
    var h = h_param;
	h &= 0x007FFFFFu;//mantissa mask
	h |= 0x3F800000u;//one mask
	let r2 = bitcast<f32>(h);
	return r2 - 1.0;
}

fn f_hash1(f : f32) -> f32 {
	return f_hash(pcg_hash(bitcast<u32>(f)));
}
fn f_hash2(f1 : f32, f2 : f32) -> f32  {
	return f_hash(pcg_hash_2d(vec2<u32>(bitcast<u32>(f1), bitcast<u32>(f2))).x);
}
fn f_hash3(f1 : f32, f2 : f32, f3 : f32) -> f32 {
	return f_hash(pcg_hash_3d(vec3<u32>(bitcast<u32>(f1), bitcast<u32>(f2), bitcast<u32>(f3))).x);
}
fn f_hash4(u1 : u32, u2 : u32, u3 : u32, u4 : u32) -> f32 {
	return f_hash(xorshift128(pcg_hash_4d(vec4<u32>(u1, u2, u3, u4))));
}

fn random() -> f32 {
    next_sample++;
    return f_hash4(parameters.seed, 0xDEADBEEFu, bitcast<u32>(parameters.dispatch_cnt), next_sample);
}

//const discarded_point : vec3<f32> = vec3(NaN);

//additional include snippets inserted on initialization
//@includes

// TODO: figure out how to template (??) transforms
fn apply_transform(iter : Iterator, _p_input : P_State) -> vec3<f32>{
    let p : vec3<f32> = _p_input.pos.xyz;
    let iter_depth : i32 = _p_input.iteration_depth;

    //transform snippets inserted on initialization
    //@transforms

    return p;
}

// returns color index
fn apply_coloring(it: Iterator, p0: vec4<f32>, p: vec4<f32>, color_index_in: f32) -> f32 {
    var color_index = color_index_in;
	let in_color = color_index;
	var speed = it.color_speed;
	if (it.shading_mode == 1)
	{
		let p_delta = length(p - p0);
		speed *= (1.0 - 1.0 / (1.0 + p_delta));
	}

    let new_index = mix(in_color, it.color_index, speed);
    if(new_index != 0.0 && fract(new_index) == 0.0) {
        color_index = new_index;
    } else {
	    color_index = fract(new_index);
    }
    return color_index;
}

fn getPaletteColor(pos: f32) -> vec3<f32> {
	var palettepos : f32 = pos * f32(settings.palettecnt - 1);
	let index : i32 = i32(floor(palettepos));
	let c1 : vec3<f32> = palette[index].xyz;
    if (index + 1 == settings.palettecnt) {
        return c1;
    }
	let c2 = palette[index+1].xyz;
	let a = fract(palettepos);
	//TODO: interpolate in a different color space?
	return mix(c1, c2, a);//lerp
}

fn project_perspective(c : camera_params, p : vec3<f32>) -> vec2<f32>{ //should update next
    var p_norm = c.view_proj_mat * vec4(p, 1.0f);

    //discard behind camera
    let dir = normalize(p_norm.xyz);
    if (dot(dir, vec3(0.0,0.0,-1.0)) < 0.0){
        return vec2(-2.0, -2.0);
    }

    if (any(p_norm > vec4(3.40282347E+38)) || p_norm.w == 0.0){ //may be projected to infinity, or w is sometimes 0, not sure why
        return vec2(-2.0, -2.0);
    }

    p_norm /= p_norm.w;

    //dof
    let blur : f32 = f32(c.aperture * max(0.0, abs(dot(p - c.focus_point.xyz, -c.forward.xyz)) - c.depth_of_field)); //use focalplane normal
    let ra   : f32 = random();
    let rl   : f32 = random();
    let swizzle = pow(rl, 0.5f) * blur * vec2(cos(ra * TAU), sin(ra * TAU));
    p_norm.x += swizzle.x;
    p_norm.y += swizzle.y;

    //discard at edges
    let cl : vec2<f32> = clamp(p_norm.xy, vec2(-1.0), vec2(1.0));
    if (length(p_norm.xy - cl) != 0.0){
        return vec2(-2.0, -2.0);
    }

    let ratio = f32(parameters.width) / f32(parameters.height);
    return vec2(
        (p_norm.x + 1) * 0.5 * f32(parameters.width) - 0.5,
        (p_norm.y * ratio + 1) * 0.5 * f32(parameters.height) - 0.5);
}

//Code for supporting alternate render targets. Not going to deal with implementing this until there's a need for it
/*
project_equirectangular(camera_params c, vec3 p, inout uint next)
{
    vec4 p_norm = c.view_proj_mat * vec4(p, 1.0f);
    vec3 dir = normalize(p_norm.xyz);
@group(0) @binding(9) var<storage, read_write> next_sample: u32;
    //rotate so that the center remains in the center of the equirectangular image where it's the most detailed
    dir = rotate_euler(vec3(-PI/2.0, PI/2.0, 0.0)) * dir;

    float a1 = atan(dir.y, dir.x);
    float a2 = -asin(dir.z);
    float x_px = (a1 / PI + 1.0) * 0.5 * width - 0.5;
    float y_px = (a2 / PI + 0.5) * height - 0.5;

    return vec2(x_px, y_px);
}

//Azimuthal Equidistant projection, aka Postel projection, aka Fisheye projection.
//With a circular frame: supposed to be used only for a square image, the corners are left black. Used for dome masters.
vec2 project_fisheye(camera_params c, vec3 p, inout uint next) {
    vec4 p0 = c.view_proj_mat * vec4(p, 1.0);

    //discard behind camera
    vec3 dir = normalize(p0.xyz);
    if (dot(dir, vec3(0.0,0.0,-1.0)) < 0.0)
        return vec2(-2.0, -2.0);

    p0 /= p0.w;
    p0 /= p0.z;

    float r = atan(sqrt(p0.x*p0.x + p0.y*p0.y), p0.z);//incidence angle
    float phi = atan(p0.y, p0.x);
    vec2 uv = vec2(0.5) + r/PI * vec2(cos(phi), sin(phi));

    float ratio = width / float(height);
    return vec2(uv.x*width - 0.5,uv.y*height*ratio - 0.5);
}*/

fn project(c : camera_params, p : vec3<f32>) -> vec2<f32> {
    return project_perspective(c, p);
    /*
    if (c.projection_type == 0)
        return project_perspective(c, p, next);
    else if(c.projection_type == 1)
        return project_equirectangular(c, p, next);
    else //if(c.projection_type == 2)
        return project_fisheye(c, p, next);*/
}

//alias method sampling in O(1)
//input: uniform random 0-1
//output: sampled iterator's index
fn alias_sample(r01: f32) -> i32 {
	let i = i32(floor(f32(settings.itnum) * r01));
	let y = fract(f32(settings.itnum) * r01);
	//biased coin flip
	if (y < iterators[i].reset_prob) {
		return i;
	}
	return iterators[i].reset_alias;
}

fn alias_sample_xaos(iterator_index : u32, r01: f32) -> i32 {
	let i = i32(floor(f32(settings.itnum) * r01));
	let y = fract(f32(settings.itnum) * r01);
	let t = alias_tables[iterator_index * settings.itnum + u32(i)];
	let prob = t.x;
	if (y < prob){
		return i;
	}
	return i32(t.y);
}

//from [0,1] uniform to [0,inf] ln
fn startingDistribution(uniformR: f32) -> f32 {
	let a = pow(uniformR, 1.0/3.0); //avoid center of sphere
	let curve = 1.578425; //1.578425: half of the values are < 0.5	//TODO: parameter?
	// float curve = 0.5 + 10.0 * pow(1.001, -dispatch_cnt);
	return -1.0 / curve * log(1.0 - a);
}

fn reset_state() -> P_State {
	var p: P_State;
	//init points into a starting distribution

	let theta = TAU * random();
	let phi = acos(2.0 * random() - 1.0);
	var rho = startingDistribution(random());//[0,inf] ln
	//experiment: rho dependent on camera distance from origo
	rho *= 2.0 * length(settings.camera.position);
    let sin_phi = sin(phi);
	p.pos = vec4(
		rho * sin_phi * cos(theta),
		rho * sin_phi * sin(theta),
		rho * cos(phi),
		0.0//unused
	);
	//p.iterator_index = int(/*random(next)*/workgroup_random * settings.itnum);
	p.iterator_index = alias_sample(random());
	p.color_index = iterators[p.iterator_index].color_index;
	p.iteration_depth = 0;
	return p;
}

fn sinc(x : f32) -> f32{
	if (x==0.0){
		return 1.0;
	}
	return sin(PI * x) / (PI * x);
}

fn Lanczos(x: f32, n: i32) -> f32 {
	//n>0
	if (abs(x) <= f32(n)){
		return sinc(x) * sinc(x / f32(n));
	}
	return 0.0;
}

fn Mitchell_Netravali(x : f32) -> f32
{
	//const float B = 1.0 / 3.0;
	//const float C = 1.0 / 3.0;
	//best when B + 2*C = 1
	let B = 0.35;
	let C = 0.325;

	let a : f32 = abs(x);
	if (a < 1.0){
		return ((12.0 - 9.0 * B - 6.0 * C) * (a * a * a) + (-18.0 + 12.0 * B + 6.0 * C) * (a * a) + 6.0 - 2.0 * B) / 6.0;
	} else if (1.0 <= a && a < 2.0) {
		return ((-B - 6.0 * C) * (a * a * a) + (6.0 * B + 30.0 * C) * (a * a) + (-12.0 * B - 48.0 * C) * a + 8.0 * B + 24.0 * C) / 6.0;
	} else {
		return 0.0;
    }
}

fn accumulate_hit(proj: vec2<i32>, color: vec4<f32>) {
	let ipx = proj.x + proj.y * i32(parameters.width);//pixel index
	histogram[ipx] += color;
}


//TODO -- INLINE RANDOM
@compute @workgroup_size(64) // 64, 1, 1
fn main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
//    let r = vec4(random(), random(), random(), random());
//    let i = u32(random() % (1920 * 1080));
//    let x = f32(id.x) / (64*256);
//    histogram[id.x] = vec4(x, x, x, 1.0);


	let gid = id.x;

	var p : P_State;
	if parameters.reset_points_state == 1 { p = reset_state(); }
	else { p = state[gid]; }

	for (var i = 0; i < parameters.invocation_iters; i++)
	{
		//pick a random xaos weighted Transform index
		var r_index: i32 = -1;
		let r: f32 = random();
		r_index = alias_sample_xaos(u32(p.iterator_index), r);
		if (       	  	  r_index == -1 || //no outgoing weight
                p.iteration_depth == -1 || //invalid point position
			random() < settings.entropy) //chance to reset by entropy
		{//reset if invalid
			p = reset_state();
		}
		else {
			p.iterator_index = r_index;
		}

		let selected_iterator : Iterator = iterators[p.iterator_index];

		let p0_pos : vec4<f32> = p.pos;
		let p_ret  : vec3<f32> = apply_transform(selected_iterator, p);
		let swizzler = mix(p0_pos.xyz, p_ret + p0_pos.xyz * selected_iterator.tf_add, selected_iterator.tf_mix);
		p.pos.x = swizzler.x;
		p.pos.y = swizzler.y;
		p.pos.z = swizzler.z;

        if (dot(p.pos.xyz, p.pos.xyz) == 0.0
//            any(isinf(p.pos.xyz)) || //at infinity
//            any(isnan(p.pos.xyz)) //reset by plugin
        ) { //check if position is invalid
            p.iteration_depth = -1;//means invalid
            continue;
        }

		apply_coloring(selected_iterator, p0_pos, p.pos, p.color_index);
		p.iteration_depth++;

		if (p.iteration_depth < i32(settings.warmup) || selected_iterator.opacity == 0.0) {
			continue;//avoid useless projection and histogram writes
        }

		let projf : vec2<f32> = project(settings.camera, p.pos.xyz);
        if (projf.x == -2.0) {
            continue;//out of frame
        }
        let proj  = vec2<i32>(i32(round(projf.x)), i32(round(projf.y)));

		var color = vec4(getPaletteColor(p.color_index), selected_iterator.opacity);

		//TODO: this is the same as dof
		let defocus = max(0.0, abs(dot(p.pos.xyz - settings.camera.focus_point.xyz, -settings.camera.forward.xyz)) - settings.camera.depth_of_field);

		if (settings.fog_effect > 0.0f) {
		    //optional fog effect
			var fog_mask : f32 = 2.0*(1.0 - 1.0 / (1.0 + pow(1.0 + settings.fog_effect, - defocus + settings.camera.depth_of_field)));
			fog_mask = clamp(fog_mask, 0.0, 1.0);
			color.w *= fog_mask;
		}
		if (color.w == 0.0) {
			continue;//avoid useless histogram writes
        }
		//mark area in focus with red
		if (settings.mark_area_in_focus != 0 && defocus < 0.01) {
			color = vec4(1.0, 0.0, 0.0, 2.0);
		}
        color.x *= color.w;
        color.y *= color.w;
        color.z *= color.w;

		if (settings.max_filter_radius > 0/* && proj.x>width/2*/) { //max filter radius <= aperture?
			//TODO: determine filter_radius based on settings.filter_method
			let filter_radius = i32(settings.max_filter_radius);

			//for (int ax = -filter_radius; ax <= filter_radius; ax++)
			let ax : i32 = -filter_radius + i32(random()) * 2 * filter_radius;

            //for (int ay = -filter_radius; ay <= filter_radius; ay++)
            let ay : i32 = -filter_radius + i32(random()) * 2 * filter_radius;

            var nb: vec2<i32> = proj + vec2<i32>(ax, ay);
            let pd: f32 = f32(distance(vec2<f32>(nb), projf));

            //TODO: use settings.filter_method to pick one
            //float aw = max(0.0, 1.0-pd);
            //float aw = max(0.0, Lanczos(pd, 2));
            let aw: f32 = f32(max(0.0, Mitchell_Netravali(pd)) * f32(filter_radius * filter_radius * 2 * 2));
            if (settings.camera.projection_type == 1){
                nb.x = nb.x % i32(parameters.width);
            }
            if (nb.x >= 0 && nb.x < i32(parameters.width) && nb.y >= 0 && nb.y < i32(parameters.height)) {
                accumulate_hit(nb, aw * color);
            }
		} else {
			accumulate_hit(proj, color);
		}

	}
	state[gid] = p;
}

// .....................................................

const points: array<vec4<f32>, 3> = array<vec4<f32>, 3>( //do we need the generic type specifier?
    vec4<f32>(-1.0, -3.0, 0.0, 1.0), // Bottom-left
    vec4<f32>(3.0, 1.0, 0.0, 1.0), // Right corner
    vec4<f32>(-1.0, 1.0, 0.0, 1.0) // Top-left
);



@vertex
fn vs_main(
    @builtin(vertex_index) idx: u32,
) -> @builtin(position) vec4<f32> {
    var out: vec4<f32>;

    if idx == 0 {
        out = vec4<f32>(-1.0, -3.0, 0.0, 1.0); // Bottom-left
    } else if idx == 1 {
        out = vec4<f32>(3.0, 1.0, 0.0, 1.0); // Right corner
    } else if idx == 2 {
        out = vec4<f32>(-1.0, 1.0, 0.0, 1.0); // Top-left
    }

	return out;
}


@fragment
fn fs_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {
    // TODO: render from a texture
    var idx = coord_in.x;
    idx += coord_in.y * f32(parameters.width);

    return histogram[u32(floor(idx))];
}
