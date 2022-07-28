use crate::{AbilityRanks, active_player};

#[derive(Debug)]
pub struct Orianna {
    pub name: String,
    stats: OriannaStats,
}

impl Orianna {
    fn new(name: String, stats: OriannaStats) -> Self {
        Orianna { name, stats }
    }

    pub fn build() -> Orianna {
        Orianna::new(String::from("Orianna"),
            OriannaStats::new(
                vec![0.0,60.0,90.0,120.0,150.0,180.0,0.5], 
                vec![0.0,60.0,105.0,150.0,195.0,240.0,0.7],
                vec![0.0,60.0,90.0,120.0,150.0,180.0,0.3],
                vec![0.0,200.0,275.0,350.0,0.8],
                vec![10.0,18.0,26.0,34.0,42.0,50.0,0.15,1.2]),)
    }

    pub fn calculate_rd(&self, active_player: &active_player::Root, abilityranks: &AbilityRanks) -> OriannaRawDamage {
        let q_rank = abilityranks.q_rank;
        let w_rank = abilityranks.w_rank;
        let e_rank = abilityranks.e_rank;
        let r_rank = abilityranks.r_rank;
        let ap = active_player.champion_stats.ability_power;
        let ad = active_player.champion_stats.attack_damage;
        let level = active_player.level;
        OriannaRawDamage {
            q: (&self.stats.q_dmg[q_rank as usize]) + (&self.stats.q_dmg[6] * ap),
            w: (&self.stats.w_dmg[w_rank as usize]) + (&self.stats.w_dmg[6] * ap),
            e: (&self.stats.e_dmg[e_rank as usize]) + (&self.stats.e_dmg[6] * ap),
            r: (&self.stats.r_dmg[r_rank as usize]) + (&self.stats.r_dmg[6] * ap),
            p: self.stats.p_dmg[(((level - 1) / 3) as f64).floor() as usize],
            aa: ad,
        }
    }
}

#[derive(Debug)]
struct OriannaStats {
    q_dmg: Vec<f64>,
    w_dmg: Vec<f64>,
    e_dmg: Vec<f64>,
    r_dmg: Vec<f64>,
    p_dmg: Vec<f64>,
}

impl OriannaStats {
    fn new(q_dmg: Vec<f64>, w_dmg: Vec<f64>, e_dmg: Vec<f64>, r_dmg: Vec<f64>, p_dmg: Vec<f64>) -> Self {
        OriannaStats { q_dmg, w_dmg, e_dmg, r_dmg, p_dmg }
    }
}

struct OriannaRawDamage {
    q: f64,
    w: f64,
    e: f64,
    r: f64,
    p: f64,
    aa: f64,
}