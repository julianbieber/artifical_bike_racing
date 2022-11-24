use bevy::{input::mouse::MouseMotion, prelude::*, transform::TransformSystem};

pub struct CameraPlugin {
    pub active: bool,
}
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_graphics)
            .add_system(cursor_grab_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                rotate_camera.after(TransformSystem::TransformPropagate),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                fix_camera_global.after(rotate_camera),
            )
            .add_system(set_camera_follow);
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    let camera_transform = Transform::from_xyz(0.0, 4.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands
        .spawn(TransformBundle {
            ..Default::default()
        })
        .insert(IsChildOf { parent: None })
        .insert(CameraOrientation::from(camera_transform))
        .with_children(|cb| {
            cb.spawn(Camera3dBundle {
                transform: camera_transform,
                ..Default::default()
            });
        });
}

#[derive(Component)]
struct CameraOrientation {
    x: f32,
    y: f32,
}

/// used to keep track of the currently followed entity
#[derive(Component)]
struct IsChildOf {
    parent: Option<Entity>,
}

/// used to indicate that the enity with this component and follows = true should be followed by the camera
/// if multipe enities fulfill the condition, it depends on the order by which the query returns the entities
#[derive(Component)]
pub struct FollowCamera {
    pub follows: bool,
}

impl CameraOrientation {
    fn rotate_by(&mut self, x_offset: f32, y_offset: f32) {
        self.x += x_offset;
        self.x = self.x.rem_euclid(std::f32::consts::TAU);
        self.y += y_offset;
        self.y = self.y.rem_euclid(std::f32::consts::TAU);
    }
}

impl From<Transform> for CameraOrientation {
    fn from(t: Transform) -> Self {
        let (x, y, _) = t.rotation.to_euler(EulerRot::XYZ);
        Self { x, y }
    }
}

fn rotate_camera(
    mut camera_query: Query<(&mut CameraOrientation, &mut Transform, &mut GlobalTransform)>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    for (mut rotation, mut transform, mut global) in camera_query.iter_mut() {
        for motion in motion_evr.iter() {
            rotation.rotate_by(
                (motion.delta.y / -30.0).to_radians(),
                (motion.delta.x / -30.0).to_radians(),
            );
        }
        let x_rot = Quat::from_rotation_x(rotation.x);
        let y_rot = Quat::from_rotation_y(rotation.y);
        transform.rotation = (y_rot * x_rot).normalize();
        let (global_scale, _, global_translation) = global.to_scale_rotation_translation();
        *global = GlobalTransform::from(Transform {
            translation: global_translation,
            rotation: transform.rotation,
            scale: global_scale,
        })
    }
}

fn fix_camera_global(
    focus: Query<&GlobalTransform, (With<CameraOrientation>, Without<Camera>)>,
    mut camera_query: Query<&mut GlobalTransform, With<Camera>>,
) {
    if let Some(focus) = focus.iter().next() {
        if let Some(mut camera) = camera_query.iter_mut().next() {
            let (scale, rotation, translation) = focus.to_scale_rotation_translation();
            let translation = translation + focus.back() * 30.0;
            *camera = GlobalTransform::from(Transform {
                translation,
                rotation,
                scale,
            });
        }
    }
}

fn cursor_grab_system(mut windows: ResMut<Windows>, mouse: Res<Input<MouseButton>>) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = windows.get_primary_mut().unwrap();

        window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Confined);
        window.set_cursor_visibility(false);
        window.set_mode(WindowMode::Fullscreen);
    }
}

fn set_camera_follow(
    mut camera_query: Query<(Entity, &mut IsChildOf), With<CameraOrientation>>,
    follow_query: Query<(Entity, &FollowCamera)>,
    mut commands: Commands,
) {
    if let Some(camera) = camera_query.iter_mut().next() {
        if let Some(target) = follow_query.iter().find(|f| f.1.follows) {
            match camera.1.parent {
                Some(is_following) if is_following == target.0 => (),
                Some(is_following) => {
                    commands.entity(is_following).remove_children(&[camera.0]);
                    commands.entity(target.0).add_child(camera.0);
                }
                None => {
                    commands.entity(target.0).add_child(camera.0);
                }
            }
        }
    }
}
