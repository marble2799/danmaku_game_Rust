use bevy::prelude::*;

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Bullet;
#[derive(Component)]
struct Velocity(Vec3);
#[derive(Component)]
struct ShootCooldown(Timer); // 連射速度を制限するタイマー

// 名前付き定数
const WINDOW_WIDTH:f32 = 500.;
const WINDOW_HEIGHT:f32 = 800.;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Danmaku Shooting".into(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(), // 縦長シューティング
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        })
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (
            move_player,
            shoot_bullet,
            move_velocity,
            despawn_objects,
        ))
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
        ShootCooldown(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    // query.single_mut()は「対象が世界にちょうど一つだけ存在する」と仮定して取得する
    // 自機がない場合にpanic!を起こさないようにif letで取り出す
    if let Ok(mut transform) = query.single_mut(){
        let speed = 500.;
        let mut direction = Vec3::ZERO;

        // キーボード入力を反映させる
        // 上下左右の優先が出ないように、elseを使わずに実装する(elseを使うと右>左が生まれてしまう)
        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD){
            direction.x += 1.;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW){
            direction.y += 1.;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS){
            direction.y -= 1.;
        }

        // 移動キーの入力を受けた場合のみ実行
        if direction.length() > 0. {
            // 斜め移動しても速度が早くならないように正規化
            direction = direction.normalize();

            // time.delta_secs()を入れることでフレームレートに依らず「1秒間に進む距離」が一定に
            transform.translation += direction * speed * time.delta_secs();

            // 画面端の判定を作成
            let x_limit = WINDOW_WIDTH / 2. - 15.0; // 500なので、-250 <= x <= 250
            let y_limit = WINDOW_HEIGHT / 2. - 15.0; // 800なので、-400 <= y <= 400
            transform.translation.x = transform.translation.x.clamp(-x_limit, x_limit);
            transform.translation.y = transform.translation.y.clamp(-y_limit, y_limit);
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut ShootCooldown), With<Player>>, // Playerタグを持ち、TransformとShootCooldownを持つEntityを取得
    time: Res<Time>,
) {
    // 自機がない場合にpanic!を起こさないようにif letで取り出す
    if let Ok((transform, mut cooldown)) = query.single_mut(){
        // タイマーを進める
        cooldown.0.tick(time.delta());

        // spaceが押され、cooldownが上がっていれば発射
        if keyboard_input.pressed(KeyCode::Space) && cooldown.0.is_finished() {
            // 弾を生成
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.3, 1.0, 0.5),
                    custom_size: Some(Vec2::new(10.0, 20.0)), // 縦長の弾
                    ..default()
                },
                // 自機の少し上から出す(自機の座標は"真ん中"であることに注意！)
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)),
                Bullet,
                Velocity(Vec3::new(0.0, 800.0, 0.0)),
            ));
        }
    }
}

fn move_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        // velocityに合わせて位置を変更
        transform.translation += velocity.0 * time.delta_secs();
    }
}

fn despawn_objects(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    let under = WINDOW_HEIGHT * -1. / 2.;
    let upper = WINDOW_HEIGHT / 2.;
    let left = WINDOW_WIDTH * -1. / 2.;
    let right = WINDOW_WIDTH / 2.;
    for (entity_id, transform) in query.iter() {
        // 中心が壁の向こう側に行っていたら削除
        if transform.translation.x < left || right < transform.translation.x  || transform.translation.y < under || upper < transform.translation.y {
            commands.entity(entity_id).despawn();
        }
    }
}