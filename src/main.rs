use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Danmaku Shooting".into(),
                resolution: (500, 800).into(), // 縦長シューティング
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        })
        )
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
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
        Transform::from_xyz(0.,-300., 0.),
        Player,
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    // query.single_mut()は「対象が世界にちょうど一つだけ存在する」と仮定して取得する
    let mut transform = query.single_mut().unwrap();

    let speed = 500.;
    let mut direction = Vec3::ZERO;

    // キーボード入力を反映させる
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD){
        direction.x += 1.;
    }else if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW){
        direction.y += 1.;
    }else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS){
        direction.y -= 1.;
    }

    // 移動キーの入力を受けた場合のみ実行
    if direction.length() > 0. {
        // 斜め移動しても速度が早くならないように正規化
        direction = direction.normalize();

        // time.delta_secs()を入れることでフレームレートに依らず「1秒間に進む距離」が一定に
        transform.translation += direction * speed * time.delta_secs();

        // 画面端の判定を作成
        let x_limit = 250.0 - 15.0; // 500なので、-250 <= x <= 250
        let y_limit = 400.0 - 15.0; // 800なので、-400 <= y <= 400
        transform.translation.x = transform.translation.x.clamp(-x_limit, x_limit);
        transform.translation.y = transform.translation.y.clamp(-y_limit, y_limit);
    }
}