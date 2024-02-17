#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput


struct TextureRampSettings {
    mode: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(0) var<uniform> settings: TextureRampSettings;
@group(0) @binding(1) var in_texture_1: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler_1: sampler;
@group(0) @binding(3) var in_texture_2: texture_2d<f32>;
@group(0) @binding(4) var texture_sampler_2: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color_1 = textureSample(in_texture_1, texture_sampler_1, in.uv);
    let color_2 = textureSample(in_texture_2, texture_sampler_2, in.uv);
    if (settings.mode == 0) {
        return color_1 + color_2;
    }
    if (settings.mode == 1) {
        return color_1 * color_2;
    }
    if (settings.mode == 2) {
        return color_1 - color_2;
    }
    if (settings.mode == 3) {
        return color_1 / color_2;
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}