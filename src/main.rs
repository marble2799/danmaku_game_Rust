use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Danmaku Shooting".into(),
                resolution: (600, 800).into(), // 縦長シューティング
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // カメラを設定
    commands.spawn(Camera2d::default());

    // 自機を設定
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3), // 赤色
            custom_size: Some(Vec2::new(30., 30.)),
            ..default()
        },
        Transform::from_xyz(0., -300., 0.),
    ));

}