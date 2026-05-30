@group(0) @binding(0)
var inputTex: texture_2d<f32>;

@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16,16)
fn smooth_light(@builtin(global_invocation_id) gid : vec3<u32>) {

    let size = textureDimensions(inputTex);

    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let px = i32(gid.x);
    let py = i32(gid.y);

    var sum = vec4<f32>(0.0);

    // 3x3 blur
    for (var oy = -1; oy <= 1; oy += 1) {
        for (var ox = -1; ox <= 1; ox += 1) {

            let x = clamp(px + ox, 0, i32(size.x) - 1);
            let y = clamp(py + oy, 0, i32(size.y) - 1);

            sum += textureLoad(inputTex, vec2<i32>(x, y), 0);
        }
    }

    let blurred = sum / 9.0;

    let current = textureLoad(inputTex, vec2<i32>(px, py), 0);

    let new_pixel = vec4<f32>(
        max(blurred.r, current.r),
        max(blurred.g, current.g),
        max(blurred.b, current.b),
        1.0
    );

    textureStore(outputTex, vec2<i32>(px, py), new_pixel);
}