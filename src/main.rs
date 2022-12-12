use serde::{Serialize, Deserialize};
use serde_yaml;
use std::fmt;
use std::ops::{Add, Sub, Mul, AddAssign};
use std::{fs::File, collections::HashMap};

trait CustomAdd<T = Self> {
    type Output;
    fn add(self, rhs: T) -> Self::Output;
}

trait CustomSub<T = Self> {
    type Output;
    fn sub(self, rhs: T) -> Self::Output;
}

trait CustomMul<T = Self> {
    type Output;
    fn mul(self, rhs: T) -> Self::Output;
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

impl<T: Sub<Output = T>> CustomSub for Option<T> {
    type Output = Self;
    fn sub(self, other: Option<T>) -> Option<T> {
        match (self, other) {
            (Some(i), Some(j)) => Some(i - j),
            (Some(i), None) => Some(i),
            (None, Some(j)) => Some(j),
            (None, None) => None,
        }
    }
}

impl<T: Mul<Output = T>> CustomMul for Option<T> {
    type Output = Self;
    fn mul(self, other: Option<T>) -> Option<T> {
        match (self, other) {
            (Some(i), Some(j)) => Some(i * j),
            (Some(i), None) => Some(i),
            (None, Some(j)) => Some(j),
            (None, None) => None,
        }
    }
}

#[derive(PartialEq)]
struct Hit {
    crit_hit_chance: f64,
    normal_hit_chance : f64,
    half_hit_chance : f64,
    block_crit_hit_chance : f64,
    block_normal_hit_chance : f64,
    block_half_hit_chance : f64,

    half_hit_damage: u64,
    normal_hit_damage: u64,
    crit_hit_damage: u64,
    block_half_hit_damage: u64,
    block_normal_hit_damage: u64,
    block_crit_hit_damage: u64,
}

impl Hit {
    fn expected_damage(&self) -> f64 {
        self.half_hit_chance * self.half_hit_damage as f64 + self.normal_hit_chance * self.normal_hit_damage as f64 + 
        self.crit_hit_chance * self.crit_hit_damage  as f64 + self.block_half_hit_chance * self.block_half_hit_damage as f64 + 
        self.block_normal_hit_chance * self.block_normal_hit_damage as f64 + self.block_crit_hit_chance * self.block_crit_hit_damage  as f64 
    }
}

impl fmt::Debug for Hit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "crit {}: {},
        normal {}: {},
        half {}: {},
        block_crit {}: {},
        block_normal {}: {},
        bloc_half {}: {}]", self.crit_hit_chance, self.crit_hit_damage,
        self.normal_hit_chance, self.normal_hit_damage,
        self.half_hit_chance, self.half_hit_damage,
        self.block_crit_hit_chance, self.block_crit_hit_damage,
        self.block_normal_hit_chance, self.block_normal_hit_damage,
        self.block_half_hit_chance, self.block_half_hit_damage)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
struct Stat {
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Skill {
    turn: u64,
    effect: Stat,
}

#[derive(PartialEq, Debug)]
struct Char<'a> {
    stat: Stat,
    skills: Vec<&'a Skill>,
}

impl Stat {
    fn attack(&self, other: &Stat, is_counter: bool) -> Option<Hit>{
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

impl Char<'_> {
    fn compute(&self)-> Stat {
        let mut raw_stat: Stat = self.stat;
        for s in self.skills.iter() {
            raw_stat += s.effect;
        }
        raw_stat
    }

    fn remove_outdated_skills(&mut self, turn: &u64) {
        self.skills.retain(|x| x.turn > *turn || x.turn == 0);
    }
}

fn expected_damage_cycle_attack_via_stat(first :&Stat, second:&Stat) -> [f64; 2] {
    let damage_first: f64 = first.attack(second, false).unwrap().expected_damage() + 
    first.attack(second, true).unwrap().expected_damage();
    let damage_second: f64 = second.attack(first, false).unwrap().expected_damage() + 
    second.attack(first, true).unwrap().expected_damage();

    [damage_first, damage_second]
}

fn expected_damage_n_cycles(first :&mut Char, second:&mut Char, n :u64) -> Option<[u64; 3]> {
    let mut hp_first = first.stat.hp?;
    let mut hp_second = second.stat.hp?;
    let mut count: u64 = 0;
    for _ in 0..n {
        let [damage_first, damage_second] = expected_damage_cycle_attack_via_stat(&first.compute(), &second.compute());
        hp_first = if damage_second as u64 > hp_first { 0 } else { hp_first - damage_second as u64 };
        hp_second = if damage_first as u64 > hp_second { 0 } else { hp_second - damage_first as u64 };
        count += 1;
        first.remove_outdated_skills(&count);
        second.remove_outdated_skills(&count);

        if hp_first == 0 || hp_second == 0 {
            break;
        }
    }

    Some([hp_first, hp_second, count])
}

fn main() -> Result<(), serde_yaml::Error> {

    let path_enemies: &str = "./data/enemies.yaml";
    let path_chars: &str = "./data/characters.yaml";
    let path_effects: &str = "./data/effects.yaml";
    let path_action: &str = "./data/action.yaml";

    let file_enemies = File::open(path_enemies).expect("Unable to open file");
    let file_chars = File::open(path_chars).expect("Unable to open file");
    let file_effects = File::open(path_effects).expect("Unable to open file");
    let file_action = File::open(path_action).expect("Unable to open file");

    let deserialized_enemies: HashMap<String, Stat> = serde_yaml::from_reader(&file_enemies).unwrap();
    let deserialized_chars: HashMap<String, Stat> = serde_yaml::from_reader(&file_chars).unwrap();
    let deserialized_effects: HashMap<String, Skill> = serde_yaml::from_reader(&file_effects).unwrap();
    let deserialized_action: HashMap<String, Vec<String>> = serde_yaml::from_reader(&file_action).unwrap();

    let mut raw_bear: Char = Char { stat: deserialized_enemies["bear"], skills: Vec::<&Skill>::new(), };
    let mut raw_main: Char = Char { stat: deserialized_chars["main"], skills: Vec::<&Skill>::new(), };

    let mut buf_bear: Char = Char { stat: deserialized_enemies["bear"], skills: Vec::<&Skill>::new(), };
    let mut buf_main: Char = Char { stat: deserialized_chars["main"], skills: Vec::<&Skill>::new(), };

    for s in deserialized_action["other"].iter() {
        buf_bear.skills.push(&deserialized_effects[s]);
    }
    for s in deserialized_action["self"].iter() {
        buf_main.skills.push(&deserialized_effects[s]);
    }

    let max_turn: u64 = 100;
    let raw_expectation = expected_damage_n_cycles(&mut raw_bear, &mut raw_main, max_turn);
    println!("{:?}", raw_expectation);
    let after_buf_expectation = expected_damage_n_cycles(&mut buf_bear, &mut buf_main, max_turn);
    println!("{:?}", after_buf_expectation);

    Ok(()) 

    /*I used the formula and data I found on the wiki. Right now it only simulate a "static" combat with only passives and buffs (nor actives nor secondary effects as bleed) */
}