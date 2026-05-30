@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16,16)
fn upscale_lightmap(@builtin(global_invocation_id) gid : vec3<u32>){
    let current = textureLoad(inputTex, vec2<i32>(i32(gid.x)/2,i32(gid.y)/2), 0);
    textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), current);
}