use rust_sc2::prelude::*;

#[bot]
#[derive(Default)]
struct FlourishingBot;
impl Player for FlourishingBot {
	// This settings are used to connect bot to the game.
	fn get_player_settings(&self) -> PlayerSettings {
		PlayerSettings::new(Race::Random)
			.with_name("BotName")
			.raw_affects_selection(false)
			.raw_crop_to_playable_area(true)
	}
	
	// This method will be called automatically each game step.
	// Main bot's logic should be here.
	// Bot's observation updates before each step.
	fn on_step(&mut self, iteration: usize) -> SC2Result<()> {
		/* Your code here */
		Ok(())
	}
}

fn main() -> SC2Result<()> {
	run_vs_computer(
		// Pass mutable referece to your bot here.
		&mut FlourishingBot::default(),
		// Opponent configuration.
		Computer::new(Race::Random, Difficulty::VeryEasy, None),
		// Map name. Panics if map doesn't exists in "StarCraft II/Maps" folder.
		"EternalEmpireLE",
		// Additional settings:
		// LaunchOptions {
		//     sc2_version: Option<&str>, // Default: None - Latest available patch.
		//     save_replay_as: Option<&str>, // Default: None - Doesn't save replay.
		//     realtime: bool, // Default: false
		// }
		LaunchOptions::default(),
	)
}