@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2)
var tiles: texture_2d<u32>;

fn loadClamped(p: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
    let clamped = clamp(p, vec2<i32>(0), size - 1);
    return textureLoad(inputTex, vec2<u32>(clamped), 0).rgb;
}

@compute @workgroup_size(16,16,1)
fn diffuse_light(@builtin(global_invocation_id) gid : vec3<u32>) {
    let sizeU = textureDimensions(inputTex);

    if (gid.x >= sizeU.x || gid.y >= sizeU.y) {
        return;
    }

    let size = vec2<i32>(sizeU);
    let uv = vec2<i32>(gid.xy);

    let tile = textureLoad(
        tiles,
        vec2<u32>(gid.x, sizeU.y - gid.y),
        0
    ).r;

    let current = textureLoad(inputTex, vec2<u32>(uv), 0).rgb;

    if (tile == 1u) {
        textureStore(outputTex, vec2<u32>(uv), vec4(current,1.0));
        return;
    }

    let decay = 0.8;
    let diagDecay = decay * 0.85;

    let l = loadClamped(uv + vec2(-1,  0), size) * decay;
    let r = loadClamped(uv + vec2( 1,  0), size) * decay;
    let u = loadClamped(uv + vec2( 0,  1), size) * decay;
    let d = loadClamped(uv + vec2( 0, -1), size) * decay;

    let ul = loadClamped(uv + vec2(-1,  1), size) * diagDecay;
    let ur = loadClamped(uv + vec2( 1,  1), size) * diagDecay;
    let dl = loadClamped(uv + vec2(-1, -1), size) * diagDecay;
    let dr = loadClamped(uv + vec2( 1, -1), size) * diagDecay;

    let propagated = max(
        max(max(l, r), max(u, d)),
        max(max(ul, ur), max(dl, dr))
    );

    textureStore(
        outputTex,
        vec2<u32>(uv),
        vec4(max(current, propagated), 1.0)
    );
}