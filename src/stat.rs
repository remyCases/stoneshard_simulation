use std::ops::{Add, AddAssign};
use serde::{Serialize, Deserialize};
use crate::hit::{Chance, Damage, BodyPart, Hit};
use rand::{self, Rng};
use std::iter::zip;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone, Copy)]
pub enum IdSkills {
    WarcryOther,
    Confusion,
    WarcrySelf,
    FencerStance,
    SeizedInitiative,
    LossInitiative,
    DisengageSelf,
    DisengageOther,
    Bleeding,
    Daze,
    Stun,
    Knockback,
    Immobilization,
    Stagger,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
enum DamageType {
    Slash,
    Pierc,
    Crush,
    Rend,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
enum WeaponType {
    Sword,
    Axe,
    Mace,
    Dagger,
    TwohSword,
    TwohAxe,
    TwohMace,
    Staff,
    Spear,
    Bow,
    Crossbow,
    Rend,
}

impl WeaponType {
    fn additional_effect(&self) -> HashMap<IdSkills, f64> {
        let mut hm = HashMap::new();
        match self {
            WeaponType::Sword => hm.insert(IdSkills::Bleeding, 0.5),
            WeaponType::Mace => hm.insert(IdSkills::Daze, 0.5),
            WeaponType::TwohSword => hm.insert(IdSkills::Bleeding, 0.75),
            WeaponType::TwohMace => hm.insert(IdSkills::Daze, 0.75),
            WeaponType::Spear => hm.insert(IdSkills::Immobilization, 0.75),
            WeaponType::Bow => hm.insert(IdSkills::Immobilization, 0.5),
            WeaponType::Crossbow => hm.insert(IdSkills::Knockback, 1.25),
            WeaponType::Rend => hm.insert(IdSkills::Bleeding, 0.5),
            _ => None,
        };
        hm
    }
}

trait CustomAdd<T = Self> {
    type Output;
    fn add(self, rhs: T) -> Self::Output;
}

impl<T: Add<Output = T>> CustomAdd for Option<T> {
    type Output = Self;
    fn add(self, other: Option<T>) -> Option<T> {
        match (self, other) {
            (Some(i), Some(j)) => Some(i + j),
            (Some(i), None) => Some(i),
            (None, Some(j)) => Some(j),
            (None, None) => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct BobyPart {
    protection: Option<u64>,
    phy_res: Option<f64>,
    slash_res: Option<f64>,
    pierc_res: Option<f64>,
    crush_res: Option<f64>,
    rend_res: Option<f64>,
    bleed_res: Option<f64>,
}

impl Add for BobyPart {
    type Output = Self;

    fn add(self, other: BobyPart) -> BobyPart {
        BobyPart {
            protection: self.protection.add(other.protection),
            phy_res: self.phy_res.add(other.phy_res),
            slash_res: self.slash_res.add(other.slash_res),
            pierc_res: self.pierc_res.add(other.pierc_res),
            crush_res: self.crush_res.add(other.crush_res),
            rend_res: self.rend_res.add(other.rend_res),
            bleed_res: self.bleed_res.add(other.bleed_res),
        }
    }
}

impl AddAssign for BobyPart {
    fn add_assign(&mut self, other: BobyPart) {
        let add_self: BobyPart = *self + other; 
        *self = add_self
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Stat {
    hp: Option<u64>,
    damage: Option<u64>,
    damage_type: Option<DamageType>,
    weapon_type: Option<WeaponType>,
    weapon_dmg: Option<f64>,
    main_hand_eff: Option<f64>,
    armor_pen: Option<f64>,
    accuracy: Option<f64>,
    crit_chance: Option<f64>,
    crit_eff: Option<f64>,
    counter: Option<f64>,
    fumble: Option<f64>,
    bleed_chance: Option<f64>,
    daze_chance: Option<f64>,
    stun_chance: Option<f64>,
    knockback_chance: Option<f64>,
    immobilization_chance: Option<f64>,
    stagger_chance: Option<f64>,
    block: Option<f64>,
    block_power: Option<u64>,
    dodge: Option<f64>,
    fortitude: Option<f64>,
    control_res: Option<f64>,
    move_res: Option<f64>,
    damage_taken: Option<f64>,
    hands: Option<BobyPart>,
    legs: Option<BobyPart>,
    torso: Option<BobyPart>,
    head: Option<BobyPart>,
    flat_damage_receive: Option<u64>,
    percent_damage_receive: Option<f64>,
    can_perform_action: Option<bool>,
}

impl Stat {
    pub fn get_hp(&self) -> Option<u64> {
        self.hp
    }

    pub fn get_counter(&self) -> Option<f64> {
        self.counter
    }

    pub fn get_additional_chance(&self) -> HashMap<IdSkills, f64> {
        let mut additional_chance = HashMap::new();
        additional_chance.insert(IdSkills::Bleeding, self.bleed_chance.unwrap_or(0.0));
        additional_chance.insert(IdSkills::Daze, self.daze_chance.unwrap_or(0.0));
        additional_chance.insert(IdSkills::Stun, self.stun_chance.unwrap_or(0.0));
        additional_chance.insert(IdSkills::Knockback, self.knockback_chance.unwrap_or(0.0));
        additional_chance.insert(IdSkills::Immobilization, self.immobilization_chance.unwrap_or(0.0));
        additional_chance.insert(IdSkills::Stagger, self.stagger_chance.unwrap_or(0.0));
        additional_chance
    }

    pub fn get_additional_res(&self, bodypart: BodyPart) -> HashMap<IdSkills, f64> {
        let fortitude = 1.0 - self.fortitude.unwrap_or(0.0);
        let mut additional_res = HashMap::new();
        additional_res.insert(IdSkills::Bleeding, match bodypart {
            BodyPart::LeftHand => fortitude * (1.0 - self.hands.unwrap().bleed_res.unwrap_or(0.0)),
            BodyPart::RightHand => fortitude * (1.0 - self.hands.unwrap().bleed_res.unwrap_or(0.0)),
            BodyPart::LeftLeg => fortitude * (1.0 - self.legs.unwrap().bleed_res.unwrap_or(0.0)),
            BodyPart::RightLeg => fortitude * (1.0 - self.legs.unwrap().bleed_res.unwrap_or(0.0)),
            BodyPart::Torso => fortitude * (1.0 - self.torso.unwrap().bleed_res.unwrap_or(0.0)),
            BodyPart::Head => fortitude * (1.0 - self.head.unwrap().bleed_res.unwrap_or(0.0)),
            _ => fortitude,
        });
        additional_res.insert(IdSkills::Daze, fortitude * (1.0 - self.control_res.unwrap_or(0.0)));
        additional_res.insert(IdSkills::Stun, fortitude * (1.0 - self.control_res.unwrap_or(0.0)));
        additional_res.insert(IdSkills::Knockback, fortitude * (1.0 - self.move_res.unwrap_or(0.0)));
        additional_res.insert(IdSkills::Immobilization, fortitude * (1.0 - self.move_res.unwrap_or(0.0)));
        additional_res.insert(IdSkills::Stagger, fortitude * (1.0 - self.move_res.unwrap_or(0.0)));
        additional_res
    }

    pub fn attack(&self, other: &Stat) -> Hit{
        let self_can_perform_action = self.can_perform_action.unwrap_or(true);
        if !self_can_perform_action {

            let chance = Chance::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            let damage = Damage::new(0, 0, 0, 0, 0, 0);

            Hit::new(chance, damage, BodyPart::None)
        } else {
            let self_accuracy = self.accuracy.unwrap_or(1.0);
            let self_fumble = self.fumble.unwrap_or(0.0);
            let self_crit_chance = self.crit_chance.unwrap_or(0.0);
            let self_crit_eff = self.crit_eff.unwrap_or(1.0);
            let self_armor_pen = self.armor_pen.unwrap_or(0.0);
            let self_weapon_dmg = self.weapon_dmg.unwrap_or(1.0);
            let self_main_hand_eff = self.main_hand_eff.unwrap_or(1.0);
            let self_damage = self.damage.unwrap_or(0);

            let other_dodge = other.dodge.unwrap_or(0.0);
            let other_block = other.block.unwrap_or(0.0);
            let other_block_value = other.block_power.unwrap_or(0);

            // below 0 dodge increases other accuracy
            let accuracy = if other_dodge < 0.0 { 
                // accuracy cant go above 1
                if self_accuracy - other_dodge > 1.0 {
                    1.0
                } else {
                    self_accuracy - other_dodge
                }
            } 
            else {
                // accuracy cant go above 1
                if self_accuracy > 1.0 {
                    1.0
                } else {
                    self_accuracy
                } 
            };

            // above 1 accuracy decreases other dodge
            // dodge cant go below 0
            let dodge = if other_dodge < 0.0 { 
                0.0 
            } else {
                if self_accuracy > 1.0 {
                    // dodge cant go below 0
                    if other_dodge - (1.0 - self_accuracy) < 0.0 {
                        0.0
                    // dodge cant go above 1
                    } else if other_dodge - (1.0 - self_accuracy) > 1.0 {
                        1.0
                    } else {
                        other_dodge - (1.0 - self_accuracy)
                    }
                } else {
                    // dodge cant go above 1
                    if other_dodge > 1.0 {
                        1.0
                    } else {
                        other_dodge
                    }
                }
            };
            let fumble = if self_fumble < 0.0 { 0.0 } else { if self_fumble > 1.0 { 1.0 } else { self_fumble } };
            let damage = (self_weapon_dmg * self_main_hand_eff * self_damage as f64) as u64;
            let half_hit = accuracy * (1.0 - fumble) * dodge + accuracy * fumble * (1.0 - dodge);
            let normal_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * (1.0 - self_crit_chance);
            let crit_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * self_crit_chance;

            let crit_eff = if self_crit_eff < 1.0 {1.0} else {self_crit_eff};
            let crit_damage = (damage as f64 * crit_eff) as u64;

            let mut rng = rand::thread_rng();
            let (body_part, other_body_part_option) = match rng.gen_range(0..6) {
                0 => (BodyPart::RightLeg, other.legs),
                1 => (BodyPart::LeftLeg, other.legs),
                2 => (BodyPart::RightHand, other.hands),
                3 => (BodyPart::LeftHand, other.hands),
                4 => (BodyPart::Torso, other.torso),
                5 => (BodyPart::Head, other.head),
                _ => (BodyPart::None, None),
            };

            let other_body_part = other_body_part_option.unwrap();

            let flat_damage_reduction = (other_body_part.protection.unwrap_or(0) as f64 * (1.0 - self_armor_pen)) as u64;
            let percent_damage_reduction = other.damage_taken.unwrap_or(1.0) * 
            (1.0 - match self.damage_type.unwrap_or(DamageType::Rend) {
                DamageType::Slash => other_body_part.slash_res.unwrap_or(0.0),
                DamageType::Pierc => other_body_part.pierc_res.unwrap_or(0.0),
                DamageType::Crush => other_body_part.crush_res.unwrap_or(0.0),
                DamageType::Rend => other_body_part.rend_res.unwrap_or(0.0),
            });
            
            let chance = Chance::new(
                crit_hit * (1.0 - other_block),
                normal_hit * (1.0 - other_block),
                half_hit * (1.0 - other_block),
                crit_hit * other_block,
                normal_hit * other_block,
                half_hit * other_block
            );
            let dmg = Damage::new(
                if crit_damage > flat_damage_reduction {((crit_damage - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0},
                if damage > flat_damage_reduction {((damage - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0},
                if damage / 2 > flat_damage_reduction {((damage / 2 - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0},
                if crit_damage > other_block_value + flat_damage_reduction {((crit_damage - other_block_value - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0},
                if damage > other_block_value + flat_damage_reduction {((damage - other_block_value - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0},
                if damage / 2 > other_block_value + flat_damage_reduction {((damage / 2 - other_block_value - flat_damage_reduction) as f64 * percent_damage_reduction) as u64} else {0}
            );
            Hit::new(chance, dmg, body_part)
        }  
    }

    pub fn residual_damage(&self) -> f64 {
        self.percent_damage_receive.unwrap_or(0.0) * self.hp.unwrap_or(0) as f64 + 
        self.flat_damage_receive.unwrap_or(0) as f64
    }

    pub fn additional_effect(&self, other: &Stat, bodypart_hit: BodyPart, is_crit: bool) -> HashMap<IdSkills, bool> {
        let hash_chance = 
        if is_crit {
            let weapon_type = self.weapon_type.unwrap_or(WeaponType::Rend);
            let crit_effect = weapon_type.additional_effect();
            self.get_additional_chance().iter().map(|(s, x)| (*s, x + crit_effect.get(s).unwrap_or(&0.0))).collect()
        } else {
            self.get_additional_chance()
        };
        let mut arr_chance = [0.0f64; 6];
        rand::thread_rng().fill(&mut arr_chance[..]);
        let mut arr_res = [0.0f64; 6];
        rand::thread_rng().fill(&mut arr_res[..]);

        let hash_res = other.get_additional_res(bodypart_hit);

        zip(zip(hash_chance, arr_chance), zip(hash_res, arr_res))
        .map(|(((s, x_c), y_c), ((_, x_r), y_r))| (s, y_c < x_c && y_r < x_r)).collect::<HashMap<IdSkills, bool>>()
    }
}

impl Add for Stat {
    type Output = Self;

    fn add(self, other: Stat) -> Stat {
        Stat{
            hp: self.hp.add(other.hp),
            damage: self.damage.add(other.damage),
            damage_type: self.damage_type,
            weapon_type: self.weapon_type,
            weapon_dmg: self.weapon_dmg.add(other.weapon_dmg),
            main_hand_eff: self.main_hand_eff.add(other.main_hand_eff),
            armor_pen: self.armor_pen.add(other.armor_pen),
            accuracy: self.accuracy.add(other.accuracy),
            crit_chance: self.crit_chance.add(other.crit_chance),
            crit_eff: self.crit_eff.add(other.crit_eff),
            counter: self.counter.add(other.counter),
            fumble: self.fumble.add(other.fumble),
            bleed_chance: self.bleed_chance.add(other.bleed_chance),
            daze_chance: self.daze_chance.add(other.daze_chance),
            stun_chance: self.stun_chance.add(other.stun_chance),
            knockback_chance: self.knockback_chance.add(other.knockback_chance),
            immobilization_chance: self.immobilization_chance.add(other.immobilization_chance),
            stagger_chance: self.stagger_chance.add(other.stagger_chance),
            block: self.block.add(other.block),
            block_power: self.block_power.add(other.block_power),
            dodge: self.dodge.add(other.dodge),
            fortitude: self.fortitude.add(other.fortitude),
            control_res: self.control_res.add(other.control_res),
            move_res: self.move_res.add(other.move_res),
            damage_taken: self.damage_taken.add(other.damage_taken),
            hands: self.hands.add(other.hands),
            legs: self.legs.add(other.legs),
            torso: self.torso.add(other.torso),
            head: self.head.add(other.head),
            flat_damage_receive: self.flat_damage_receive.add(other.flat_damage_receive),
            percent_damage_receive: self.percent_damage_receive.add(other.percent_damage_receive),
            can_perform_action: other.can_perform_action,
        }
    }
}

impl AddAssign for Stat {
    fn add_assign(&mut self, other: Stat) {
        let add_self: Stat = *self + other; 
        *self = add_self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_attack() {
        let damage = 10;
        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(1.0),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(0.0),
            block_power: Some(0),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(damage, damage, damage/2, damage, damage, damage/2));
    }

    #[test]
    fn test_crit_attack() {
        let damage = 10;
        let crit_eff = 2.0;
        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(1.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(0.0),
            block_power: Some(0),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64, damage, damage/2)
        );
    }

    #[test]
    fn test_fumble_attack() {
        let damage = 10;
        let crit_eff = 2.0;
        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(1.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(0.0),
            block_power: Some(0),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64, damage, damage/2)
        );
    }

    #[test]
    fn test_dodge_attack() {
        let damage = 10;
        let crit_eff = 2.0;
        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(0.0),
            block_power: Some(0),
            dodge: Some(1.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64, damage, damage/2)
        );
    }

    #[test]
    fn test_fumbledodge_attack() {
        let damage = 10;
        let crit_eff = 2.0;
        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(1.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(0.0),
            block_power: Some(0),
            dodge: Some(1.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64, damage, damage/2)
        );
    }

    #[test]
    fn test_block_attack() {
        let damage = 10;
        let crit_eff = 2.0;
        let block_power = 5;

        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(1.0),
            main_hand_eff: Some(1.0),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(1.0),
            block_power: Some(block_power),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64 - block_power, damage - block_power, damage/2 - block_power)
        );
    }

    #[test]
    fn test_more_dmg_attack() {
        let base_damage = 10;
        let crit_eff = 2.0;
        let block_power = 5;
        let weapon_dmg = 1.2;
        let main_hand_eff = 1.1;
        let damage = (base_damage as f64 * weapon_dmg * main_hand_eff) as u64;

        let dummy_body_part = BobyPart {
            protection: Some(0),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(base_damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(weapon_dmg),
            main_hand_eff: Some(main_hand_eff),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(1.0),
            block_power: Some(block_power),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64, damage, damage/2, 
            (damage as f64 * crit_eff) as u64 - block_power, damage - block_power, damage/2 - block_power)
        );
    }

    #[test]
    fn test_prot_attack() {
        let base_damage = 10;
        let crit_eff = 2.0;
        let block_power = 5;
        let weapon_dmg = 1.2;
        let main_hand_eff = 1.1;
        let damage = (base_damage as f64 * weapon_dmg * main_hand_eff) as u64;
        let protection = 3;

        let dummy_body_part = BobyPart {
            protection: Some(protection),
            phy_res: Some(0.0),
            slash_res: Some(0.0),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(base_damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(weapon_dmg),
            main_hand_eff: Some(main_hand_eff),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(1.0),
            block_power: Some(block_power),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (damage as f64 * crit_eff) as u64 - protection, damage - protection, damage/2 - protection, 
            (damage as f64 * crit_eff) as u64 - block_power - protection, damage - block_power - protection, 0)
        );
    }

    #[test]
    fn test_res_attack() {
        let base_damage = 10;
        let crit_eff = 2.0;
        let block_power = 5;
        let weapon_dmg = 1.2;
        let main_hand_eff = 1.1;
        let damage = (base_damage as f64 * weapon_dmg * main_hand_eff) as u64;
        let protection = 3;
        let slash_res = 0.3;

        let dummy_body_part = BobyPart {
            protection: Some(protection),
            phy_res: Some(0.0),
            slash_res: Some(1.0 - slash_res),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };

        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(base_damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(weapon_dmg),
            main_hand_eff: Some(main_hand_eff),
            armor_pen: Some(0.0),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(1.0),
            block_power: Some(block_power),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (((damage as f64 * crit_eff) as u64 - protection) as f64 * slash_res) as u64, 
            ((damage - protection) as f64 * slash_res) as u64, 
            ((damage/2 - protection) as f64 * slash_res) as u64, 
            (((damage as f64 * crit_eff) as u64 - block_power - protection) as f64 * slash_res) as u64, 
            ((damage - block_power - protection) as f64 * slash_res) as u64, 
            0)
        );
    }

    #[test]
    fn test_pen_attack() {
        let base_damage = 10;
        let crit_eff = 2.0;
        let block_power = 5;
        let weapon_dmg = 1.2;
        let main_hand_eff = 1.1;
        let damage = (base_damage as f64 * weapon_dmg * main_hand_eff) as u64;
        let base_protection = 3;
        let slash_res = 0.5;
        let armor_pen = 0.2;
        let protection = (base_protection as f64 * (1.0 - armor_pen)) as u64;
        
        let dummy_body_part = BobyPart {
            protection: Some(base_protection),
            phy_res: Some(0.0),
            slash_res: Some(1.0 - slash_res),
            pierc_res: Some(0.0),
            crush_res: Some(0.0),
            rend_res: Some(0.0),
            bleed_res: Some(0.0),
        };
 
        let player_stats = Stat
        {
            hp: Some(100),
            damage: Some(base_damage),
            damage_type: Some(DamageType::Slash),
            weapon_type: Some(WeaponType::Sword),
            weapon_dmg: Some(weapon_dmg),
            main_hand_eff: Some(main_hand_eff),
            armor_pen: Some(armor_pen),
            accuracy: Some(1.0),
            crit_chance: Some(0.0),
            crit_eff: Some(crit_eff),
            counter: Some(0.0),
            fumble: Some(0.0),
            bleed_chance: Some(0.0),
            daze_chance: Some(0.0),
            stun_chance: Some(0.0),
            knockback_chance: Some(0.0),
            immobilization_chance: Some(0.0),
            stagger_chance: Some(0.0),
            block: Some(1.0),
            block_power: Some(block_power),
            dodge: Some(0.0),
            fortitude: Some(0.0),
            control_res: Some(0.0),
            move_res: Some(0.0),
            damage_taken: Some(1.0),
            hands: Some(dummy_body_part.clone()),
            legs: Some(dummy_body_part.clone()),
            torso: Some(dummy_body_part.clone()),
            head: Some(dummy_body_part.clone()),
            flat_damage_receive: Some(0),
            percent_damage_receive: Some(0.0),
            can_perform_action: Some(true),
        };

        let dummy_stat = player_stats.clone();
        let hit_player = player_stats.attack(&dummy_stat);
        
        assert_eq!(hit_player.get_chance(), Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        assert_eq!(hit_player.get_damage(), Damage::new(
            (((damage as f64 * crit_eff) as u64 - protection) as f64 * slash_res) as u64, 
            ((damage - protection) as f64 * slash_res) as u64, 
            ((damage/2 - protection) as f64 * slash_res) as u64, 
            (((damage as f64 * crit_eff) as u64 - block_power - protection) as f64 * slash_res) as u64, 
            ((damage - block_power - protection) as f64 * slash_res) as u64, 
            0)
        );
    }
}