use bevy::{input::mouse::MouseMotion, prelude::*, transform};

pub struct CameraPlugin {}
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_graphics)
            .add_startup_system(cursor_grab_system)
            .add_system(move_camera);
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    let camera_transform = Transform::from_xyz(0.0, 0.0, 0.0).looking_at(-Vec3::Z, Vec3::Y);
    commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_xyz(0.0, 20.0, 100.0),
            ..Default::default()
        })
        .insert(CameraBase {})
        .with_children(|cb| {
            cb.spawn_bundle(Camera3dBundle {
                transform: camera_transform.clone(),
                ..Default::default()
            })
            .insert(CameraOrientation::from(camera_transform));
        });
}

#[derive(Component)]
struct CameraOrientation {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct CameraBase {}

impl CameraOrientation {
    fn rotate_by(&mut self, x_offset: f32, y_offset: f32) {
        self.x = self.x + x_offset;
        self.x = self.x.rem_euclid(std::f32::consts::TAU);
        self.y = self.y + y_offset;
        self.y = self.y.rem_euclid(std::f32::consts::TAU);
    }
}

impl From<Transform> for CameraOrientation {
    fn from(t: Transform) -> Self {
        let (x, y, _) = t.rotation.to_euler(EulerRot::XYZ);
        Self { x, y }
    }
}

fn move_camera(
    mut camera_query: Query<(&mut CameraOrientation, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    for (mut rotation, mut transform) in camera_query.iter_mut() {
        for motion in motion_evr.iter() {
            rotation.rotate_by(
                (motion.delta.y / -30.0).to_radians(),
                (motion.delta.x / -30.0).to_radians(),
            );
        }
        let x_rot = Quat::from_rotation_x(rotation.x);
        let y_rot = Quat::from_rotation_y(rotation.y);
        transform.rotation = (y_rot * x_rot).normalize();
    }
}
fn cursor_grab_system(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
}
