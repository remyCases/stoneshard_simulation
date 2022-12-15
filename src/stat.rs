use std::ops::{Add, AddAssign};
use serde::{Serialize, Deserialize};
use crate::hit::{Chance, Damage, Hit};
use rand::{self, Rng};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
enum DamageType {
    slash,
    pierc,
    crush,
    rend,
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
}

impl Stat {
    pub fn get_hp(&self) -> Option<u64> {
        self.hp
    }

    pub fn get_counter(&self) -> Option<f64> {
        self.counter
    }

    pub fn attack(&self, other: &Stat) -> Option<Hit>{
        let self_accuracy = self.accuracy?;
        let self_fumble = self.fumble?;
        let self_crit_chance = self.crit_chance.unwrap_or(0.0);
        let self_crit_eff = self.crit_eff.unwrap_or(1.0);
        let self_armor_pen = self.armor_pen.unwrap_or(0.0);
        let self_weapon_dmg = self.weapon_dmg.unwrap_or(1.0);
        let self_main_hand_eff = self.main_hand_eff.unwrap_or(1.0);
        let self_damage = self.damage?;
        
        let other_dodge = other.dodge.unwrap_or(0.0);
        let other_block = other.block?;
        let other_block_value = other.block_power?;

        let accuracy = if other_dodge < 0.0 { 
            if self_accuracy - other_dodge > 1.0 {
                1.0
            } else {
                self_accuracy - other_dodge
            }
        } 
        else {
            self_accuracy 
        };

        let dodge = if other_dodge < 0.0 { 0.0 } else { if other_dodge > 1.0 { 1.0 } else { other_dodge } };
        let fumble = if self_fumble < 0.0 { 0.0 } else { if self_fumble > 1.0 { 1.0 } else { self_fumble } };

        let damage = (self_weapon_dmg * self_main_hand_eff * self_damage as f64) as u64;

        let half_hit = accuracy * (1.0 - fumble) * dodge + accuracy * fumble * (1.0 - dodge);
        let normal_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * (1.0 - self_crit_chance);
        let crit_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * self_crit_chance;

        let crit_eff = if self_crit_eff < 1.0 {1.0} else {self_crit_eff};
        let crit_damage = (damage as f64 * crit_eff) as u64;

        let mut rng = rand::thread_rng();
        let other_body_part = match rng.gen_range(0..6) {
            0 => other.hands?,
            1 => other.hands?,
            2 => other.legs?,
            3 => other.legs?,
            4 => other.torso?,
            _ => other.head?,
        };
        
        let flat_damage_reduction = (other_body_part.protection.unwrap_or(0) as f64 * (1.0 - self_armor_pen)) as u64;
        let percent_damage_reduction = other.damage_taken.unwrap_or(1.0) * (1.0 - other_body_part.phy_res.unwrap_or(0.0)) * 
        (1.0 - match self.damage_type? {
            DamageType::slash => other_body_part.slash_res.unwrap_or(0.0),
            DamageType::pierc => other_body_part.pierc_res.unwrap_or(0.0),
            DamageType::crush => other_body_part.crush_res.unwrap_or(0.0),
            DamageType::rend => other_body_part.rend_res.unwrap_or(0.0),
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

        Some(Hit::new(chance, dmg))
    }

    pub fn residual_damage(&self) -> Option<f64> {
        Some(self.percent_damage_receive.unwrap_or(0.0) * self.hp? as f64 + self.flat_damage_receive.unwrap_or(0) as f64)
    }
}

impl Add for Stat {
    type Output = Self;

    fn add(self, other: Stat) -> Stat {
        Stat{
            hp: self.hp.add(other.hp),
            damage: self.damage.add(other.damage),
            damage_type: self.damage_type,
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
        }
    }
}

impl AddAssign for Stat {
    fn add_assign(&mut self, other: Stat) {
        let add_self: Stat = *self + other; 
        *self = add_self
    }
}