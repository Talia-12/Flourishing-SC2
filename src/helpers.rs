use rust_sc2::prelude::*;

use crate::flourish_bot::FlourishBot;

impl FlourishBot {
	// Tags of the mineral fields close enough to base to be "base's" mineral fields.
	fn local_mineral_tags(&self, base: &Unit) -> Vec<u64> {
		self.units.mineral_fields
			.iter()
			.closer(11.0, base)
			.map(|m| m.tag())
			.collect::<Vec<u64>>()
	}

	// Adds all workers at the base that don't need to be there to the idle_workers collection.
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

	pub fn add_excess_workers_from_gas(&self, bases: &Units, gas: &Unit, idle_workers: &mut Units) {
		let ideal_harvesters = gas.ideal_harvesters().unwrap() as usize;
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
				.take(assigned_harvesters - ideal_harvesters)
				.cloned(),
		);
	}
}