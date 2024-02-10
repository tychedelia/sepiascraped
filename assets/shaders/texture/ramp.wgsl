#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct TextureRampSettings {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    mode: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}

@group(0) @binding(0) var<uniform> settings: TextureRampSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if (settings.mode == 0) {
        return mix(settings.color_a, settings.color_b, in.uv.x);
    } else if (settings.mode == 1) {
        return mix(settings.color_a, settings.color_b, in.uv.y);
    } else if (settings.mode == 2){
        let distFromCenter = distance(in.uv, vec2(0.5, 0.5));
        let gradientFactor = clamp(distFromCenter / 1.0, 0.0, 1.0);
        return mix(settings.color_a, settings.color_b, gradientFactor);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// Function to offset UV coordinates
fn offsetUV(uv: vec2<f32>, offset: f32) -> vec2<f32> {
    // Add the offset and use fract to wrap around [0, 1]
    return fract(uv + vec2<f32>(offset, offset));
}