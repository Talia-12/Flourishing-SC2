use rust_sc2::prelude::*;

use crate::flourish_bot::FlourishBot;

impl FlourishBot {
	/// Tags of the mineral fields close enough to base to be "base's" mineral fields.
	fn local_mineral_tags(&self, base: &Unit) -> Vec<u64> {
		self.units.mineral_fields
			.iter()
			.closer(11.0, base)
			.map(|m| m.tag())
			.collect::<Vec<u64>>()
	}

	/// Adds all workers at the base that don't need to be there to the idle_workers collection.
	pub fn add_excess_workers_from_base(&self, base: &Unit, idle_workers: &mut Units) {
		let local_minerals = self.local_mineral_tags(base);

		let assigned_harvesters = base.assigned_harvesters().unwrap();
		let ideal_harvesters = base.ideal_harvesters().unwrap();

		idle_workers.extend(
			self.units
				.my
				.workers
				.iter()
				.filter(|u| {
					u.target_tag().map_or(false, |target_tag| {
						local_minerals.contains(&target_tag)
							|| (u.is_carrying_minerals() && target_tag == base.tag())
					})
				})
				.take(
					(assigned_harvesters - ideal_harvesters)
						as usize,
				)
				.cloned(),
		);
	}

	pub fn add_excess_workers_from_gas(&self, bases: &Units, gas: &Unit, idle_workers: &mut Units, desired_workers: usize) {
		let assigned_harvesters = gas.assigned_harvesters().unwrap() as usize;

		idle_workers.extend(
			self.units
				.my
				.workers
				.iter()
				.filter(|u| {
					u.target_tag().map_or(false, |target_tag| {
						target_tag == gas.tag()
							|| (u.is_carrying_vespene()
								&& target_tag == bases.closest(gas).unwrap().tag())
					})
				})
				.take(assigned_harvesters - desired_workers)
				.cloned(),
		);
	}

	/// Checks if bot has enough resources and supply to build given unit type, if we want a buffer
	/// left over at the end.
	pub fn can_afford_with_buffer(&self, unit: UnitTypeId, check_supply: bool, buffer_minerals: u32, buffer_vespene: u32) -> bool {
		let cost = self.get_unit_cost(unit);
		if self.minerals < cost.minerals + buffer_minerals || self.vespene < cost.vespene + buffer_vespene {
			return false;
		}
		if check_supply && (self.supply_left as f32) < cost.supply {
			return false;
		}
		true
	}

	/// Checks if bot has enough resources and supply to build at least n of a given unit type
	pub fn can_afford_multiple(&self, unit: UnitTypeId, check_supply: bool, n: u32) -> bool {
		let cost = self.get_unit_cost(unit);
		if self.minerals < cost.minerals * n || self.vespene < cost.vespene * n {
			return false;
		}
		if check_supply && (self.supply_left as f32) < cost.supply {
			return false;
		}
		true
	}
}