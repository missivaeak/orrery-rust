@binding(0) @group(0) var<uniform> global_v_uniform: GlobalVertexUniform;
struct GlobalVertexUniform {
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
};

@binding(1) @group(0) var<uniform> global_f_uniform: GlobalVertexUniform;
struct GlobalFragmentUniform {
  camera_position: vec4<f32>,
  light_position: vec4<f32>,
  light_colour: vec4<f32>,
  specular_colour: vec4<f32>,
  ambient_intensity: f32,
  diffuse_intensity: f32,
  specular_intensity: f32,
  specular_gloss: f32,
};

@binding(2) @group(0) var<uniform> object_v_uniform: ObjectVertexUniform;
struct ObjectVertexUniform {
  model: mat4x4<f32>,
};

struct Input {
  @location(0) position: vec4<f32>,
  @location(1) colour: vec4<f32>
}

struct Between {
  @builtin(position) position: vec4<f32>,
  @location(0) colour: vec4<f32>
}

@vertex
fn vs_main(in: Input) -> Between {
  let mvp = global_v_uniform.projection * global_v_uniform.view * object_v_uniform.model;
  var out: Between;
  out.colour = in.colour;
  out.position = mvp * in.position;
  return out;
}

@fragment
fn fs_main(in: Between) -> @location(0) vec4<f32> {
  return in.colour;
}
