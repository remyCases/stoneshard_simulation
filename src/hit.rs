use rand::{self, Rng};
use std::ops::ControlFlow;
use std::iter::zip;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Chance {
    crit_hit: f64,
    normal_hit: f64,
    half_hit: f64,
    block_crit_hit: f64,
    block_normal_hit: f64,
    block_half_hit: f64,
}

impl Chance {
    pub fn new(crit_hit: f64, normal_hit: f64, half_hit: f64, 
        block_crit_hit: f64, block_normal_hit: f64, block_half_hit: f64) -> Self {
            Chance {
                crit_hit,
                normal_hit,
                half_hit,
                block_crit_hit,
                block_normal_hit,
                block_half_hit,
            }
    }
}

impl IntoIterator for Chance {
    type Item = f64;
    type IntoIter = std::array::IntoIter<f64, 6>;

    fn into_iter(self) -> Self::IntoIter {
        [self.crit_hit, 
        self.normal_hit, 
        self.half_hit, 
        self.block_crit_hit, 
        self.block_normal_hit, 
        self.block_half_hit].into_iter()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Damage {
    crit_hit: u64,
    normal_hit: u64,
    half_hit: u64,
    block_crit_hit: u64,
    block_normal_hit: u64,
    block_half_hit: u64,
}

impl Damage {
    pub fn new(crit_hit: u64, normal_hit: u64, half_hit: u64, 
        block_crit_hit: u64, block_normal_hit: u64, block_half_hit: u64) -> Self {
            Damage {
                crit_hit,
                normal_hit,
                half_hit,
                block_crit_hit,
                block_normal_hit,
                block_half_hit,
            }
    }

    fn into_array(&self) -> [u64; 6] {
        [self.crit_hit, 
        self.normal_hit, 
        self.half_hit, 
        self.block_crit_hit, 
        self.block_normal_hit, 
        self.block_half_hit]
    }

    fn get(&self, n :usize) -> u64 {
        match n {
            0 => self.crit_hit,
            1 => self.normal_hit,
            2 => self.half_hit,
            3 => self.block_crit_hit,
            4 => self.block_normal_hit,
            5 => self.block_half_hit,
            _ => 0,
        }
    }
}
impl IntoIterator for Damage {
    type Item = u64;
    type IntoIter = std::array::IntoIter<u64, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_array().into_iter()
    }
}

#[derive(PartialEq, Debug)]
pub struct Hit {
    chance: Chance,
    damage: Damage,
}

impl Hit {
    pub fn new(chance: Chance, damage: Damage) -> Self {
            Hit {
                chance,
                damage,
            }
    }

    pub fn simulate_damage(&self, added_proba: Option<f64>) -> f64 {
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..1.0);
        let proba = match added_proba {
            Some(c) => c,
            None => 1.0,
        };

        let choose_hit = self.chance.into_iter().enumerate().try_fold(0.0, |acc, (n, x)| {
            if (acc + proba * x) > random_value {
                ControlFlow::Break(n)
            } else {
                ControlFlow::Continue(acc + proba * x)
            }
        });

        match choose_hit {
            ControlFlow::Break(n) => self.damage.get(n) as f64,
            ControlFlow::Continue(_) => 0.0,
        }
    }

    pub fn expected_damage(&self, added_proba: Option<f64>) -> f64 {
        let proba = match added_proba {
            Some(c) => c,
            None => 1.0,
        };
        zip(self.chance, self.damage).fold(0.0, |acc, (x, y)| acc + x * proba * y as f64)
    }
}