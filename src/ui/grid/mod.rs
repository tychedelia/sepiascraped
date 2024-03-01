use crate::Sets::Ui;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::NoFrustumCulling;
use bevy::render::view::VisibleEntities;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_mod_picking::events::{Drag, DragEnd, DragStart, Pointer};
use bevy_mod_picking::picking_core::Pickable;
use bevy_mod_picking::prelude::{Listener, On};
use bevy_mod_picking::PickableBundle;

mod render;

pub struct InfiniteGridPlugin;

impl Plugin for InfiniteGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PreviousScale { scale: 1.0 })
            .add_systems(Startup, grid_setup)
            .add_systems(Update, resize_grid_drag_mesh.in_set(Ui));
    }

    fn finish(&self, app: &mut App) {
        render::render_app_builder(app);
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Components
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Component, Default)]
pub struct InfiniteGrid;

#[derive(Component, Copy, Clone)]
pub struct InfiniteGridSettings {
    pub x_axis_color: Color,
    pub y_axis_color: Color,
    pub minor_line_color: Color,
    pub major_line_color: Color,
}

impl Default for InfiniteGridSettings {
    fn default() -> Self {
        Self {
            x_axis_color: Color::rgb(1.0, 0.2, 0.2),
            y_axis_color: Color::rgb(0.2, 0.2, 1.0),
            minor_line_color: Color::rgb(0.1, 0.1, 0.1),
            major_line_color: Color::rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct GridFrustumIntersect {
    pub points: [Vec3; 4],
    pub center: Vec3,
    pub up_dir: Vec3,
    pub width: f32,
    pub height: f32,
}

#[derive(Bundle, Default)]
pub struct InfiniteGridBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub settings: InfiniteGridSettings,
    pub grid: InfiniteGrid,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
    pub shadow_casters: VisibleEntities,
    pub no_frustum_culling: NoFrustumCulling,
}

#[derive(Component)]
struct GridDrag;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Resources
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Resource)]
struct PreviousScale {
    scale: f32,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Systems
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

fn drag_grid(
    grid_entity: Query<Entity, With<InfiniteGrid>>,
    drag: Listener<Pointer<Drag>>,
    window: Query<&Window>,
    projection: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>,
    mut transform: Query<&mut Transform>,
) {
    let grid_entity = grid_entity.single();
    let window = window.single();
    let (camera, projection, camera_transform) = projection.single();

    let mut parent_transform = transform.get_mut(grid_entity).unwrap();
    parent_transform.translation.x += drag.delta.x * projection.scale;
    parent_transform.translation.y -= drag.delta.y * projection.scale;

    let screen_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let world_center = camera
        .viewport_to_world_2d(camera_transform, screen_center)
        .expect("Failed to convert screen center to world coordinates");
    let relative_translation = world_center.extend(0.0) - parent_transform.translation;
    let mut transform = transform.get_mut(drag.target).unwrap();
    transform.translation.x = relative_translation.x;
    transform.translation.y = relative_translation.y;
}

pub fn grid_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    let window = window.single();
    commands
        .spawn(InfiniteGridBundle {
            settings: InfiniteGridSettings {
                // shadow_color: None,
                x_axis_color: Color::rgb(1.0, 0.2, 0.2),
                y_axis_color: Color::rgb(0.2, 0.2, 1.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                GridDrag,
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Mesh::from(shape::Quad {
                            size: Vec2::new(window.width() + 10.0, window.height() + 10.0),
                            ..Default::default()
                        }))
                        .into(),
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..Default::default()
                },
                PickableBundle::default(), // <- Makes the mesh pickable.
                On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
                On::<Pointer<Drag>>::run(drag_grid),
            ));
        });
}

pub fn resize_grid_drag_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_drag: Query<(Entity, &Mesh2dHandle), With<GridDrag>>,
    projection: Query<&OrthographicProjection>,
    mut previous_scale: ResMut<PreviousScale>,
) {
    let current_scale = projection.single().scale;
    let previous_scale_value = previous_scale.scale;
    let scale_change = current_scale / previous_scale_value;

    let (entity, mesh) = grid_drag.single();
    let mesh = meshes
        .get_mut(&mesh.0)
        .expect("Failed to get grid drag mesh");
    mesh.scale_by(Vec2::splat(scale_change).extend(1.0));
    // TODO: https://github.com/bevyengine/bevy/issues/4294
    commands.entity(entity).remove::<Aabb>();
    previous_scale.scale = current_scale;
}
