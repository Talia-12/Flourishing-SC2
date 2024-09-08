use std::cmp::Reverse;

use priority_queue::PriorityQueue;
use rust_sc2::{ids::UnitTypeId, unit::Unit, units::AllUnits, game_data::Cost, consts::{GAME_SPEED, FRAMES_PER_SECOND}};

use crate::flourish_bot::FlourishBot;

#[derive(Default)]
pub struct Surveillance {
	enemy_units: PriorityQueue<(u64, UnitTypeId), Reverse<u32>>,
	enemy_army_supply: f32
}

impl Surveillance {
	const TIME_TILL_REMOVE: f32 = 3.0 * 60.0;

	pub fn observed_enemy_unit_die(&mut self, unit: Unit) {
		if let Some(_) = self.enemy_units.remove(&(unit.tag(), unit.type_id())) {
			self.enemy_army_supply -= unit.supply_cost()
		}
	}

	pub fn rounded_enemy_supply(&self) -> u32 {
		self.enemy_army_supply.ceil() as u32
	}
}

// Doing this in FlourishBot so we can have mutable access to surveillance while viewing
// units.
impl FlourishBot {
	/// Update Surveillance's record of all observed enemy army units to include ones that have just
	/// been seen, and remove ones that haven't been seen in a while. 
	pub fn update_enemy_units(&mut self) {
		let current_step = self.game_step();

		for unit in self.units.enemy.units.clone().iter().filter(|u| !u.is_worker()) {
			if let None = self.surveillance.enemy_units.push((unit.tag(), unit.type_id()), Reverse(current_step)) {
				// if this unit hasn't been seen before at all:
				self.surveillance.enemy_army_supply += unit.supply_cost();
			}
		}

		while let Some(((tag, unit_type), Reverse(last_seen))) = self.surveillance.enemy_units.pop() {
			if last_seen + (Surveillance::TIME_TILL_REMOVE * FRAMES_PER_SECOND) as u32 > current_step {
				// if the unit was seen some time in the last 3 minutes, assume it hasn't died randomly.
				self.surveillance.enemy_units.push((tag, unit_type), Reverse(last_seen));
				break;
			}

			self.surveillance.enemy_army_supply -= self.get_unit_cost(unit_type).supply
		}
	}
}