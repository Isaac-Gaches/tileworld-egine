struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) scale: f32,
    @location(4) tex_index: u32,
    @location(5) colour: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) light_tex_coord: vec2<f32>,
    @location(1) sprite_uv: vec2<f32>,
};

struct Camera{
    position: vec2<f32>,
    zoom: f32,
    ratio: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var sprite_texture: texture_2d<f32>;
@group(0) @binding(2)
var sprite_sampler: sampler;

struct LightMeta{
    position: vec2<f32>,
    v_render_distance: f32,
    h_render_distance: f32,
    chunk_size: f32,
}

@group(0) @binding(3)
var light_texture: texture_2d<f32>;
@group(0) @binding(4)
var light_sampler: sampler;
@group(0) @binding(5)
var<uniform> light_meta: LightMeta;

struct AtlasFrame{
    min_uv: vec2<f32>,
    max_uv: vec2<f32>,
}

@group(0) @binding(6)
var<storage,read> atlas: array<AtlasFrame>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    out.light_tex_coord = vec2<f32>(
        (instance.position.x + vertex.position.x * instance.scale + 0.5 + light_meta.h_render_distance - light_meta.position.x)/(light_meta.h_render_distance*2.+light_meta.chunk_size),
        1.0-(instance.position.y + vertex.position.y  * instance.scale - 0.5 + light_meta.v_render_distance - light_meta.position.y)/(light_meta.v_render_distance*2.+light_meta.chunk_size)
    );

    let frame = atlas[instance.tex_index];
    let quad_uv = (vertex.position.xy * vec2<f32>(1.0,-1.0) + vec2<f32>(0.5,0.5));
    out.sprite_uv = frame.min_uv + quad_uv * (frame.max_uv - frame.min_uv);

    out.clip_position = vec4<f32>((vec3<f32>(instance.position,0.) + (vertex.position * vec3<f32>(instance.scale,instance.scale,1.0)) - vec3<f32>(camera.position,0.)) * vec3<f32>(camera.zoom,camera.zoom,1.0) * vec3<f32>(1.0,camera.ratio,1.0),1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(sprite_texture, sprite_sampler, in.sprite_uv);
    if tex.a== 0.{discard;}
    let light = textureSample(light_texture, light_sampler, in.light_tex_coord) ;
    return tex * light;
}
