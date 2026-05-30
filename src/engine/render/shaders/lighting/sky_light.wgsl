@group(0) @binding(0)
var inputTex: texture_2d<f32>;
@group(0) @binding(1)
var outputTex: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2)
var tiles: texture_2d<u32>;
@group(0) @binding(3)
var<uniform> sky_light: vec3<f32>;

@compute @workgroup_size(16,16)
fn set_sky_light(@builtin(global_invocation_id) gid : vec3<u32>){
    let size = textureDimensions(inputTex);
    let tile = textureLoad(tiles,vec2<u32>((gid.x),(size.y-(gid.y) )),0).r;
    let current = textureLoad(inputTex, vec2<u32>(gid.x,gid.y), 0);
    if tile == 0{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), max(vec4<f32>(sky_light,1.),current));
    }
    else{
        textureStore(outputTex, vec2<i32>(i32(gid.x), i32(gid.y)), current);
    }
}
