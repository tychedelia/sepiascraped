#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput


struct TextureRampSettings {
    mode: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(0) var<uniform> settings: TextureRampSettings;
@group(0) @binding(1) var<uniform> in_texture_1: texture_2d<f32>;
@group(0) @binding(2) var<uniform> in_texture_2: texture_2d<f32>;
@group(0) @binding(3) var<uniform> texture_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color_1 = textureSample(in_texture_1, texture_sampler, in.uv);
    let color_2 = textureSample(in_texture_2, texture_sampler, in.uv);
    return color_1 + color_2;
}