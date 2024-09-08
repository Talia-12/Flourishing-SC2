use std::cmp::Ordering;

use rust_sc2::prelude::*;

use crate::prereqs::upgrade_prereqs;
use crate::build_scheduler::BuildScheduler;
use crate::surveillance::Surveillance;

#[bot]
pub struct FlourishBot {
	last_loop_distributed: u32,
	last_loop_upgraded: u32,
	last_debug_messages: f32,
	attacking: bool,
	has_enough_gas: bool,
	has_way_too_much_gas: bool,
	has_enough_workers_for_gas: bool,
	upgrades_to_research: Vec<UpgradeId>,
	pub build_scheduler: BuildScheduler,
	pub surveillance: Surveillance,
}

impl Default for FlourishBot {
	fn default() -> Self {
		let upgrades_to_research = vec![
			UpgradeId::Zerglingmovementspeed,
			UpgradeId::ZergMeleeWeaponsLevel1,
			UpgradeId::ZergMissileWeaponsLevel1,
			UpgradeId::ZergFlyerWeaponsLevel1,
			UpgradeId::ZergGroundArmorsLevel1,
			UpgradeId::ZergFlyerArmorsLevel1,
			UpgradeId::GlialReconstitution,
			UpgradeId::TunnelingClaws,
			UpgradeId::ZergMeleeWeaponsLevel2,
			UpgradeId::ZergMissileWeaponsLevel2,
			UpgradeId::ZergFlyerWeaponsLevel2,
			UpgradeId::ZergGroundArmorsLevel2,
			UpgradeId::ZergFlyerArmorsLevel2,
			UpgradeId::ZergMeleeWeaponsLevel3,
			UpgradeId::ZergMissileWeaponsLevel3,
			UpgradeId::ZergFlyerWeaponsLevel3,
			UpgradeId::ZergGroundArmorsLevel3,
			UpgradeId::ZergFlyerArmorsLevel3,
		];

		Self {
			_bot: Default::default(),
			last_loop_distributed: Default::default(),
			last_loop_upgraded: Default::default(),
			last_debug_messages: Default::default(),
			attacking: Default::default(),
			has_enough_gas: Default::default(),
			has_way_too_much_gas: Default::default(),
			has_enough_workers_for_gas: Default::default(),
			surveillance: Default::default(),
			build_scheduler: BuildScheduler::initialise(&upgrades_to_research),
			upgrades_to_research
		}
	}
}

impl Player for FlourishBot {
	// This settings are used to connect bot to the game.
	fn get_player_settings(&self) -> PlayerSettings {
		PlayerSettings::new(Race::Zerg)
			.with_name("Flourish")
			.raw_affects_selection(false)
			.raw_crop_to_playable_area(true)
	}

	fn on_start(&mut self) -> SC2Result<()> {
		// Setting rallypoint for hatchery
		if let Some(townhall) = self.units.my.townhalls.first() {
			townhall.command(AbilityId::RallyWorkers, Target::Pos(self.start_center), false);
		}

		// Splitting workers to closest mineral crystals
		for u in &self.units.my.workers {
			if let Some(mineral) 	= self.units.mineral_fields.closest(u) {
				u.gather(mineral.tag(), false);
			}
		}

		// Ordering drone on initial 50 minerals
		if let Some(larva) = self.units.my.larvas.first() {
			larva.train(UnitTypeId::Drone, false);
		}
		self.subtract_resources(UnitTypeId::Drone, true);
		
		Ok(())
	}
	
	// This method will be called automatically each game step.
	// Main bot's logic should be here.
	// Bot's observation updates before each step.
	fn on_step(&mut self, _iteration: usize) -> SC2Result<()> {
		self.global_data();
		self.debug_messages();
		self.distribute_workers();
		self.upgrades();
		self.build();
		self.order_units();
		self.execute_micro();
		
		Ok(())
	}

	fn on_event(&mut self, event: Event) -> SC2Result<()> {
		match event {
			Event::UnitDestroyed(tag, team) => {
				if let Some(team) = team {
					if team.is_enemy() {
						self.surveillance.observed_enemy_unit_die(self.units.enemy.all.get(tag).unwrap().clone());
					}
				}
			},
			Event::UnitCreated(_) => { },
			Event::ConstructionStarted(_) => { },
			Event::ConstructionComplete(_) => { },
			Event::RandomRaceDetected(_) => { },
		}

		Ok(())
	}
}

impl FlourishBot {
	const DEBUG_MESSAGE_DELAY: f32 = 60.0;
	const DISTRIBUTION_DELAY: u32 = 8;
	const UPGRADE_DELAY: u32 = 12;

	fn global_data(&mut self) {
		self.has_enough_gas = self.vespene > 200 && self.vespene > self.minerals / 3;
		self.has_way_too_much_gas = self.has_enough_gas && self.vespene > 2*self.minerals;
		self.has_enough_workers_for_gas = self.counter().count(UnitTypeId::Drone) > 10;
		self.update_enemy_units()
	}

	fn debug_messages(&mut self) {
		let time = self.time;
		let last_debug_messages = &mut self.last_debug_messages;
		if *last_debug_messages + Self::DEBUG_MESSAGE_DELAY > time {
			return;
		}
		*last_debug_messages = time;
	}

	fn distribute_workers(&mut self) {
		if self.units.my.workers.is_empty() {
			return;
		}
		let mut idle_workers = self.units.my.workers.idle();
		let bases = self.units.my.townhalls.ready();

		// Check distribution delay if there aren't any idle workers
		let game_loop = self.state.observation.game_loop();
		let last_loop = &mut self.last_loop_distributed;
		if idle_workers.is_empty() && *last_loop + Self::DISTRIBUTION_DELAY + bases.len() as u32 > game_loop {
			return;
		}
		*last_loop = game_loop;

		// Distribute
		let mineral_fields = &self.units.mineral_fields;
		if mineral_fields.is_empty() {
			return;
		}
		if bases.is_empty() {
			return;
		}

		let mut deficit_minings = Units::new();
		let mut deficit_geysers = Units::new();

		// Distributing mineral workers
		let mineral_tags = mineral_fields.iter().map(|m| m.tag()).collect::<Vec<u64>>();
		for base in &bases {
			match base.assigned_harvesters().cmp(&base.ideal_harvesters()) {
				Ordering::Less => for _ in 0..(base.ideal_harvesters().unwrap() - base.assigned_harvesters().unwrap()) {
					deficit_minings.push(base.clone());
				},
				Ordering::Greater => self.add_excess_workers_from_base(base, &mut idle_workers),
				_ => {}
			}
		}

		// Distributing gas workers
		let target_gas_workers: usize = if !self.has_enough_workers_for_gas { 0 } else if self.has_way_too_much_gas { 1 } else if self.has_enough_gas { 2 } else { 3 };

		self.units.my.gas_buildings.iter().ready().for_each(|gas| {
			let ideal_harvesters = gas.ideal_harvesters().unwrap() as usize;
			let assigned_harvesters = gas.assigned_harvesters().unwrap() as usize;

			match gas.assigned_harvesters().cmp(&Some(target_gas_workers as u32)) {
				Ordering::Less => {
					// If there are less than the desired number of gas workers, workers
					// can be stolen from anywhere to put in gas
					idle_workers.extend(self.units.my.workers.filter(|u| {
						u.target_tag()
							.map_or(false, |target_tag| mineral_tags.contains(&target_tag))
					}));
					
					for _ in 0..(ideal_harvesters - assigned_harvesters) {
						deficit_geysers.push(gas.clone());
					}
				}
				Ordering::Greater => self.add_excess_workers_from_gas(&bases, gas, &mut idle_workers, target_gas_workers),
				_ => {}
			}
		});

		// Distributing idle workers
		let minerals_near_base = if idle_workers.len() > deficit_minings.len() + deficit_geysers.len() {
			let minerals = mineral_fields.filter(|m| bases.iter().any(|base| base.is_closer(11.0, *m)));
			if minerals.is_empty() {
				None
			} else {
				Some(minerals)
			}
		} else {
			None
		};

		for u in &idle_workers {
			if let Some(closest) = deficit_geysers.closest(u) {
				let tag = closest.tag();
				deficit_geysers.remove(tag);
				u.gather(tag, false);
			} else if let Some(closest) = deficit_minings.closest(u) {
				u.gather(
					mineral_fields
						.closer(11.0, closest)
						.max(|m| m.mineral_contents().unwrap_or(0))
						.unwrap()
						.tag(),
					false,
				);
				let tag = closest.tag();
				deficit_minings.remove(tag);
			} else if u.is_idle() {
				if let Some(mineral) = minerals_near_base.as_ref().and_then(|ms| ms.closest(u)) {
					u.gather(mineral.tag(), false);
				}
			}
		}
	}

	fn upgrades(&mut self) {
		let game_loop = self.state.observation.game_loop();
		let last_loop = &mut self.last_loop_upgraded;
		if *last_loop + Self::UPGRADE_DELAY as u32 > game_loop {
			return;
		}
		*last_loop = game_loop;

		let mut to_remove = vec![];

		for upgrade in self.upgrades_to_research.clone() {
			if self.has_upgrade(upgrade) {
				to_remove.push(upgrade);
				continue;
			}
			if self.is_ordered_upgrade(upgrade) || !self.can_afford_upgrade(upgrade) {
				continue;
			}

			if let Some((structure_type, prereq_structures, prereq_upgrades)) = upgrade_prereqs(upgrade) {
				if !self.has_prereqs(prereq_structures, prereq_upgrades) {
					continue;
				}
				
				if let Some(structure) = self
					.units
					.my
					.structures
					.iter()
					.find(|s| s.type_id() == structure_type && !s.is_active())
				{
					structure.research(upgrade, false);
					self.subtract_upgrade_cost(upgrade);
				}
			}
		}

		for upgrade in to_remove {
			// Ignore if no such element is found
			if let Some(pos) = self.upgrades_to_research.iter().position(|x| *x == upgrade) {
				self.upgrades_to_research.remove(pos);
			}
		}
	}

	fn get_builder(&self, pos: Point2, mineral_tags: &[u64]) -> Option<&Unit> {
		self.units
			.my
			.workers
			.iter()
			.filter(|u| {
				!(u.is_constructing()
					|| u.is_returning() || u.is_carrying_resource()
					|| (u.is_gathering() && u.target_tag().map_or(true, |tag| !mineral_tags.contains(&tag))))
			})
			.closest(pos)
	}

	fn build(&mut self) {
		if self.minerals < 75 {
			return;
		}

		let tech_buildings = vec![
			(UnitTypeId::SpawningPool, 1, 0.0),
			(UnitTypeId::EvolutionChamber, 2, 0.0),
			(UnitTypeId::RoachWarren, 1, 280.0)
		];

		let mineral_tags = self
			.units
			.mineral_fields
			.iter()
			.map(|u| u.tag())
			.collect::<Vec<u64>>();

		for (tech_building, desired_num, min_start_time) in tech_buildings {
			if self.time < min_start_time || self.counter().all().count(tech_building) >= desired_num {
				continue;
			}

			if self.can_afford(tech_building, false) {
				let place = self.start_location.towards(self.game_info.map_center, 6.0);
				if let Some(location) = self.find_placement(tech_building, place, Default::default()) {
					if let Some(builder) = self.get_builder(location, &mineral_tags) {
						builder.build(tech_building, location, false);
						self.subtract_resources(tech_building, false);
						continue;
					}
				}
			}

			// if we want to build one of these buildings and couldn't, don't try and build the next one.
			break;
		}

		let extractor = UnitTypeId::Extractor;
		let hatchery = UnitTypeId::Hatchery;		
		let num_extractors = self.counter().all().count(extractor);
		let num_hatcheries = self.counter().all().count(hatchery);

		let has_extractors_for_hatcheries = num_extractors >= 2 * num_hatcheries;
		if !self.has_enough_gas && !has_extractors_for_hatcheries && self.can_afford(extractor, false) {
			let start = self.start_location;
			if let Some(geyser) = self.find_gas_placement(start) {
				if let Some(builder) = self.get_builder(geyser.position(), &mineral_tags) {
					builder.build_gas(geyser.tag(), false);
					self.subtract_resources(extractor, false);
				}
			}
		}

		if self.can_afford(hatchery, false) && num_hatcheries < 1 + (self.time / 160.0) as usize {
			if let Some(exp) = self.get_expansion() {
				if let Some(builder) = self.get_builder(exp.loc, &mineral_tags) {
					builder.build(hatchery, exp.loc, false);
					self.subtract_resources(hatchery, false);
				}
			}
		}

		let lair = UnitTypeId::Lair;
		let hive = UnitTypeId::Hive;
		let num_lairs = self.counter().all().count(lair);
		let num_hives = self.counter().all().count(hive);
		if self.can_afford_multiple(lair, false, 2) && num_lairs + num_hives == 0 && self.time > 7.0 * 60.0 {
			if let Some(hatchery) = self.units.my.townhalls.iter().of_type(hatchery).closest(self.start_location) {
				hatchery.train(lair, false);
			}
		}
	}

	fn order_units(&mut self) {
		// Can't order units without resources
			if self.minerals < 50 {
			return;
		}

		// Order one queen per each base
		let queen = UnitTypeId::Queen;
		if self.counter().all().count(queen) < self.units.my.townhalls.len() && self.can_afford(queen, true) {
			if let Some(townhall) = self.units.my.townhalls.first() {
				townhall.train(queen, false);
				self.subtract_resources(queen, true);
			}
		}

		// Can't order units without larva
		if self.units.my.larvas.is_empty() {
			return;
		}

		let over = UnitTypeId::Overlord;
		let mut overs_under_prod = self.counter().ordered().count(over) as u32;

		while overs_under_prod <= 10
			&& (self.supply_left + (7.6 * overs_under_prod as f32) as u32) < 3 + (0.05 * self.supply_cap as f32) as u32
			&& self.supply_cap + 8 * overs_under_prod < 200
			&& self.can_afford(over, false)
		{
			if let Some(larva) = self.units.my.larvas.pop() {
				larva.train(over, false);
				self.subtract_resources(over, false);
				overs_under_prod += 1;
			} else {
				break;
			}
		}

		// when we're getting ready for the timing attack focus on zerglings
		let upgrades = vec![UpgradeId::Zerglingmovementspeed, UpgradeId::ZergMeleeWeaponsLevel1];
		let upgrades_almost_ready = upgrades.iter().any(|upgrade| self.upgrade_progress(*upgrade) >= 0.2 && !self.has_upgrade(*upgrade));
		
		let zergling = UnitTypeId::Zergling;
		while upgrades_almost_ready && self.can_afford(zergling, true) {
			if let Some(larva) = self.units.my.larvas.pop() {
				larva.train(zergling, false);
				self.subtract_resources(zergling, true);
			} else {
				break;
			}
		}

		let drone = UnitTypeId::Drone;
		if (self.supply_workers as usize) < 80.min(self.counter().all().count(UnitTypeId::Hatchery) * 16)
			&& self.can_afford(drone, true)
		{
			if let Some(larva) = self.units.my.larvas.pop() {
				larva.train(drone, false);
				self.subtract_resources(drone, true);
			}
		}

		if self.can_afford(zergling, true) {
			if let Some(larva) = self.units.my.larvas.pop() {
				larva.train(zergling, false);
				self.subtract_resources(zergling, true);
			}
		}
	}

	fn execute_micro(&mut self) {
		// Injecting Larva
		let mut queens = self.units.my.units.filter(|u| {
			u.type_id() == UnitTypeId::Queen
				&& !u.is_using(AbilityId::EffectInjectLarva)
				&& u.has_ability(AbilityId::EffectInjectLarva)
		});
		if !queens.is_empty() {
			self.units
				.my
				.townhalls
				.iter()
				.filter(|h| {
					!h.has_buff(BuffId::QueenSpawnLarvaTimer)
						|| h.buff_duration_remain().unwrap() * 20 > h.buff_duration_max().unwrap()
				})
				.for_each(|h| {
					if let Some(queen) = queens.closest(h) {
						queen.command(AbilityId::EffectInjectLarva, Target::Tag(h.tag()), false);
						let tag = queen.tag();
						queens.remove(tag);
					}
				});
		}

		let zergling = UnitTypeId::Zergling;
		let zerglings = self.units.my.units.of_type(zergling);
		if zerglings.is_empty() {
			return;
		}

		// Check if speed upgrade is >80% ready
		let upgrades = vec![UpgradeId::Zerglingmovementspeed, UpgradeId::ZergMeleeWeaponsLevel1];
		let upgrades_almost_ready = upgrades.iter().all(|upgrade| self.has_upgrade(*upgrade) || self.upgrade_progress(*upgrade) >= 0.8);
		let num_zerglings: usize = self.counter().count(zergling);
		let start_attack_threshold = 20;
		let end_attack_threshold = 3;
		let should_attack = upgrades_almost_ready && num_zerglings > start_attack_threshold || self.attacking && num_zerglings > end_attack_threshold;
		self.attacking = should_attack;

		// Attacking with zerglings or defending our locations
		let targets = if should_attack {
			self.units.enemy.all.ground()
		} else {
			self.units
				.enemy
				.all
				.filter(|e| !e.is_flying() && self.units.my.townhalls.iter().any(|h| h.is_closer(25.0, *e)))
		};
		if !targets.is_empty() {
			for u in &zerglings {
				if let Some(target) = targets
					.iter()
					.in_range_of(u, 0.0)
					.min_by_key(|t| t.hits())
					.or_else(|| targets.closest(u))
				{
					// Don't get stuck trying to kill changelings
					if target.type_id() == UnitTypeId::ChangelingZergling {
						u.attack(Target::Tag(target.tag()), false);
					} else {
						u.attack(Target::Pos(target.position()), false);
					}
				}
			}
		} else {
			let target = if should_attack {
				self.enemy_start
			} else {
				self.start_location.towards(self.start_center, -8.0)
			};
			for u in &zerglings {
				u.move_to(Target::Pos(target), false);
			}
		}
	}
}