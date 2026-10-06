@group(0) @binding(0)
var<uniform> GVU: GlobalVertexUniform;
struct GlobalVertexUniform {
    view_mat: mat4x4f,
    projection_mat: mat4x4f,
}

@group(0) @binding(1)
var<uniform> GFU: GlobalFragmentUniform;
struct GlobalFragmentUniform {
    camera_position: vec4f,
    light_position: vec4f,
    light_colour: vec4f,
    specular_colour: vec4f,
    ambient_intensity: f32,
    diffuse_intensity: f32,
    specular_intensity: f32,
    specular_gloss: f32,
}

@group(0) @binding(2)
var<uniform> OVU: ObjectVertexUniform;
struct ObjectVertexUniform {
    model_mat: mat4x4f,
    normal_mat: mat4x4f,
}

@group(0) @binding(3)
var<uniform> OFU: ObjectFragmentUniform;
struct ObjectFragmentUniform {
    colour: vec4f,
}

@group(0) @binding(4)
var texture: texture_2d_array<f32>;
@group(0) @binding(5)
var texture_sampler: sampler;

var<immediate> imm: Immediates;
struct Immediates {
    mesh_index: u32,
}

struct Interpolators {
    @builtin(position) c_position: vec4f,
    @location(0) w_position: vec3f,
    @location(1) w_normal: vec3f,
    @location(2) o_normal: vec3f,
    @location(3) uv: vec2f,
    @location(4) colour: vec4f,
    @location(5) face_index: u32,
}

@vertex
fn vs_main(
    @location(0) o_position: vec4f,
    @location(1) o_normal: vec4f,
    @location(2) uv: vec3f,
    @location(3) colour: vec4f,
) -> Interpolators {
    let mvp = GVU.projection_mat * GVU.view_mat * OVU.model_mat;
    let face_index = u32(uv.z);
    let heightSample = textureLoad(texture, uv, face_index, 0);

    var out: Interpolators;
    out.w_normal = (OVU.normal_mat * o_normal).xyz;
    out.w_position = (OVU.model_mat * o_position).xyz;
    out.c_position = mvp * o_position;
    out.o_normal = o_normal.xyz;
    out.uv = uv.xy;
    out.colour = colour;
    out.face_index = u32(uv.z);
    return out;
}

@fragment
fn fs_main(
    @location(0) w_position: vec3f,
    @location(1) w_normal: vec3f,
    @location(2) o_normal: vec3f,
    @location(3) uv: vec2f,
    @location(4) colour: vec4f,
    @location(5) face_index: u32,
) -> @location(0) vec4f {
    let normal_dir = normalize(w_normal);
    let light_dir = normalize(GFU.light_position.xyz - w_position);
    let view_dir = normalize(GFU.camera_position.xyz - w_position);
    let half_dir = normalize(view_dir + light_dir);

    let tex_colour = textureSample(texture, texture_sampler, uv, face_index);

    let diffuse = GFU.diffuse_intensity * max(dot(normal_dir, light_dir), 0.0);
    let specular = GFU.specular_intensity
        * pow(max(dot(normal_dir, half_dir), 0.0), GFU.specular_gloss);
    let ambient = GFU.ambient_intensity;

    // Combine cubemap color with lighting
    return vec4(
            tex_colour.rgb * (ambient + diffuse) + GFU.specular_colour.rgb * specular,
            tex_colour.a,
        );
}
