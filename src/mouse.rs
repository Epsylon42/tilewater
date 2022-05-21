use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::camera::Camera;

#[derive(Default)]
pub struct MousePos {
    screen: Vec2,
    world: Vec2,
}

impl MousePos {
    pub fn get_screen(&self) -> Vec2 {
        self.screen
    }

    pub fn get_world(&self) -> Vec2 {
        self.world
    }
}

fn mouse_position(
    mut mouse_pos: ResMut<MousePos>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    for event in cursor_moved_events.iter() {
        mouse_pos.screen = event.position;
    }

    if let Some((camera, transform)) = camera.iter().next() {
        let window = windows.get_primary().unwrap();
        let normalized = (mouse_pos.screen / Vec2::new(window.width(), window.height())
            - Vec2::splat(0.5))
            * 2.0;

        let transform = transform.compute_matrix();
        let camera = camera.projection_matrix;

        mouse_pos.world = (camera * transform)
            .inverse()
            .transform_point3(normalized.extend(0.0))
            .xy();
    }
}

pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos::default())
            .add_system(mouse_position);
    }
}
