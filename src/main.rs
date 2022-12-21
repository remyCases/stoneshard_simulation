mod stat;
mod hit;

use serde::{Serialize, Deserialize};
use serde_yaml;
use std::{fs::File, collections::HashMap, ops::{AddAssign, Add}};
use stat::{Stat, IdSkills};
use hit::{Hit, HitType, BodyPart};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Skill {
    id: IdSkills,
    turn: u64,
    effect: Stat,
}

#[derive(PartialEq, Debug)]
struct Char<'a> {
    stat: Stat,
    skills: HashMap<IdSkills, &'a Skill>,
}

impl<'a> Char<'a> {
    fn clone(&self) -> Self {
        Char {
            stat: self.stat.clone(),
            skills: self.skills.clone(),
        }
    }

    fn compute(&self)-> Stat {
        let mut raw_stat: Stat = self.stat.clone();
        for (_, s) in self.skills.iter() {
            raw_stat += s.effect.clone();
        }
        raw_stat
    }

    fn remove_outdated_skills(&mut self, turn: &u64) {
        self.skills.retain(|_, x| x.turn > *turn || x.turn == 0);
    }

    fn add_skill(&mut self, skill: &'a Skill) {
        self.skills.insert(skill.id, skill);
    }

    fn resolve_hit(&self, other: &mut Char<'a>, skills_map: &'a HashMap<IdSkills, Skill>, bodypart_hit :BodyPart, is_crit: bool) {
        let hm = self.stat.additional_effect( &other.stat, bodypart_hit, is_crit);
        for (s, b) in hm.iter() {
            if *b {
                other.add_skill(&skills_map[s]);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResultSimulation {
    first_hp_at_end: u64,
    second_hp_at_end: u64,
    turn: u64,
}

impl Add for ResultSimulation {
    type Output = ResultSimulation;

    fn add(self, rhs: Self) -> Self::Output {
        ResultSimulation {
            first_hp_at_end: self.first_hp_at_end + rhs.first_hp_at_end,
            second_hp_at_end: self.second_hp_at_end + rhs.second_hp_at_end,
            turn: self.turn + rhs.turn,
        }
    }
}

impl AddAssign for ResultSimulation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Debug)]
struct StatSimu {
    mean: f64,
    var: f64,
    n: u64,
}

impl StatSimu {
    fn confident_interval(&self) -> [f64; 3] {
        let factor: f64 = f64::sqrt(self.var / self.n as f64) * 1.96;
        [self.mean - factor, self.mean, self.mean + factor]
    }
}

fn simulate_damage_cycle_attack_via_stat<'a>(
    first: &mut Char<'a>, 
    second: &mut Char<'a>, 
    block_first: u64, 
    block_second: u64, 
    skills_map: &'a HashMap<IdSkills, Skill>
) -> Option<[f64; 4]> 
{
    let first_stat = &first.compute();
    let second_stat = &second.compute();
    let hit_first: Hit = first_stat.attack(second_stat);
    let hit_second: Hit = second_stat.attack(first_stat);

    let first_hit_type: HitType = hit_first.draw(None);
    let (first_dmg, second_dmg_block) = first_stat.get_damage(
        second_stat, 
        hit_first.get_bodypart_hit(), 
        first_hit_type, 
        block_second
    );
    match first_hit_type {
        HitType::CritHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), true),
        HitType::NormalHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), false),
        HitType::BlockCritHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), true),
        HitType::BlockNormalHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), false),
        _ => (),
    };

    let second_counter_hit_type: HitType = hit_second.draw(second_stat.get_counter());
    let (second_counter_dmg, first_counter_dmg_block) = second_stat.get_damage(
        first_stat, 
        hit_second.get_bodypart_hit(), 
        second_counter_hit_type, 
        block_first
    );
    match second_counter_hit_type {
        HitType::CritHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), true),
        HitType::NormalHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), false),
        HitType::BlockCritHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), true),
        HitType::BlockNormalHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), false),
        _ => (),
    };

    let second_hit_type: HitType = hit_second.draw(None);
    let (second_dmg, first_dmg_block) = second_stat.get_damage(
        first_stat, 
        hit_second.get_bodypart_hit(), 
        second_hit_type, 
        block_first - first_counter_dmg_block as u64
    );
    match second_hit_type {
        HitType::CritHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), true),
        HitType::NormalHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), false),
        HitType::BlockCritHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), true),
        HitType::BlockNormalHit => second.resolve_hit(first, skills_map, hit_second.get_bodypart_hit(), false),
        _ => (),
    };

    let first_counter_hit_type: HitType = hit_first.draw(first_stat.get_counter());
    let (first_counter_dmg, second_counter_dmg_block) = first_stat.get_damage(
        second_stat, 
        hit_first.get_bodypart_hit(), 
        first_hit_type, 
        block_second - second_dmg_block as u64
    );
    match first_counter_hit_type {
        HitType::CritHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), true),
        HitType::NormalHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), false),
        HitType::BlockCritHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), true),
        HitType::BlockNormalHit => first.resolve_hit(second, skills_map, hit_first.get_bodypart_hit(), false),
        _ => (),
    };
    
    Some([
        first_dmg + first_counter_dmg + second_stat.residual_damage(), 
        second_dmg + second_counter_dmg + first_stat.residual_damage(),
        first_dmg_block + first_counter_dmg_block,
        second_dmg_block + second_counter_dmg_block,
    ])
}

fn simulate_damage_n_cycles<'a>(
    first :& mut Char<'a>, 
    second:& mut Char<'a>, 
    n :u64,
    skills_map: &'a HashMap<IdSkills, Skill>
) -> Option<ResultSimulation> 
{
    let mut hp_first = first.stat.get_hp()?;
    let mut hp_second = second.stat.get_hp()?;
    let mut block_first = first.stat.get_block()?;
    let mut block_second = second.stat.get_block()?;
    let mut count: u64 = 0;
    for _ in 0..n {
        let [
            damage_first, damage_second, 
            damage_block_first, damage_block_second
        ] = simulate_damage_cycle_attack_via_stat(
            first, second, block_first, block_second, skills_map
        )?;
        hp_first = if damage_second as u64 > hp_first { 0 } else { hp_first - damage_second as u64 };
        hp_second = if damage_first as u64 > hp_second { 0 } else { hp_second - damage_first as u64 };
        block_first = if damage_block_second as u64 > block_first { 0 } else { block_first - damage_block_second as u64 };
        block_second = if damage_block_first as u64 > block_second { 0 } else { block_second - damage_block_first as u64 };
        
        count += 1;
        first.remove_outdated_skills(&count);
        second.remove_outdated_skills(&count);

        if hp_first == 0 || hp_second == 0 {
            break;
        }

        block_first += (block_first as f64 * first.stat.get_block_recovery().unwrap_or(0.0)) as u64;
        if block_first > first.stat.get_block()? { block_first = first.stat.get_block()?}
        block_second += (block_second as f64 * second.stat.get_block_recovery().unwrap_or(0.0)) as u64;
        if block_second > second.stat.get_block()? { block_second = second.stat.get_block()?}
    }

    Some(ResultSimulation { 
        first_hp_at_end: hp_first,
        second_hp_at_end: hp_second,
        turn: count, }
    )
}

fn monte_carlo_damage<'a>(
    first_data: &Char, 
    second_data: &Char, 
    n: u64,
    skills_map: &'a HashMap<IdSkills, Skill>
) -> Option<[StatSimu; 3]> 
{
    let mut sum_win: u64 = 0;
    let mut sum_hp_first: u64 = 0;
    let mut sum_hp_second: u64 = 0;
    let mut sumsq_hp_first: u64 = 0;
    let mut sumsq_hp_second: u64 = 0;
    let n_simu: u64 = 10000;

    for _ in 0..n_simu {
        let mut first = first_data.clone();
        let mut second = second_data.clone();
        let result_simulation = simulate_damage_n_cycles(
            &mut first, 
            &mut second, 
            n,
            skills_map)?;
        sum_win += if result_simulation.first_hp_at_end > 0 {1} else {0};
        sum_hp_first += result_simulation.first_hp_at_end;
        sumsq_hp_first += result_simulation.first_hp_at_end * result_simulation.first_hp_at_end;
        sum_hp_second += result_simulation.second_hp_at_end;
        sumsq_hp_second += result_simulation.second_hp_at_end* result_simulation.second_hp_at_end;
    }

    let mean_win: f64 = sum_win as f64/ n_simu as f64;
    let mean_hp_first: f64 = sum_hp_first as f64/ n_simu as f64;
    let mean_hp_second: f64 = sum_hp_second as f64/ n_simu as f64;
    Some([
        StatSimu{
            mean: mean_win,
            var: mean_win - mean_win * mean_win,
            n: n_simu,
        }, 
        StatSimu{
            mean: mean_hp_first,
            var: sumsq_hp_first as f64/ n_simu as f64 - mean_hp_first * mean_hp_first,
            n: n_simu,
        },
        StatSimu{
            mean: mean_hp_second,
            var: sumsq_hp_second as f64/ n_simu as f64 - mean_hp_second * mean_hp_second,
            n: n_simu,
        }])
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
    let deserialized_effects: HashMap<IdSkills, Skill> = serde_yaml::from_reader(&file_effects).unwrap();
    let deserialized_action: HashMap<String, Vec<IdSkills>> = serde_yaml::from_reader(&file_action).unwrap();

    let mut ref_bear: Char = Char { 
        stat: deserialized_enemies["crawler"].clone(), skills: HashMap::<IdSkills, &Skill>::new(), 
    };
    let mut ref_main: Char = Char { 
        stat: deserialized_chars["main_rot"].clone(), skills: HashMap::<IdSkills, &Skill>::new(), 
    };

    let mut buf_bear: Char = Char { 
        stat: deserialized_enemies["crawler"].clone(), skills: HashMap::<IdSkills, &Skill>::new(), 
    };
    let mut buf_main: Char = Char { 
        stat: deserialized_chars["main"].clone(), skills: HashMap::<IdSkills, &Skill>::new(), 
    };

    for s in deserialized_action["other_ref"].iter() {
        ref_bear.skills.insert(deserialized_effects[s].id, &deserialized_effects[s]);
    }
    for s in deserialized_action["self_ref"].iter() {
        ref_main.skills.insert(deserialized_effects[s].id, &deserialized_effects[s]);
    }
    for s in deserialized_action["other"].iter() {
        buf_bear.skills.insert(deserialized_effects[s].id, &deserialized_effects[s]);
    }
    for s in deserialized_action["self"].iter() {
        buf_main.skills.insert(deserialized_effects[s].id, &deserialized_effects[s]);
    }

    let max_turn: u64 = 100;
    let raw_expectation = monte_carlo_damage(
        &mut ref_bear, 
        &mut ref_main, 
        max_turn, 
        &deserialized_effects
    );
    let unwrap_raw = raw_expectation.unwrap();
    for i in 0..3 {
        println!("{:?}", unwrap_raw[i].confident_interval());
    }

    let after_buf_expectation = monte_carlo_damage(
        &mut buf_bear, 
        &mut buf_main, 
        max_turn, 
        &deserialized_effects
    );
    let unwrap_buf = after_buf_expectation.unwrap();
    for i in 0..3 {
        println!("{:?}", unwrap_buf[i].confident_interval());
    }

    Ok(()) 
}