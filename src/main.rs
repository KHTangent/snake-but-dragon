use rand::{rng, Rng};
use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;

const GRID_BORDER: f32 = 2.0;
const GRID_CONTENTS: f32 = 30.0;
const GRID_PIXELS: f32 = GRID_BORDER + GRID_CONTENTS;
const GRID_SIZE: Vec2 = Vec2::new(40.0, 22.0);
const WINDOW_SIZE: Vec2 = Vec2::new(GRID_SIZE.x * GRID_PIXELS, GRID_SIZE.y * GRID_PIXELS);

const INITIAL_SEGMENTS: usize = 4;
const TICS_PER_SECOND: f32 = 4.0;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameStates {
	#[default]
	InGame,
	GameOver,
}

#[derive(Event)]
struct FoodEated;

#[derive(Component)]
struct Food;

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

	fn inverse(&self) -> Self {
		match self {
			Direction::Up => Direction::Down,
			Direction::Down => Direction::Up,
			Direction::Left => Direction::Right,
			Direction::Right => Direction::Left,
		}
	}
}

#[derive(Component)]
struct Player {
	facing: Direction,
	next_movement: Direction,
	segment_positions: VecDeque<Vec2>,
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
	let first_pos = Vec2::new(GRID_SIZE.x / 2.0, GRID_SIZE.y / 2.0);
	let mut segment_positions = VecDeque::with_capacity((GRID_SIZE.x * GRID_SIZE.y) as usize);
	for _ in 0..INITIAL_SEGMENTS {
		segment_positions.push_back(Vec2 { x: -5.0, y: -5.0 });
	}
	(
		Player {
			facing: Direction::Right,
			next_movement: Direction::Right,
			segment_positions,
		},
		GridPos(first_pos),
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

fn make_segment() -> impl Bundle {
	(
		Sprite::from_color(Color::srgb(0.4, 0.4, 1.0), Vec2::ONE),
		GridPos::new(-5.0, -5.0),
		Segment,
		Transform {
			scale: Vec3 {
				x: GRID_CONTENTS,
				y: GRID_CONTENTS,
				z: 1.0,
			},
			translation: Vec3::new(-WINDOW_SIZE.x, -WINDOW_SIZE.y, 1.0),
			..default()
		},
	)
}

fn get_valid_food_placement(player_pos: &Vec2, player: &Player) -> Vec2 {
	let potential_x = rng().random_range(0..GRID_SIZE.x as i32);
	let potential_y = rng().random_range(0..GRID_SIZE.y as i32);
	let potential = Vec2::new(potential_x as f32, potential_y as f32);

	if &potential == player_pos || player.segment_positions.iter().any(|&pos| pos == potential) {
		return get_valid_food_placement(player_pos, player);
	}

	return potential;
}

fn make_food(player_pos: &Vec2, player: &Player) -> impl Bundle {
	let position = get_valid_food_placement(player_pos, player);
	(
		Food,
		GridPos::new(position.x, position.y),
		Sprite::from_color(Color::srgb(0.4, 1.0, 0.4), Vec2::ONE),
		Transform {
			scale: Vec3 {
				x: GRID_CONTENTS,
				y: GRID_CONTENTS,
				z: 1.0,
			},
			translation: Vec3::new(-WINDOW_SIZE.x, -WINDOW_SIZE.y, 1.0),
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
	for _ in 0..INITIAL_SEGMENTS {
		commands.spawn(make_segment());
	}
}

fn move_from_gridpos(query: Query<(&mut Transform, &GridPos)>) {
	for (mut transform, gridpos) in query {
		transform.translation = gridpos.to_translation().extend(1.0);
	}
}

fn process_tick(time: Res<Time>, mut tick_timer: ResMut<TickTimer>) {
	let elapsed = time.delta();
	tick_timer.timer.tick(elapsed);
}

fn move_player(
	tick_timer: Res<TickTimer>,
	player_query: Single<(&mut GridPos, &mut Player)>,
	mut eated_events: ResMut<Events<FoodEated>>,
) {
	if !tick_timer.timer.finished() {
		return;
	}
	let mut just_ate = false;
	for _ in eated_events.drain() {
		just_ate = true;
	}
	let (mut player_pos, mut player) = player_query.into_inner();
	player.facing = player.next_movement.clone();
	player.segment_positions.push_back(player_pos.0.clone());
	player_pos.0 += player.facing.to_vec2();
	if !just_ate {
		player.segment_positions.pop_front();
	}
}

fn move_segments(
	tick_timer: Res<TickTimer>,
	player: Single<&Player>,
	segments: Query<&mut GridPos, With<Segment>>,
) {
	if !tick_timer.timer.finished() {
		return;
	}
	for (index, mut segment_pos) in segments.into_iter().enumerate() {
		if let Some(pos) = player.segment_positions.get(index) {
			segment_pos.0.x = pos.x;
			segment_pos.0.y = pos.y;
		}
	}
}

fn check_self_intersect(
	tick_timer: Res<TickTimer>,
	player_pos: Single<&GridPos, With<Player>>,
	segments: Query<&GridPos, With<Segment>>,
	mut next_state: ResMut<NextState<GameStates>>,
) {
	if !tick_timer.timer.finished() {
		return;
	}
	for segment_pos in segments {
		if segment_pos.0 == player_pos.0 {
			next_state.set(GameStates::GameOver);
		}
	}
}

fn handle_food_eating(
	player_pos: Single<&GridPos, With<Player>>,
	food: Single<(&GridPos, Entity), With<Food>>,
	mut commands: Commands,
	mut food_eated_writer: EventWriter<FoodEated>,
) {
	let (food_pos, food_entity) = food.into_inner();
	if player_pos.0 != food_pos.0 {
		return;
	}
	commands.entity(food_entity).despawn();
	commands.spawn(make_segment());
	food_eated_writer.write(FoodEated);
}

fn spawn_food_if_needed(
	player_query: Single<(&GridPos, &Player)>,
	existing_foods: Query<&Food>,
	mut commands: Commands,
) {
	if existing_foods.iter().len() != 0 {
		return;
	}

	let (player_pos, player) = player_query.into_inner();
	let new_food = make_food(&player_pos.0, &player);
	commands.spawn(new_food);
}

fn handle_inputs(keyboard_input: Res<ButtonInput<KeyCode>>, mut player: Single<&mut Player>) {
	let mut new_direction: Option<Direction> = None;
	if keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
		new_direction = Some(Direction::Up);
	}
	if keyboard_input.just_pressed(KeyCode::KeyA) || keyboard_input.just_pressed(KeyCode::ArrowLeft)
	{
		new_direction = Some(Direction::Left);
	}
	if keyboard_input.just_pressed(KeyCode::KeyD)
		|| keyboard_input.just_pressed(KeyCode::ArrowRight)
	{
		new_direction = Some(Direction::Right);
	}
	if keyboard_input.just_pressed(KeyCode::KeyS) || keyboard_input.just_pressed(KeyCode::ArrowDown)
	{
		new_direction = Some(Direction::Down);
	}

	if let Some(d) = new_direction {
		if player.facing != d.inverse() {
			player.next_movement = d;
		}
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
		.init_resource::<Events<FoodEated>>()
		.add_systems(Startup, setup)
		.add_systems(
			FixedUpdate,
			(
				move_from_gridpos,
				(
					process_tick,
					((
						move_player,
						handle_food_eating,
						move_segments,
						(check_self_intersect, handle_food_eating),
						spawn_food_if_needed,
					)
						.chain()
						.after(process_tick)),
				)
					.run_if(in_state(GameStates::InGame)),
			),
		)
		.add_systems(Update, handle_inputs.run_if(in_state(GameStates::InGame)))
		.init_state::<GameStates>()
		.run();
}
