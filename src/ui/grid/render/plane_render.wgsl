#import bevy_render::view::View

struct InfiniteGridPosition {
    planar_rotation_matrix: mat3x3<f32>,
    origin: vec3<f32>,
    normal: vec3<f32>,

};

struct InfiniteGridSettings {
    scale: f32,
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
        vec3<f32>(-1., -1., 0.),
        vec3<f32>(-1., 1., 0.),
        vec3<f32>(1., -1., 0.),
        vec3<f32>(1., 1., 0.)
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
    let screenSize = vec2<f32>(view.viewport.z, view.viewport.w);
    let ndcX = (in.position.x / screenSize.x) * 2.0 - 1.0;
    let ndcY = ((screenSize.y - in.position.y) / screenSize.y) * 2.0 - 1.0;
    let ndc = vec4<f32>(ndcX, ndcY, 1.0, 1.0);
    let viewSpace = view.inverse_projection * ndc;
    let viewSpacePos = viewSpace.xyz / viewSpace.w;
    let worldSpace = view.inverse_view * vec4<f32>(viewSpacePos, 1.0);

    if (abs(worldSpace.x) < 1.0) {
        return FragmentOutput(vec4<f32>(grid_settings.x_axis_col, 1.0));
    }
    if (abs(worldSpace.y) < 1.0) {
        return FragmentOutput(vec4<f32>(grid_settings.z_axis_col, 1.0));
    }

    // Drawing major grid lines
    let majorSpacing: f32 = 500.0;
    let majorLineWidth: f32 = 0.5;
    let majorGridLineX = abs(fract(worldSpace.x / majorSpacing + 0.5) - 0.5) < majorLineWidth / majorSpacing;
    let majorGridLineY = abs(fract(worldSpace.y / majorSpacing + 0.5) - 0.5) < majorLineWidth / majorSpacing;

    if (majorGridLineX || majorGridLineY) {
        return FragmentOutput(grid_settings.major_line_col);
    }


    let minorSpacing: f32 = 50.0; // Distance between grid lines
    let lineWidth: f32 = 0.3; // How wide the grid lines should appear

    // Calculate grid lines based on world space position and grid spacing
    let gridLineX = abs(fract(worldSpace.x / minorSpacing + 0.5) - 0.5) < lineWidth / minorSpacing;
    let gridLineY = abs(fract(worldSpace.y / minorSpacing + 0.5) - 0.5) < lineWidth / minorSpacing;

    // Drawing grid lines
    if (gridLineX || gridLineY) {
          return FragmentOutput(grid_settings.minor_line_col);
    }

    return FragmentOutput(vec4<f32>(0.0, 0.0, 0.0, 0.0));
}