use bevy::prelude::*;

const WINDOW_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameStates {
	#[default]
	InGame,
	GameOver,
}

fn setup(mut commands: Commands) {}

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
		.init_state::<GameStates>()
		.run();
}
