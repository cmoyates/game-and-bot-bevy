// assets/shader.wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Material bind group (Bevy Material2d):
@group(2) @binding(0) var scene_tex: texture_2d<f32>;
@group(2) @binding(1) var scene_samp: sampler;

// Only one uniform: how strong the burnt grade is.
struct Globals {
  burnt_amount: f32,   // 0..1
};
@group(2) @binding(2) var<uniform> globals: Globals;

// --- helpers ---
fn sat3(v: vec3<f32>) -> vec3<f32> {
  return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}

// ACES-ish tone map on RGB, keep A
fn aces_tonemap4(x: vec4<f32>) -> vec4<f32> {
  let a = 2.51;
  let b = 0.03;
  let c = 2.43;
  let d = 0.59;
  let e = 0.14;
  let mapped = sat3((x.rgb * (a * x.rgb + vec3<f32>(b)))
                  / (x.rgb * (c * x.rgb + vec3<f32>(d)) + vec3<f32>(e)));
  return vec4<f32>(mapped, x.a);
}

// "Burnt" look: warm bias + slight highlight desat on RGB, keep A
fn burnt_grade4(color: vec4<f32>, amount: f32) -> vec4<f32> {
  if (amount <= 0.0) { return color; }
  let luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let highlight = smoothstep(0.6, 1.2, luma);

  // warm push toward yellow in highlights
  let warm = vec3<f32>(1.0, 0.97, 0.85);
  var out_rgb = mix(color.rgb, color.rgb * warm, amount * highlight);

  // slight desat as it gets brighter
  let grey = vec3<f32>(luma, luma, luma);
  out_rgb = mix(out_rgb, grey, amount * 0.25 * highlight);

  return vec4<f32>(out_rgb, color.a);
}

// --- fragment ---
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
  var color = textureSample(scene_tex, scene_samp, in.uv);

  // 1) ACES tone map
  color = aces_tonemap4(color);

  // 2) Burnt grade
  color = burnt_grade4(color, globals.burnt_amount);

  // clamp RGB; preserve original alpha
  return vec4<f32>(1.0, 0.0, 0.0, color.a);
}
