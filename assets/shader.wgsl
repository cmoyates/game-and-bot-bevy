// assets/shader.wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// === Bindings (Bevy Material2d) ===
@group(2) @binding(0) var scene_tex: texture_2d<f32>;
@group(2) @binding(1) var scene_samp: sampler;

// ---- Settings ----
// You had only burnt_amount; we extend with CRT params.
// If you can't change uniforms yet, set these nonzero defaults in the code where used.
struct Globals {
  // Your grade:
  burnt_amount: f32,        // 0..3 (0 keeps original; try 0.6..1.2)

  // CRT extras:
  mask_intensity: f32,      // 0..1, strength of vertical RGB triads (try 0.7)
  scanline_intensity: f32,  // 0..1, row darkening amount (try 0.6)
  aberration_px: f32,       // in *pixels*, RGB radial split (0..1.5 typical; 0 disables)
  pixelate_px: f32,         // >=1; 1 = off, 2/3/4 = coarse pixels
};
@group(2) @binding(2) var<uniform> globals: Globals;

// ---- Math helpers ----
const PI: f32 = 3.1415926535897932384626433832795;

fn sat3(v: vec3<f32>) -> vec3<f32> {
  return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}
fn lerp3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> { return a + (b - a) * t; }
fn lerp(a: f32, b: f32, t: f32) -> f32 { return a + (b - a) * t; }

// ---- Tone map & grade (yours, unchanged) ----
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
fn burnt_grade4(color: vec4<f32>, amount: f32) -> vec4<f32> {
  if (amount <= 0.0) { return color; }
  let amt = clamp(amount, 0.0, 3.0);
  let luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let highlight = smoothstep(0.4, 1.1, luma);
  let warm = vec3<f32>(1.0, 0.92, 0.75);
  var out_rgb = mix(color.rgb, color.rgb * warm, amt * 0.8 * highlight);
  let grey = vec3<f32>(luma, luma, luma);
  out_rgb = mix(out_rgb, grey, amt * 0.35 * highlight);
  let contrast = 1.0 + 0.4 * amt;
  out_rgb = ((out_rgb - vec3<f32>(0.5)) * contrast) + vec3<f32>(0.5);
  return vec4<f32>(out_rgb, color.a);
}

// ---- CRT helpers ----

// Pixel-aware UV quantization for optional "pixelate" pass (before sampling)
fn pixelate_uv(uv: vec2<f32>, dims: vec2<f32>, px: f32) -> vec2<f32> {
  if (px <= 1.0) { return uv; }
  let step_uv = vec2<f32>(px) / dims;
  // center sample in the quantized cell:
  return (floor(uv / step_uv) + vec2<f32>(0.5)) * step_uv;
}

// Subtle radial chromatic aberration in *pixel* units
fn sample_with_aberration(uv: vec2<f32>, dims: vec2<f32>, aberr_px: f32) -> vec4<f32> {
  if (aberr_px <= 0.0) {
    return textureSample(scene_tex, scene_samp, uv);
  }
  // Direction from center with aspect compensation so it's round:
  let aspect = dims.x / max(dims.y, 1.0);
  let dir = vec2<f32>(uv.x - 0.5, (uv.y - 0.5) * aspect);
  let len = max(length(dir), 1e-6);
  let unit = dir / len;
  let uv_shift = (aberr_px / dims) * unit;

  let cR = textureSample(scene_tex, scene_samp, uv + uv_shift).r;
  let cG = textureSample(scene_tex, scene_samp, uv).g;
  let cB = textureSample(scene_tex, scene_samp, uv - uv_shift).b;
  let a  = textureSample(scene_tex, scene_samp, uv).a;
  return vec4<f32>(cR, cG, cB, a);
}

// Aperture-grille vertical RGB triads in *screen pixel* space.
// Returns per-channel multipliers near 1.0 for the "lit" stripe, <1 for the other two.
fn aperture_mask(px: vec2<f32>, intensity: f32) -> vec3<f32> {
  let t = clamp(intensity, 0.0, 1.0);
  let stripe = i32(floor(px.x)) % 3; // 0:R, 1:G, 2:B
  // Base mask colors: lit channel at 1.0, others dimmed to m (0.35 default feel)
  let m = 0.35;
  var triad = vec3<f32>(m, m, m);
  if (stripe == 0) { triad = vec3<f32>(1.0, m,   m  ); }
  if (stripe == 1) { triad = vec3<f32>(m,   1.0, m  ); }
  if (stripe == 2) { triad = vec3<f32>(m,   m,   1.0); }
  // Lerp from no mask (1,1,1) to triad by intensity
  return lerp3(vec3<f32>(1.0), triad, t);
}

// Simple sinusoidal scanline modulation in *screen pixel* space.
// 1.0 means no darkening; value dips once per pixel row.
fn scanline_factor(py: f32, intensity: f32) -> f32 {
  let t = clamp(intensity, 0.0, 1.0);
  // Cosine with period = 1 pixel: cos(2π * y)
  let s = 0.5 - 0.5 * cos(2.0 * PI * py);
  // Map to [1 - t*1, 1] (darken valleys, keep peaks at ~1)
  return 1.0 - t * s;
}

// ---- Fragment ----
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
  // Screen resolution in pixels from the postprocess texture:
  let dims_u = textureDimensions(scene_tex, 0);
  let dims = vec2<f32>(f32(dims_u.x), f32(dims_u.y));

  // 1) Pre-sample image (optional pixelation + aberration)
  var uv = pixelate_uv(in.uv, dims, globals.pixelate_px);
  var color = sample_with_aberration(uv, dims, globals.aberration_px);

  // 2) Your tone map + grade
  color = aces_tonemap4(color);
  color = burnt_grade4(color, globals.burnt_amount);

  // 3) CRT mask & scanlines in *screen pixel* space
  let p = uv * dims; // pixel coords
  let mask = aperture_mask(p, globals.mask_intensity);
  let sl = scanline_factor(p.y, globals.scanline_intensity);

  color = vec4<f32>(color.rgb * (mask * sl), color.a);


  // 4) Clamp and output
  return vec4<f32>(sat3(color.rgb), color.a);
}
