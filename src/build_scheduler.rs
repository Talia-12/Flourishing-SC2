use rust_sc2::prelude::{UnitTypeId, UpgradeId};
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

		queue.push(Unit(UnitTypeId::Drone), 0);
		queue.push(Unit(UnitTypeId::Zergling), 0);
		queue.push(Unit(UnitTypeId::Roach), 0);
		queue.push(Unit(UnitTypeId::Overlord), 0);
		queue.push(Unit(UnitTypeId::Hatchery), 0);
		queue.push(Unit(UnitTypeId::Lair), 0);
		queue.push(Unit(UnitTypeId::Hive), 0);
		queue.push(Unit(UnitTypeId::SpawningPool), 0);
		queue.push(Unit(UnitTypeId::EvolutionChamber), 0);
		queue.push(Unit(UnitTypeId::RoachWarren), 0);
		queue.push(Unit(UnitTypeId::Extractor), 0);

		for upgrade in upgrades_to_research {
			queue.push(Upgrade(*upgrade), 0);
		}

		Self {
			build_queue: queue
		}
	}

	fn update_military_priority(&mut self, bot: &FlourishBot) {
		
	}
}