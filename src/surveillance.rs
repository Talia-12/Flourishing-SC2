use std::cmp::Reverse;

use priority_queue::PriorityQueue;
use rust_sc2::{ids::UnitTypeId, unit::Unit, units::AllUnits, game_data::Cost};

#[derive(Default)]
pub struct Surveillance {
	enemy_units: PriorityQueue<(u64, UnitTypeId), Reverse<u32>>,
	enemy_army_supply: f32
}

impl Surveillance {
	pub fn update_enemy_units<F>(&mut self, current_step: u32, units: &AllUnits, get_unit_cost: F)
			where F: Fn(UnitTypeId) -> Cost {
		
		for unit in units.enemy.units.iter().filter(|u| !u.is_worker()) {
			if let None = self.enemy_units.push((unit.tag(), unit.type_id()), Reverse(current_step)) {
				// if this unit hasn't been seen before at all:
				self.enemy_army_supply += unit.supply_cost();
			}
		}

		while let Some(((tag, unit_type), Reverse(last_seen))) = self.enemy_units.pop() {
			if last_seen + 3 * 60 * 50 > current_step {
				// if the unit was seen some time in the last 3 minutes, assume it hasn't died randomly.
				self.enemy_units.push((tag, unit_type), Reverse(last_seen));
				break;
			}

			self.enemy_army_supply -= get_unit_cost(unit_type).supply
		}
	}

	pub fn observed_enemy_unit_die(&mut self, unit: Unit) {
		if let Some(_) = self.enemy_units.remove(&(unit.tag(), unit.type_id())) {
			self.enemy_army_supply -= unit.supply_cost()
		}
	}

	pub fn rounded_enemy_supply(&self) -> u32 {
		self.enemy_army_supply.ceil() as u32
	}
}