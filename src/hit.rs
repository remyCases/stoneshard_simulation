use rand::{self, Rng};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum HitType {
    CritHit,
    NormalHit,
    HalfHit,
    BlockCritHit,
    BlockNormalHit,
    BlockHalfHit,
    NoHit,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BodyPart {
    RightLeg,
    LeftLeg,
    RightHand,
    LeftHand,
    Torso,
    Head,
    None,
}

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
    pub fn new(
        crit_hit: f64, 
        normal_hit: f64, 
        half_hit: f64, 
        block_crit_hit: f64, 
        block_normal_hit: f64, 
        block_half_hit: f64
    ) -> Self {
            Chance {
                crit_hit,
                normal_hit,
                half_hit,
                block_crit_hit,
                block_normal_hit,
                block_half_hit,
            }
    }

    pub fn draw(&self, input_proba: Option<f64>) -> HitType {
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..1.0);

        let added_proba = match input_proba {
            Some(c) => c,
            None => 1.0,
        };

        if self.crit_hit > random_value / added_proba {
            HitType::CritHit
        } else if self.crit_hit + self.normal_hit > random_value / added_proba {
            HitType::NormalHit
        } else if self.crit_hit + self.normal_hit + self.half_hit > random_value / added_proba {
            HitType::HalfHit
        } else if self.crit_hit + self.normal_hit + self.half_hit + self.block_crit_hit > random_value / added_proba {
            HitType::BlockCritHit
        } else if self.crit_hit + self.normal_hit + self.half_hit + self.block_crit_hit + self.block_normal_hit > random_value / added_proba {
            HitType::BlockNormalHit
        } else if self.crit_hit + self.normal_hit + self.half_hit + self.block_crit_hit + self.block_normal_hit + self.block_half_hit > random_value / added_proba {
            HitType::BlockHalfHit
        } else {
            HitType::NoHit
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
    pub fn new(
        crit_hit: u64, 
        normal_hit: u64, 
        half_hit: u64, 
        block_crit_hit: u64, 
        block_normal_hit: u64, 
        block_half_hit: u64
    ) -> Self {
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

    fn get(&self, h :HitType) -> u64 {
        match h {
            HitType::CritHit => self.crit_hit,
            HitType::NormalHit => self.normal_hit,
            HitType::HalfHit => self.half_hit,
            HitType::BlockCritHit => self.block_crit_hit,
            HitType::BlockNormalHit => self.block_normal_hit,
            HitType::BlockHalfHit => self.block_half_hit,
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
    body_part: BodyPart,
}

impl Hit {
    pub fn new(chance: Chance, damage: Damage, body_part: BodyPart) -> Self {
            Hit {
                chance,
                damage,
                body_part,
            }
    }

    pub fn simulate_damage(&self, added_proba: Option<f64>) -> (f64, HitType) {
        let hit_type = self.chance.draw(added_proba);
        let hit_damage = self.damage.get(hit_type);
        (hit_damage as f64, hit_type)
    }

    pub fn get_chance(&self) -> Chance {
        self.chance
    }
    pub fn get_damage(&self) -> Damage {
        self.damage
    }
    pub fn get_bodypart_hit(&self) -> BodyPart {
        self.body_part
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chance_crit() {
        let chance = Chance::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::CritHit);
        }
    }
    #[test]
    fn test_chance_normal() {
        let chance = Chance::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::NormalHit);
        }
    }
    #[test]
    fn test_chance_half() {
        let chance = Chance::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::HalfHit);
        }
    }
    #[test]
    fn test_chance_blockcrit() {
        let chance = Chance::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::BlockCritHit);
        }
    }
    #[test]
    fn test_chance_blocknormal() {
        let chance = Chance::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::BlockNormalHit);
        }
    }
    #[test]
    fn test_chance_blockhalf() {
        let chance = Chance::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::BlockHalfHit);
        }
    }
    #[test]
    fn test_chance_nohit() {
        let chance = Chance::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..100 {
            assert_eq!(chance.draw(None), HitType::NoHit);
        }
    }

    #[test]
    fn test_chance_no_crit() {
        let chance = Chance::new(0.0, 0.1, 0.1, 0.1, 0.1, 0.1);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::CritHit);
        }
    }
    #[test]
    fn test_chance_no_normal() {
        let chance = Chance::new(0.1, 0.0, 0.1, 0.1, 0.1, 0.1);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::NormalHit);
        }
    }
    #[test]
    fn test_chance_no_half() {
        let chance = Chance::new(0.1, 0.1, 0.0, 0.1, 0.1, 0.1);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::HalfHit);
        }
    }
    #[test]
    fn test_chance_no_blockcrit() {
        let chance = Chance::new(0.1, 0.1, 0.1, 0.0, 0.1, 0.1);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::BlockCritHit);
        }
    }
    #[test]
    fn test_chance_no_blocknormal() {
        let chance = Chance::new(0.1, 0.1, 0.1, 0.1, 0.0, 0.1);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::BlockNormalHit);
        }
    }
    #[test]
    fn test_chance_no_blockhalf() {
        let chance = Chance::new(0.1, 0.1, 0.1, 0.1, 0.1, 0.0);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::BlockHalfHit);
        }
    }
    #[test]
    fn test_chance_no_nohit() {
        let chance = Chance::new(0.1, 0.1, 0.1, 0.1, 0.3, 0.3);
        for _ in 0..100 {
            assert_ne!(chance.draw(None), HitType::NoHit);
        }
    }
}