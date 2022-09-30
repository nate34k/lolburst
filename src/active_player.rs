#[derive(Default)]
pub struct AbilityRanks {
    pub q_rank: i64,
    pub w_rank: i64,
    pub e_rank: i64,
    pub r_rank: i64,
}

impl AbilityRanks {
    pub fn new(q_rank: i64, w_rank: i64, e_rank: i64, r_rank: i64) -> Self {
        AbilityRanks {
            q_rank,
            w_rank,
            e_rank,
            r_rank,
        }
    }
}
