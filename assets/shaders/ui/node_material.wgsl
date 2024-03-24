#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct NodeMaterial {
    selected: u32,
    category_color: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: NodeMaterial;
@group(2) @binding(1) var image_texture: texture_2d<f32>;
@group(2) @binding(2) var image_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

    if (distance(mesh.uv, vec2<f32>(0.5, 0.5)) > 0.7) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let border_width = 0.05;

    if (mesh.uv.x > 1.0 - border_width || mesh.uv.x < border_width || mesh.uv.y > 1.0 - border_width || mesh.uv.y < border_width) {
        let hilight_border_width = 0.02;
        if (mesh.uv.x > 1.0 - hilight_border_width || mesh.uv.x < hilight_border_width || mesh.uv.y > 1.0 - hilight_border_width || mesh.uv.y < hilight_border_width) {
            if (material.selected == 1) {
                return vec4<f32>(1.0, 1.0, 1.0, 1.0);
            } else {
                return material.category_color;
            }
        }

        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else {
        return textureSample(image_texture, image_sampler, map_uv(mesh.uv));
    }
}

fn map_uv(uv: vec2<f32>) -> vec2<f32> {
    return (uv - vec2<f32>(0.05, 0.05)) / 0.9;
}