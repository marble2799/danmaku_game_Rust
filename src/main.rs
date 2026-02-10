use bevy::{prelude::*, color::palettes::css::RED};
use rand::prelude::*;

#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct Bullet;
#[derive(Component)]
struct InGameEntity;
#[derive(Component)]
struct Velocity(Vec3);
#[derive(Component, PartialEq)]
enum TeamSide {
    PLAYER,
    ENEMY,
}
#[derive(Component)]
struct Acceleration(Vec3);
#[derive(Component)]
struct ShootCooldown(Timer); // 連射速度を制限するタイマー
#[derive(Component)]
enum Collider{
    CIRCLE(Circle),
    SQUARE(Vec2),
}
impl Collider {
    fn get_size(&self) -> (f32, f32) {
        match self {
            Self::CIRCLE(c)=> (c.radius, c.radius),
            Self::SQUARE(val) => (val[0]/2., val[1]/2.),
        }
    }
}
#[derive(Component)]
struct Status {
    hp:i32,
    attack:i32,
    is_invincible: bool,
}
impl Status {
    fn new(hp:i32, attack:i32) -> Self {
        Status { hp, attack, is_invincible:false }
    }

    fn reduce_hp(&mut self, attack:i32) {
        if !self.is_invincible {
            self.hp -= attack;
        }
    }

    fn get_attack(&self) -> i32 {
        self.attack
    }

    fn is_die(&self) -> bool {
        self.hp <= 0
    }
}

#[derive(Resource)]
struct EnemySpawnConfig {
    cooldown:Timer,
    hp: i32,
    attack: i32,
}
impl EnemySpawnConfig {
    fn new(cooldown:Timer, hp:i32, attack:i32) -> Self {
        EnemySpawnConfig { cooldown, hp, attack }
    }
}
#[derive(Resource)]
struct Score{score:i64, is_changed:bool}
impl Score {
    fn init_score(&mut self) {
        self.score = 0;
        self.is_changed = true;
    }

    fn get_score(&self) -> i64 {
        self.score
    }

    fn get_is_changed(&mut self) -> bool {
        let ans = self.is_changed;
        self.is_changed = false;
        ans
    }

    fn add_score(&mut self, add_val:i64) -> i64 {
        self.score += add_val;
        self.is_changed = true;
        self.score
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    GameStart,
    Playing,
    GameOver,
}

// ゲーム画面全体(本来はこれとWINDOW_WIDTHの差分に残機などを書き込みたい)
// const GAME_WIDTH:f32 = 800.;
// Playerなどが移動できる領域
const WINDOW_WIDTH:f32 = 500.;
const WINDOW_HEIGHT:f32 = 800.;
const ENEMY_SPAWN_DURATION:f32 = 2.;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.2)))
        .insert_resource(EnemySpawnConfig::new(Timer::from_seconds(ENEMY_SPAWN_DURATION, TimerMode::Repeating), 10, 1))
        .insert_resource(Score{score:0, is_changed:true})
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
        .init_state::<GameState>()
        .add_systems(Startup, start_setup)
        .add_systems(Update, game_start.run_if(in_state(GameState::GameStart)))
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(Update, (
            move_player,
            shoot_bullet,
            enemy_shoot,
            update_velocity,
            move_velocity,
            despawn_objects,
            spawn_enemies,
            check_collisions_with_bullet,
            check_collisions_between_target,
            update_score,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), gameover_setup)
        .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)))
        .run();
}

fn start_setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    // タイトル画面の設定
    commands.spawn((
        Text::new("Press SpaceKey to start !!"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(400),
            right: px(30),
            ..default()
        }
    ));

    // 操作の説明 | wasdで移動, spaceで射撃(長押しOK)
    commands.spawn((
        Text::new("WASD: movement\nSpaceKey: shoot(can hold)"),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(300),
            right: px(110),
            ..default()
        }
    ));
}

fn setup(mut commands: Commands) {
    // スコアボードの設定
    commands.spawn((
        Text::new("Score: "),
    ))
    .with_child((
        TextSpan::default(),
        ScoreText,
    ));

    // 自機を設定
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3), // 赤色
            custom_size: Some(Vec2::new(30., 30.)),
            ..default()
        },
        Transform::from_xyz(0.,-300., 0.),
        Collider::CIRCLE(Circle::new(5.0)),
        Player,
        Status::new(1, 0), // 弾幕シューティングなので即死, 本体は何かにあたったら即死なのでattackは0
        TeamSide::PLAYER,
        ShootCooldown(Timer::from_seconds(0.1, TimerMode::Repeating)),
        InGameEntity,
    ));
}

fn gameover_setup(
    mut commands: Commands,
    query: Query<Entity, With<Bullet>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        Text::new("Game Over..."),
        TextFont {
            font_size: 50.,
            ..default()
        },
        TextColor(RED.into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(400),
            right: px(75),
            ..default()
        }
    ));
    commands.spawn((
        Text::new("Pree R to restart"),
        TextFont {
            font_size: 30.,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: px(350),
            right: px(100),
            ..default()
        }
    ));
}

fn restart_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, Or<(With<InGameEntity>, With<Text>)>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
        score.init_score();
        next_state.set(GameState::Playing);
    }
}

fn game_start(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    text_query: Query<Entity, With<Text>>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        println!("Game Start!");
        for text_entity in &text_query {
            commands.entity(text_entity).despawn();
        }
        next_state.set(GameState::Playing);
    }
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
                Collider::SQUARE(Vec2::new(10., 20.)),
                TeamSide::PLAYER,
                Status::new(1, 1),
                Bullet,
                Velocity(Vec3::new(0.0, 800.0, 0.0)),
                InGameEntity,
            ));
        }
    }
}

fn update_velocity(
    mut query: Query<(&mut Velocity, &Acceleration)>,
    time: Res<Time>,
) {
    for (mut velocity, acceleration) in query.iter_mut() {
        velocity.0 += acceleration.0 * time.delta_secs();
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
    query: Query<(Entity, &Transform), Or<(With<Bullet>, With<Enemy>)>>,
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

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_config: ResMut<EnemySpawnConfig>,
) {
    // 時間を進める
    spawn_config.cooldown.tick(time.delta());

    if spawn_config.cooldown.is_finished() {
        let mut rng = rand::rng();
        let x = rng.random_range(-200.0..200.0);
        let x_v = if x == 0. {
            0.
        }else if x > 0. {
            rng.random_range(-60.0..-10.)
        }else {
            rng.random_range(-10.0..60.)
        };
        let y = 400.;
        commands.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.5),
                custom_size: Some(Vec2::new(30., 30.)),
                ..default()
            },
            Transform::from_xyz(x,y,0.),
            Enemy,
            Acceleration(Vec3::new(0.0, 125.0, 0.0)),
            Collider::SQUARE(Vec2::new(30., 30.)),
            TeamSide::ENEMY,
            Status::new(spawn_config.hp, spawn_config.attack), // 将来的に複数の敵を作成したいので、spawn_configを参照する
            Velocity(Vec3::new(x_v, -200.0, 0.0)),
            ShootCooldown(Timer::from_seconds(0.4, TimerMode::Repeating)),
            InGameEntity,
        ));
    }
}

fn check_aabb_collision(
    first_transform: &Vec3,
    first_collider: &Collider,
    second_transform: &Vec3,
    second_collider: &Collider,
) -> bool {
    let distance = first_transform - second_transform;
    let size_first = first_collider.get_size();
    let size_second = second_collider.get_size();

    distance.x.abs() < (size_first.0 + size_second.0) && distance.y.abs() < (size_first.1 + size_second.1)
}

fn check_collisions_with_bullet(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &mut Status, &Collider, &TeamSide), With<Bullet>>,
    mut target_query: Query<(Entity, &Transform, &mut Status, &Collider, &TeamSide), Without<Bullet>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    // 弾と{敵, player}が接触しているか確認
    for (bullet_entity, bullet_transform, bullet_status, bullet_collider, bullet_teamside) in &bullet_query {
        for (target_entity, target_transform, mut target_status, target_collider, target_teamside) in target_query.iter_mut() {
            if bullet_teamside == target_teamside {
                continue;
            }
            // aabb方式の衝突判定を行う
            if check_aabb_collision(&bullet_transform.translation, bullet_collider, &target_transform.translation, target_collider) {
                // 敵のHPを減らす
                target_status.reduce_hp(bullet_status.get_attack());

                commands.entity(bullet_entity).despawn();

                // もし敵のHPがゼロなら、スコアを上げて敵を消す
                if target_status.is_die() {
                    if target_teamside == &TeamSide::ENEMY {
                        commands.entity(target_entity).despawn();
                        score.add_score(1);
                    }else { // playerのHPがゼロならGameOver
                        commands.entity(target_entity).despawn();
                        println!("Game Over...");
                        next_state.set(GameState::GameOver);
                        return;
                    }
                }

                // 弾を減らしたあとは内側のループから抜けて無駄な探索を避ける
                break;
            }
        }
    }
}

fn check_collisions_between_target(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &mut Status, &Collider),With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Status, &Collider, Option<&Bullet>), Without<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if let Ok((player_entity, player_transform, mut player_status, player_collider)) = player_query.single_mut() {
        for (enemy_entity, enemy_transform, mut enemy_status, enemy_collider, opt_bullet) in enemy_query.iter_mut() {
            // bulletなら除外
            if let Some(_) = opt_bullet {
                continue;
            }

            if check_aabb_collision(&player_transform.translation, player_collider, &enemy_transform.translation, enemy_collider) {
                // PlayerのHPを減らす
                player_status.reduce_hp(enemy_status.get_attack());
                enemy_status.reduce_hp(player_status.get_attack());

                commands.entity(enemy_entity).despawn();

                // もしPlayerのHPがゼロならGame Over
                if player_status.is_die() {
                    commands.entity(player_entity).despawn();
                    println!("Game Over...");
                    next_state.set(GameState::GameOver);
                    return;
                }
                // 敵のHPがゼロなら削除
                if enemy_status.is_die() {
                    commands.entity(enemy_entity).despawn();
                    score.add_score(1);
                }
            }
        }
    }
}

fn enemy_shoot(
    mut commands: Commands,
    time:Res<Time>,
    mut enemy_query: Query<(&Transform, &mut ShootCooldown), With<Enemy>>,
    player_query: Query<&Transform,With<Player>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_transform, mut enemy_shootcooldown) in enemy_query.iter_mut() {
            enemy_shootcooldown.0.tick(time.delta());

            if enemy_shootcooldown.0.is_finished() {
                // 方向ベクトルの計算
                let diff = player_transform.translation - enemy_transform.translation;
                let direction = if diff.length_squared() > 0.0 {
                    diff.normalize() // 正規化
                }else {
                    Vec3::Y // 敵の弾とplayerが重なっていたらとりあえず下へ
                };

                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.8, 0.8, 0.8),
                        custom_size: Some(Vec2::new(12.0, 12.0)),
                        ..default()
                    },
                    Transform::from_translation(enemy_transform.translation),
                    Bullet,
                    TeamSide::ENEMY,
                    Collider::CIRCLE(Circle::new(5.0)),
                    Status::new(1, 1),
                    Velocity(direction * 300.), // 計算したベクトルに、速度定数を加える
                    InGameEntity,
                ));
            }
        }
    }
}

fn update_score(
    mut score: ResMut<Score>,
    mut query: Query<&mut TextSpan, With<ScoreText>>,
) {
    // Score変わってなかったら編集する必要なし
    if !score.get_is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        **text = format!("{}", score.get_score());
    }
}