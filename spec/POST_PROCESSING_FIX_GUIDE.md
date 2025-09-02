# Bevy Fullscreen Quad Post-Process (0.16.1) — Agent Guide

## Preconditions

- Project builds on Bevy **0.16.1**.
- There is a module `post.rs` (custom `Material2d`) and a module `render.rs` (offscreen target + cameras).
- The shader file lives at `assets/shader.wgsl`.

> Notes the agent should follow:
>
> - `Material2d` bind group is **group 2**. Texture→`binding(0)`, Sampler→`binding(1)`, your uniform block→`binding(2)`. ([Docs.rs][1])
> - Use `RenderLayers` to isolate what a camera renders. Default layer is 0. Cameras only draw entities with **intersecting** layers. ([Docs.rs][2])
> - For an overlay camera, set **higher `Camera.order`** and **don’t clear** so it draws on top. ([Bevy Cheat Book][3], [Docs.rs][4])

---

## Step 1 — Ensure the material plugin is registered first

**Goal:** Register the pipeline for `PostMaterial` before anything tries to use it.

1. In `post.rs` confirm this structure:

```rust
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{Material2d, Material2dPlugin};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct Globals { pub burnt_amount: f32 }

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PostMaterial {
    #[texture(0)] #[sampler(1)]
    pub scene_tex: Handle<Image>,
    #[uniform(2)]
    pub globals: Globals,
}

impl Material2d for PostMaterial {
    fn fragment_shader() -> ShaderRef { "shader.wgsl".into() }
}

pub struct PostProcessingPlugin;
impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<PostMaterial>::default());
    }
}
```

2. In `main.rs`, ensure plugin order:

```rust
.add_plugins(DefaultPlugins)
.add_plugins(post::PostProcessingPlugin)      // material registered here
.add_plugins(render::RenderToTexturePlugin)   // this spawns cameras + quad
```

**Why:** The pipeline layout for your material is generated from `AsBindGroup`; it must exist before you spawn entities using it. ([Docs.rs][1])

**STOP & WAIT** (Build should succeed, no pipeline binding errors.)

---

## Step 2 — Make the 3D scene render to an Image (offscreen)

**Goal:** Point the main 3D camera at a texture, not the window.

In `render.rs` (or wherever you set up cameras):

- Keep your existing **scene** camera:

```rust
commands.spawn((
    Camera3d::default(),
    Camera { target: scene_rt.clone().into(), ..default() },
    MainSceneCam,
));
```

- `scene_rt` must be an `Image` with `TEXTURE_BINDING | COPY_DST | RENDER_ATTACHMENT` (you already do this).

This matches the “render to texture” pattern. ([Bevy Cheat Book][3])

**STOP & WAIT** (Run the app: it should still show your game via the post path once Step 3 is done.)

---

## Step 3 — Create a dedicated **post** camera that truly draws last

**Goal:** Ensure nothing renders on top of the fullscreen quad.

1. Add **RenderLayers** constant:

```rust
use bevy::render::view::RenderLayers;
const POST_LAYER: u8 = 7;
```

2. Spawn the **post camera** with:

- **Very high `order`** (e.g., 999)
- **No clear** (`ClearColorConfig::None`)
- **RenderLayers::layer(POST_LAYER)**

```rust
use bevy::core_pipeline::clear_color::ClearColorConfig;

commands.spawn((
    Camera2d,
    Camera { order: 999, ..default() },                          // draw last
    Camera2d { clear_color: ClearColorConfig::None, ..default() },// don't wipe
    RenderLayers::layer(POST_LAYER),                              // isolate
    PostCam,
));
```

3. Spawn the **fullscreen quad** on the **same layer**:

```rust
commands.spawn((
    Mesh2d(Mesh::from(Rectangle::default())),
    MeshMaterial2d(PostMaterial {
        scene_tex: scene_rt.clone(),
        globals: Globals { burnt_amount: 0.0 },
    }),
    Transform::from_xyz(0.0, 0.0, 0.0),
    RenderLayers::layer(POST_LAYER),
    FullscreenQuad,
));
```

**Why:**

- `order` controls camera render order. Higher order → draws later/on top. ([Bevy Cheat Book][3])
- `ClearColorConfig::None` makes this camera draw over what’s already in the viewport. ([Docs.rs][4])
- `RenderLayers` ensures this camera **only** sees the post quad; other cameras won’t draw that quad. ([Docs.rs][2])

**STOP & WAIT** (Run. You should see the quad affect the whole frame once Step 4 tint test is applied.)

---

## Step 4 — Prove the pass is actually running (obvious shader test)

**Goal:** Make the effect impossible to miss, then back it out.

Edit `assets/shader.wgsl` to return solid magenta:

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(2) @binding(0) var scene_tex: texture_2d<f32>;
@group(2) @binding(1) var scene_samp: sampler;

struct Globals { burnt_amount: f32, };
@group(2) @binding(2) var<uniform> globals: Globals;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0); // magenta
}
```

**Expected:** full-screen magenta. If you still see game content on top, some other camera is drawing later—print all camera `order`s and fix the outlier, or raise the post camera order again. (Two-camera overlay with no clear and higher order is the documented approach.) ([Bevy Cheat Book][3])

**STOP & WAIT** (Confirm magenta. Then restore your real shader body.)

---

## Step 5 — Sanity UV test (optional but useful)

**Goal:** Confirm the quad fills the viewport and UVs are correct.

Temporarily return the UV gradient:

```wgsl
return vec4<f32>(in.uv, 0.0, 1.0);
```

**Expected:** left-to-right X gradient, top-to-bottom Y gradient. If it doesn’t fill, ensure your quad mesh matches the window size (your resize system) or scale the transform. The official “screenspace texture” example demonstrates this pattern. ([Bevy][5])

**STOP & WAIT** (Confirm the gradient covers the whole screen. Then restore your real shader body.)

---

## Step 6 — Minimal ACES + Burnt shader (with uniform)

**Goal:** Keep only the tonemap + grade while preserving alpha.

Use this stripped shader:

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(2) @binding(0) var scene_tex: texture_2d<f32>;
@group(2) @binding(1) var scene_samp: sampler;

struct Globals { burnt_amount: f32, };
@group(2) @binding(2) var<uniform> globals: Globals;

fn sat3(v: vec3<f32>) -> vec3<f32> {
  return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}

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
  let luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let highlight = smoothstep(0.6, 1.2, luma);
  let warm = vec3<f32>(1.0, 0.97, 0.85);
  var out_rgb = mix(color.rgb, color.rgb * warm, amount * highlight);
  let grey = vec3<f32>(luma, luma, luma);
  out_rgb = mix(out_rgb, grey, amount * 0.25 * highlight);
  return vec4<f32>(out_rgb, color.a);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
  var color = textureSample(scene_tex, scene_samp, in.uv);
  color = aces_tonemap4(color);
  color = burnt_grade4(color, globals.burnt_amount);
  return vec4<f32>(sat3(color.rgb), color.a);
}
```

**STOP & WAIT** (Verify: changing `burnt_amount` at runtime should visibly alter the look.)

---

## Step 7 — Add a toggle to prove uniforms are wired

**Goal:** Flip `burnt_amount` via keyboard.

Add this system:

```rust
fn toggle_burn(
    keys: Res<ButtonInput<KeyCode>>,
    mut mats: ResMut<Assets<PostMaterial>>,
    q_handles: Query<&Handle<PostMaterial>>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        for h in &q_handles {
            if let Some(m) = mats.get_mut(h) {
                m.globals.burnt_amount = if m.globals.burnt_amount >= 0.5 { 0.0 } else { 1.0 };
            }
        }
    }
}
```

Register it:

```rust
.add_systems(Update, toggle_burn)
```

**Expected:** Press **B** → effect snaps between off/on. This validates the `Material2d` uniform path (group 2, binding 2). ([Docs.rs][1])

**STOP & WAIT** (Confirm the toggle works.)

---

## Step 8 — (Optional) Enable shader hot-reload for fast iteration

**Goal:** Save WGSL → see changes live.

Update `DefaultPlugins` to enable watching assets:

```rust
use bevy::asset::{AssetMetaCheck, ChangeWatcher};
use std::time::Duration;

.add_plugins(DefaultPlugins.set(AssetPlugin {
    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
    meta_check: AssetMetaCheck::Never,
    ..default()
}))
```

When `assets/shader.wgsl` changes, Bevy will reload it automatically. (Hot-reloading of assets while running is a documented feature; feature flags and behavior changed across versions, but file watching is the standard path.) ([Bevy Cheat Book][6], [Bevy][7])

**STOP & WAIT** (Edit the shader and confirm it updates live.)

---

## Troubleshooting checklist (fast)

- Seeing normal content **over** your post quad? Verify **post camera** has **higher `order`** and **`ClearColorConfig::None`**, and that nothing else has an even higher order. ([Bevy Cheat Book][3], [Docs.rs][4])
- Post quad not visible? Ensure the **quad is on the same `RenderLayers`** as the post camera and fills the viewport. Use the UV gradient test. ([Docs.rs][2], [Bevy][5])
- Pipeline layout error (missing binding)? Check that WGSL group/bindings **exactly** match `AsBindGroup` attributes and that the **material plugin** is added before spawning the quad. ([Docs.rs][8])

---

[1]: https://docs.rs/bevy/latest/bevy/sprite/trait.Material2d.html?utm_source=chatgpt.com "Material2d in bevy::sprite - Rust"
[2]: https://docs.rs/bevy/latest/bevy/render/view/struct.RenderLayers.html?utm_source=chatgpt.com "RenderLayers in bevy::render::view - Rust"
[3]: https://bevy-cheatbook.github.io/graphics/camera.html?utm_source=chatgpt.com "Cameras - Unofficial Bevy Cheat Book"
[4]: https://docs.rs/bevy/latest/bevy/render/prelude/enum.ClearColorConfig.html?utm_source=chatgpt.com "ClearColorConfig in bevy::render::prelude - Rust"
[5]: https://bevy.org/examples/shaders/shader-material-screenspace-texture/?utm_source=chatgpt.com "Shaders / Material - Screenspace Texture"
[6]: https://bevy-cheatbook.github.io/assets/hot-reload.html?utm_source=chatgpt.com "Hot-Reloading Assets - Unofficial Bevy Cheat Book"
[7]: https://bevy.org/learn/migration-guides/0-11-to-0-12/?utm_source=chatgpt.com "Migration Guide: 0.11 to 0.12"
[8]: https://docs.rs/bevy/latest/bevy/render/render_resource/trait.AsBindGroup.html?utm_source=chatgpt.com "AsBindGroup in bevy::render::render_resource - Rust"
