use rust_sc2::{ids::{UpgradeId::{self, *}, UnitTypeId::{self, *}}, units::iter::UnitsIterator};

use crate::flourish_bot::FlourishBot;

// Takes an UpgradeId and returns which building it is researched in, and what buildings
// and other upgrades need to be complete before that upgrade can be researched.
pub fn upgrade_prereqs(upgrade: UpgradeId) -> Option<(UnitTypeId, Vec<UnitTypeId>, Vec<UpgradeId>)> {
	match upgrade {
    GlialReconstitution => Some((RoachWarren, vec![Lair], vec![])),
    TunnelingClaws => Some((RoachWarren, vec![Lair], vec![])),
    ChitinousPlating => Some((UltraliskCavern, vec![], vec![])),
    ZergMeleeWeaponsLevel1 => Some((EvolutionChamber, vec![], vec![])),
    ZergMeleeWeaponsLevel2 => Some((EvolutionChamber, vec![Lair], vec![ZergMeleeWeaponsLevel1])),
    ZergMeleeWeaponsLevel3 => Some((EvolutionChamber, vec![Hive], vec![ZergMeleeWeaponsLevel2])),
    ZergGroundArmorsLevel1 => Some((EvolutionChamber, vec![], vec![])),
    ZergGroundArmorsLevel2 => Some((EvolutionChamber, vec![Lair], vec![ZergGroundArmorsLevel1])),
    ZergGroundArmorsLevel3 => Some((EvolutionChamber, vec![Hive], vec![ZergGroundArmorsLevel2])),
    ZergMissileWeaponsLevel1 => Some((EvolutionChamber, vec![], vec![])),
    ZergMissileWeaponsLevel2 => Some((EvolutionChamber, vec![Lair], vec![ZergMissileWeaponsLevel1])),
    ZergMissileWeaponsLevel3 => Some((EvolutionChamber, vec![Hive], vec![ZergGroundArmorsLevel2])),
    Overlordspeed => Some((Hatchery, vec![], vec![])),
    Overlordtransport => Some((Lair, vec![], vec![])),
    Burrow => Some((Hatchery, vec![], vec![])),
    Zerglingattackspeed => Some((SpawningPool, vec![Hive], vec![])),
    Zerglingmovementspeed => Some((SpawningPool, vec![], vec![])),
    Hydraliskspeed => Some((HydraliskDen, vec![Lair], vec![])),
    ZergFlyerWeaponsLevel1 => Some((Spire, vec![], vec![])),
    ZergFlyerWeaponsLevel2 => Some((Spire, vec![Lair], vec![ZergFlyerWeaponsLevel1])),
    ZergFlyerWeaponsLevel3 => Some((Spire, vec![Hive], vec![ZergFlyerWeaponsLevel2])),
    ZergFlyerArmorsLevel1 => Some((Spire, vec![], vec![])),
    ZergFlyerArmorsLevel2 => Some((Spire, vec![Lair], vec![ZergFlyerArmorsLevel1])),
    ZergFlyerArmorsLevel3 => Some((Spire, vec![Hive], vec![ZergFlyerArmorsLevel2])),
    InfestorEnergyUpgrade => Some((InfestationPit, vec![], vec![])),
    CentrificalHooks => Some((BanelingNest, vec![Lair], vec![])),
    AnabolicSynthesis => Some((UltraliskCavern, vec![Lair], vec![])),
    HydraliskSpeedUpgrade => Some((HydraliskDen, vec![Lair], vec![])),
    NeuralParasite => Some((InfestationPit, vec![], vec![])),
    LocustLifetimeIncrease => Some((InfestationPit, vec![], vec![])),
    LurkerRange => Some((LurkerDenMP, vec![Hive], vec![])),
    EvolveGroovedSpines => Some((HydraliskDen, vec![], vec![])),
    EvolveMuscularAugments => Some((HydraliskDen, vec![Lair], vec![])),
		_ => None
	}
}

impl FlourishBot {
	pub fn has_prereqs(&self, structures: Vec<UnitTypeId>, upgrades: Vec<UpgradeId>) -> bool {
		for s in structures {
			// if there are any elements of type s, the any will evaluate true once and
			// the if will not be entered; if there are none of type s the true will never
			// be seen, and the if block will be evaluated.
			if !self.has_prereq(s) {
				return false;
			}
		}
		for upgrade in upgrades {
			if !self.has_upgrade(upgrade) {
				return false;
			}
		}

		return true;
	}

	pub fn has_prereq(&self, structure: UnitTypeId) -> bool {
		self.units.my.structures.iter().of_type(structure).any(|_| true)
	}
}