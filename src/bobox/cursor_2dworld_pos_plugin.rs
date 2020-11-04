use bevy::{
    prelude::*, render::camera::Camera, render::camera::OrthographicProjection, window::CursorMoved,
};

/// Creates and keep updated the Cursor2dWorldPos resource that contains
/// the last know cursor position in window coordinates and in world coordinate
/// using the transform and orthographic projection of the Camera named 'camera2d'
pub struct Cursor2dWorldPosPlugin;

impl Plugin for Cursor2dWorldPosPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Cursor2dWorldPos::default())
            .add_system_to_stage(stage::POST_UPDATE, cursor_pos_system.system());
    }
}

/// Last known Cursor position in different coordinates system
#[derive(Default)]
pub struct Cursor2dWorldPos {
    /// Last known position of the cursor in window coordinates
    pub window_pos: Vec2,
    /// Last known position if the cursor in world coordinates
    /// through the eye of the Camera named 'camera2d'
    pub world_pos: Vec2,
}
impl Cursor2dWorldPos {
    fn update_world_pos(
        &mut self,
        camera_ortho: &OrthographicProjection,
        camera_transform: &Transform,
    ) {
        let world_x = self.window_pos.x() + camera_ortho.left + camera_transform.translation().x();
        let world_y =
            self.window_pos.y() + camera_ortho.bottom + camera_transform.translation().y();
        self.world_pos = Vec2::new(world_x, world_y);
    }
}

#[derive(Default)]
struct CursorSystemLocalState {
    cursor_moved_event_reader: EventReader<CursorMoved>,
}
const CAMERA2D_NAME: &str = "Camera2d";
/// This system prints out all mouse events as they come in
fn cursor_pos_system(
    mut state: Local<CursorSystemLocalState>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut cursor_2dworld_pos: ResMut<Cursor2dWorldPos>,
    mut query_camera: Query<(&Camera, &OrthographicProjection, &Transform)>,
    mut query_changed_camera: Query<(
        &Camera,
        Or<(Changed<OrthographicProjection>, Changed<Transform>)>,
    )>,
) {
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        for (camera, camera_ortho, camera_transform) in &mut query_camera.iter() {
            if camera.name == Some(CAMERA2D_NAME.to_string()) {
                cursor_2dworld_pos.window_pos = Vec2::new(event.position.x(), event.position.y());
                cursor_2dworld_pos.update_world_pos(camera_ortho, camera_transform);
            }
        }
    }
    for (camera, (camera_ortho, camera_transform)) in &mut query_changed_camera.iter() {
        if camera.name == Some(String::from("Camera2d")) {
            cursor_2dworld_pos.update_world_pos(&camera_ortho, &camera_transform);
        }
    }
}
