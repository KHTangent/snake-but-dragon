use std::time::Duration;

use bevy::prelude::*;

const GRID_BORDER: f32 = 2.0;
const GRID_CONTENTS: f32 = 30.0;
const GRID_PIXELS: f32 = GRID_BORDER + GRID_CONTENTS;
const GRID_SIZE: Vec2 = Vec2::new(40.0, 22.0);
const WINDOW_SIZE: Vec2 = Vec2::new(GRID_SIZE.x * GRID_PIXELS, GRID_SIZE.y * GRID_PIXELS);

const TICS_PER_SECOND: f32 = 4.0;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameStates {
	#[default]
	InGame,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
enum Direction {
	#[default]
	Up,
	Down,
	Left,
	Right,
}

impl Direction {
	fn to_vec2(&self) -> Vec2 {
		match self {
			Direction::Up => Vec2::new(0.0, 1.0),
			Direction::Down => Vec2::new(0.0, -1.0),
			Direction::Left => Vec2::new(-1.0, 0.0),
			Direction::Right => Vec2::new(1.0, 0.0),
		}
	}
}

#[derive(Component)]
struct Player {
	facing: Direction,
}

#[derive(Component)]
struct Segment;

#[derive(Component)]
#[require(Transform)]
struct GridPos(Vec2);

impl GridPos {
	fn new(x: f32, y: f32) -> Self {
		GridPos(Vec2 { x, y })
	}
	fn to_translation(&self) -> Vec2 {
		Vec2 {
			x: (self.0.x - GRID_SIZE.x / 2.0 + 0.5) * GRID_PIXELS,
			y: (self.0.y - GRID_SIZE.y / 2.0 + 0.5) * GRID_PIXELS,
		}
	}
}

fn make_player() -> impl Bundle {
	(
		Player {
			facing: Direction::Right,
		},
		Segment,
		GridPos::new(GRID_SIZE.x / 2.0, GRID_SIZE.y / 2.0),
		Sprite::from_color(Color::srgb(0.0, 0.0, 1.0), Vec2::ONE),
		Transform {
			scale: Vec3 {
				x: GRID_CONTENTS,
				y: GRID_CONTENTS,
				z: 1.0,
			},
			..default()
		},
	)
}
#[derive(Resource)]
struct TickTimer {
	timer: Timer,
}

fn setup(mut commands: Commands) {
	commands.insert_resource(TickTimer {
		timer: Timer::new(
			Duration::from_secs_f32(1.0 / TICS_PER_SECOND),
			TimerMode::Repeating,
		),
	});

	commands.spawn(Camera2d);

	commands.spawn(make_player());
}

fn move_from_gridpos(query: Query<(&mut Transform, &GridPos)>) {
	for (mut transform, gridpos) in query {
		transform.translation = gridpos.to_translation().extend(1.0);
	}
}

fn process_tick(
	time: Res<Time>,
	mut tick_timer: ResMut<TickTimer>,
	player_query: Single<(&mut GridPos, &Player)>,
) {
	let elapsed = time.delta();
	tick_timer.timer.tick(elapsed);
	if !tick_timer.timer.finished() {
		return;
	}

	let (mut player_pos, player) = player_query.into_inner();
	player_pos.0 += player.facing.to_vec2();
}

fn handle_inputs(keyboard_input: Res<ButtonInput<KeyCode>>, mut player: Single<&mut Player>) {
	if keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
		player.facing = Direction::Up;
	}
	if keyboard_input.just_pressed(KeyCode::KeyA) || keyboard_input.just_pressed(KeyCode::ArrowLeft)
	{
		player.facing = Direction::Left;
	}
	if keyboard_input.just_pressed(KeyCode::KeyD)
		|| keyboard_input.just_pressed(KeyCode::ArrowRight)
	{
		player.facing = Direction::Right;
	}
	if keyboard_input.just_pressed(KeyCode::KeyS) || keyboard_input.just_pressed(KeyCode::ArrowDown)
	{
		player.facing = Direction::Down;
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Snake but dragon".into(),
				resizable: false,
				resolution: WINDOW_SIZE.into(),
				..default()
			}),
			..default()
		}))
		.add_systems(Startup, setup)
		.add_systems(
			FixedUpdate,
			(
				move_from_gridpos,
				process_tick.run_if(in_state(GameStates::InGame)),
			),
		)
		.add_systems(Update, handle_inputs.run_if(in_state(GameStates::InGame)))
		.init_state::<GameStates>()
		.run();
}
