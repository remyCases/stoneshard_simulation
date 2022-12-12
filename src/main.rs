mod stat;
mod hit;

use serde::{Serialize, Deserialize};
use serde_yaml;
use std::{fs::File, collections::HashMap};
use stat::Stat;

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

#[derive(Debug)]
struct ResultSimulation {
    first_hp_at_end: u64,
    second_hp_at_end: u64,
    turn: u64,
}

fn expected_damage_cycle_attack_via_stat(first :&Stat, second:&Stat) -> [f64; 2] {
    let damage_first: f64 = first.attack(second, false).unwrap().expected_damage() + 
    first.attack(second, true).unwrap().expected_damage();
    let damage_second: f64 = second.attack(first, false).unwrap().expected_damage() + 
    second.attack(first, true).unwrap().expected_damage();

    [damage_first, damage_second]
}

fn expected_damage_n_cycles(first :&mut Char, second:&mut Char, n :u64) -> Option<ResultSimulation> {
    let mut hp_first = first.stat.get_hp()?;
    let mut hp_second = second.stat.get_hp()?;
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

    Some(ResultSimulation { 
        first_hp_at_end: hp_first,
        second_hp_at_end: hp_second,
        turn: count, }
    )
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

    let mut ref_bear: Char = Char { stat: deserialized_enemies["bear"], skills: Vec::<&Skill>::new(), };
    let mut ref_main: Char = Char { stat: deserialized_chars["main"], skills: Vec::<&Skill>::new(), };

    let mut buf_bear: Char = Char { stat: deserialized_enemies["bear"], skills: Vec::<&Skill>::new(), };
    let mut buf_main: Char = Char { stat: deserialized_chars["main"], skills: Vec::<&Skill>::new(), };

    for s in deserialized_action["other_ref"].iter() {
        ref_bear.skills.push(&deserialized_effects[s]);
    }
    for s in deserialized_action["self_ref"].iter() {
        ref_main.skills.push(&deserialized_effects[s]);
    }
    for s in deserialized_action["other"].iter() {
        buf_bear.skills.push(&deserialized_effects[s]);
    }
    for s in deserialized_action["self"].iter() {
        buf_main.skills.push(&deserialized_effects[s]);
    }

    let max_turn: u64 = 100;
    let raw_expectation = expected_damage_n_cycles(&mut ref_bear, &mut ref_main, max_turn);
    println!("{:?}", raw_expectation);
    let after_buf_expectation = expected_damage_n_cycles(&mut buf_bear, &mut buf_main, max_turn);
    println!("{:?}", after_buf_expectation);

    Ok(()) 

    /*I used the formula and data I found on the wiki. Right now it only simulate a "static" combat with only passives and buffs (nor actives nor secondary effects as bleed) */
}