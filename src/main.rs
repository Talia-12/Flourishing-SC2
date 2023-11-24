use flourish_sc2::flourish_bot::FlourishBot;
use rust_sc2::prelude::*;

fn main() -> SC2Result<()> {
	run_vs_computer(
		// Pass mutable referece to your bot here.
		&mut FlourishBot::default(),
		// Opponent configuration.
		Computer::new(Race::Random, Difficulty::MediumHard, None),
		// Map name. Panics if map doesn't exists in "StarCraft II/Maps" folder.
		"BerlingradAIE",
		// Additional settings:
		// LaunchOptions {
		//     sc2_version: Option<&str>, // Default: None - Latest available patch.
		//     save_replay_as: Option<&str>, // Default: None - Doesn't save replay.
		//     realtime: bool, // Default: false
		// }
		LaunchOptions::default(),
	)
}