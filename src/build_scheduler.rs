use rust_sc2::prelude::{UnitTypeId::{self, Drone, Zergling, Roach, Overlord, Hatchery, Lair, Hive, SpawningPool, EvolutionChamber, RoachWarren, Extractor}, UpgradeId};
use priority_queue::PriorityQueue;

use crate::flourish_bot::FlourishBot;

#[derive(PartialEq, Eq, Hash)]
enum Buildable {
	Unit(UnitTypeId),
	Upgrade(UpgradeId)
}
use Buildable::*;

pub struct BuildScheduler {
	build_queue: PriorityQueue<Buildable, i32>
}

impl BuildScheduler {
	pub fn initialise(upgrades_to_research: &Vec<UpgradeId>) -> Self {
		let mut queue = PriorityQueue::with_capacity(11 + upgrades_to_research.len());

		queue.push(Unit(Drone), 0);
		queue.push(Unit(Zergling), 0);
		queue.push(Unit(Roach), 0);
		queue.push(Unit(Overlord), 0);
		queue.push(Unit(Hatchery), 0);
		queue.push(Unit(Lair), 0);
		queue.push(Unit(Hive), 0);
		queue.push(Unit(SpawningPool), 0);
		queue.push(Unit(EvolutionChamber), 0);
		queue.push(Unit(RoachWarren), 0);
		queue.push(Unit(Extractor), 0);

		for upgrade in upgrades_to_research {
			queue.push(Upgrade(*upgrade), 40);
		}

		Self {
			build_queue: queue
		}
	}
}

// Doing this in FlourishBot so we can have mutable access to BuildScheduler while viewing
// other fields of FlourishBot.
impl FlourishBot {
	fn update_military_priority(&mut self) {
		let my_army_supply = self.supply_army;
		let enemy_army_supply = self.surveillance.rounded_enemy_supply();

		if my_army_supply < enemy_army_supply {
			// panik; enemy has more stuff than us and we could be about to die
			self.build_scheduler.build_queue.change_priority(&Unit(Zergling), 100);

			if 2 * self.counter().all().count(Roach) < self.counter().all().count(Zergling) {
				self.build_scheduler.build_queue.change_priority(&Unit(Roach), 150);
			} else {
				self.build_scheduler.build_queue.change_priority(&Unit(Roach), 90);
			}

			return;
		}

		self.build_scheduler.build_queue.change_priority(&Unit(Zergling), 10);
		if 2 * self.counter().all().count(Roach) < self.counter().all().count(Zergling) {
			self.build_scheduler.build_queue.change_priority(&Unit(Roach), 15);
		} else {
			self.build_scheduler.build_queue.change_priority(&Unit(Roach), 9);
		}
	}
}