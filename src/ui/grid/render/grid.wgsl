#import bevy_render::view::View

struct InfiniteGridPosition {
    translation: vec3<f32>,
};

struct InfiniteGridSettings {
    x_axis_col: vec3<f32>,
    z_axis_col: vec3<f32>,
    minor_line_col: vec4<f32>,
    major_line_col: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> grid_position: InfiniteGridPosition;
@group(1) @binding(1)
var<uniform> grid_settings: InfiniteGridSettings;

struct Vertex {
    @builtin(vertex_index) index: u32,
};


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // 0 1 2 1 2 3
    var grid_plane = array<vec3<f32>, 4>(
        vec3<f32>(-1.0, -1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 0.0),
        vec3<f32>(1.0, -1.0, 0.0),
        vec3<f32>(1.0, 1.0, 0.0)
    );
    let p = grid_plane[vertex.index].xyz;

    var out: VertexOutput;

    out.position = vec4<f32>(p, 1.);
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};



@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let screen_size = vec2<f32>(view.viewport.z, view.viewport.w);
    let ndc_x = (in.position.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = ((screen_size.y - in.position.y) / screen_size.y) * 2.0 - 1.0;
    let ndc = vec4<f32>(ndc_x, ndc_y, 1.0, 1.0);
    let view_space = view.inverse_projection * ndc;
    let view_space_pos = view_space.xyz / view_space.w;
    let t = grid_position.translation;
    let world_space = view.inverse_view * vec4<f32>(view_space_pos, 1.0) + vec4<f32>(-t.x, -t.y, t.z, 0.0);

    let scale_x = view.projection[0][0]; // m00: 2/(right-left)
    let scale_y = view.projection[1][1]; // m11: 2/(top-bottom)

    let world_units_per_px_x = 1.0 / (screen_size.x * 0.5 * scale_x);
    let world_units_per_px_y = 1.0 / (screen_size.x * 0.5 * scale_y);

    let axis_width_x: f32 = 3 * world_units_per_px_x; // How wide the axis lines should appear
    let axis_width_y: f32 = 3 * world_units_per_px_y; // How wide the axis lines should appear

    if (abs(world_space.x) < axis_width_y) {
        return FragmentOutput(vec4<f32>(grid_settings.x_axis_col, 1.0));
    }
    if (abs(world_space.y) < axis_width_y) {
        return FragmentOutput(vec4<f32>(grid_settings.z_axis_col, 1.0));
    }

    // Drawing major grid lines
    let major_spacing: f32 = 500.0; // Distance between major grid lines

    let major_line_width_x: f32 = 2 * world_units_per_px_x; // How wide the major grid lines should appear
    let major_line_width_y: f32 = 2 * world_units_per_px_y; // How wide the major grid lines should appear

    let major_grid_line_x = abs(fract(world_space.x / major_spacing + 0.5) - 0.5) < major_line_width_x / major_spacing;
    let major_grid_line_y = abs(fract(world_space.y / major_spacing + 0.5) - 0.5) < major_line_width_y / major_spacing;
    if (major_grid_line_x || major_grid_line_y) {
        return FragmentOutput(grid_settings.major_line_col);
    }

    // Drawing minor grid lines
    let minor_spacing: f32 = 250.0; // Distance between grid lines
    let minor_line_width_x: f32 = 1 * world_units_per_px_x; // How wide the grid lines should appear
    let minor_line_width_y: f32 = 1 * world_units_per_px_y; // How wide the grid lines should appear

    // Calculate grid lines based on world space position and grid spacing
    let grid_line_x = abs(fract(world_space.x / minor_spacing + 0.5) - 0.5) < minor_line_width_x / minor_spacing;
    let grid_line_y = abs(fract(world_space.y / minor_spacing + 0.5) - 0.5) < minor_line_width_y / minor_spacing;

    // Drawing grid lines
    if (grid_line_x || grid_line_y) {
          return FragmentOutput(grid_settings.minor_line_col);
    }

    return FragmentOutput(vec4<f32>(0.0, 0.0, 0.0, 0.0));
}