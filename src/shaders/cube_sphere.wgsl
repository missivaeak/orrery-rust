@binding(0) @group(0) var<uniform> GVU: GlobalVertexUniform;
struct GlobalVertexUniform {
    view_mat: mat4x4f,
    projection_mat: mat4x4f,
};

@binding(1) @group(0) var<uniform> GFU: GlobalFragmentUniform;
struct GlobalFragmentUniform {
    camera_position: vec4f,
    light_position: vec4f,
    light_colour: vec4f,
    specular_colour: vec4f,
    ambient_intensity: f32,
    diffuse_intensity: f32,
    specular_intensity: f32,
    specular_gloss: f32,
};

@binding(2) @group(0) var<uniform> OVU: ObjectVertexUniform;
struct ObjectVertexUniform {
    model_mat: mat4x4f,
    normal_mat: mat4x4f,
};

@binding(3) @group(0) var<uniform> OFU: ObjectFragmentUniform;
struct ObjectFragmentUniform {
    colour: vec4f,
};

@binding(4) @group(0) var texture: texture_cube<f32>;
@binding(5) @group(0) var texture_sampler: sampler;

struct Interpolators {
    @builtin(position) c_position: vec4f,
    @location(0) w_position: vec3f,
    @location(1) w_normal: vec3f,
    @location(2) uv: vec2f,
    @location(3) colour: vec4f,
}

@vertex
fn vs_main(
    @location(0) o_position: vec4f,
    @location(1) o_normal: vec4f,
    @location(2) uv: vec2f,
    @location(3) colour: vec4f
) -> Interpolators {
    let mvp = GVU.projection_mat * GVU.view_mat * OVU.model_mat;
    var out: Interpolators;
    out.w_normal = (OVU.normal_mat * o_normal).xyz;
    out.w_position = (OVU.model_mat * o_position).xyz;
    out.c_position = mvp * o_position;
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment
fn fs_main(
    @location(0) w_position: vec3f,
    @location(1) w_normal: vec3f,
    @location(2) uv: vec2f,
    @location(3) colour: vec4f
) -> @location(0) vec4f {
    let normal_dir = normalize(w_normal);
    let light_dir = normalize(GFU.light_position.xyz - w_position);
    let view_dir = normalize(GFU.camera_position.xyz - w_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse = GFU.diffuse_intensity * max(dot(normal_dir, light_dir), 0.0);
    let specular = GFU.specular_intensity * pow(max(dot(normal_dir, half_dir), 0.0), GFU.specular_gloss);
    let ambient = GFU.ambient_intensity;

    return vec4(colour.xyz * (ambient + diffuse) + GFU.specular_colour.rgb * specular, 1.0);
}
