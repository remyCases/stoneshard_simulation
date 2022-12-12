use std::fmt;

#[derive(PartialEq)]
pub struct Hit {
    pub crit_hit_chance: f64,
    pub normal_hit_chance : f64,
    pub half_hit_chance : f64,
    pub block_crit_hit_chance : f64,
    pub block_normal_hit_chance : f64,
    pub block_half_hit_chance : f64,

    pub half_hit_damage: u64,
    pub normal_hit_damage: u64,
    pub crit_hit_damage: u64,
    pub block_half_hit_damage: u64,
    pub block_normal_hit_damage: u64,
    pub block_crit_hit_damage: u64,
}

impl Hit {
    pub fn expected_damage(&self) -> f64 {
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