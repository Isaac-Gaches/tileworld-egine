@group(0) @binding(0)
var tiles: texture_2d<u32>;

@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16,16)
fn set_occlusion_map(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(tiles);

    let tile = textureLoad(tiles,vec2<u32>((gid.x),(size.y-(gid.y) )),0).r;

    if tile == 1{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(0.0,0.0,0.0,1.));
        return;
    }
    else {
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(0.8,0.8,0.8,1.));
        return;
    }
}