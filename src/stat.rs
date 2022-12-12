use std::ops::{Add, AddAssign};
use serde::{Serialize, Deserialize};
use crate::hit::Hit;

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
pub struct Stat {
    hp: Option<u64>,
    protection: Option<u64>,
    weapon_dmg: Option<f64>,
    accuracy: Option<f64>,
    fumble: Option<f64>,
    crit_chance: Option<f64>,
    crit_eff: Option<f64>,
    armor_pen: Option<f64>,
    counter: Option<f64>,
    dodge: Option<f64>,
    block: Option<f64>,
    damage: Option<u64>,
    block_value: Option<u64>,
}

impl Stat {
    pub fn get_hp(&self) -> Option<u64> {
        self.hp
    }

    pub fn attack(&self, other: &Stat, is_counter: bool) -> Option<Hit>{
        let self_accuracy = self.accuracy?;
        let self_fumble = self.fumble?;
        let self_crit_chance = self.crit_chance?;
        let self_crit_eff = self.crit_eff?;
        let self_weapon_dmg = self.weapon_dmg?;
        let self_counter = if is_counter {self.counter?} else {1.0};

        let self_damage = self.damage?;
        
        let other_dodge = other.dodge?;
        let other_block = other.block?;
        let other_block_value = other.block_value?;

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

        let damage = (self_weapon_dmg * self_damage as f64) as u64;

        let half_hit = accuracy * (1.0 - fumble) * dodge + accuracy * fumble * (1.0 - dodge);
        let normal_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * (1.0 - self_crit_chance);
        let crit_hit = accuracy * (1.0 - fumble) * (1.0 - dodge) * self_crit_chance;

        let crit_damage = (damage as f64 * self_crit_eff) as u64;

        Some(Hit {
            crit_hit_chance: crit_hit * (1.0 - other_block) * self_counter,
            normal_hit_chance : normal_hit * (1.0 - other_block) * self_counter,
            half_hit_chance : half_hit * (1.0 - other_block) * self_counter,
            block_crit_hit_chance : crit_hit * other_block * self_counter,
            block_normal_hit_chance : normal_hit * other_block * self_counter,
            block_half_hit_chance : half_hit * other_block * self_counter,

            crit_hit_damage: crit_damage,
            normal_hit_damage: damage,
            half_hit_damage: damage / 2,
            block_crit_hit_damage: if crit_damage > other_block_value {crit_damage - other_block_value} else {0},
            block_normal_hit_damage: if damage > other_block_value {damage - other_block_value} else {0},
            block_half_hit_damage: if damage / 2 > other_block_value {damage / 2 - other_block_value} else {0},
        })

    }
}

impl Add for Stat {
    type Output = Self;

    fn add(self, other: Stat) -> Stat {
        Stat{
            hp: self.hp.add(other.hp),
            protection: self.protection.add(other.protection),
            weapon_dmg: self.weapon_dmg.add(other.weapon_dmg),
            accuracy: self.accuracy.add(other.accuracy),
            fumble: self.fumble.add(other.fumble),
            crit_chance: self.crit_chance.add(other.crit_chance),
            crit_eff: self.crit_eff.add(other.crit_eff),
            armor_pen: self.armor_pen.add(other.armor_pen),
            counter: self.counter.add(other.counter),
            dodge: self.dodge.add(other.dodge),
            block: self.block.add(other.block),
            damage: self.damage.add(other.damage),
            block_value: self.block_value.add(other.block_value),
        }
    }
}

impl AddAssign for Stat {
    fn add_assign(&mut self, other: Stat) {
        let add_self: Stat = *self + other; 
        *self = add_self
    }
}