use crate::{
    AbilityRanks,
    active_player,
    dmg,};

#[derive(Debug)]
pub struct Orianna {
    pub name: String,
    stats: Stats,
}

impl Orianna {
    fn new(name: String, stats: Stats) -> Self {
        Orianna { name, stats }
    }

    pub fn build() -> Orianna {
        Orianna::new(String::from("Orianna"),
            Stats::new(
                vec![0.0,60.0,90.0,120.0,150.0,180.0,0.5], 
                vec![0.0,60.0,105.0,150.0,195.0,240.0,0.7],
                vec![0.0,60.0,90.0,120.0,150.0,180.0,0.3],
                vec![0.0,200.0,275.0,350.0,0.8],
                vec![10.0,18.0,26.0,34.0,42.0,50.0,0.15,1.2]),)
    }

    pub fn calculate_rd(&self, active_player: &active_player::Root, abilityranks: &AbilityRanks) -> RawDamage {
        let q_rank = abilityranks.q_rank;
        let w_rank = abilityranks.w_rank;
        let e_rank = abilityranks.e_rank;
        let r_rank = abilityranks.r_rank;
        let ap = active_player.champion_stats.ability_power;
        let ad = active_player.champion_stats.attack_damage;
        let level = active_player.level;
        RawDamage {
            q: (&self.stats.q_dmg[q_rank as usize]) + (&self.stats.q_dmg[6] * ap),
            w: (&self.stats.w_dmg[w_rank as usize]) + (&self.stats.w_dmg[6] * ap),
            e: (&self.stats.e_dmg[e_rank as usize]) + (&self.stats.e_dmg[6] * ap),
            r: (&self.stats.r_dmg[r_rank as usize]) + (&self.stats.r_dmg[4] * ap),
            p: self.stats.p_dmg[(((level - 1) / 3) as f64).floor() as usize],
            aa: ad,
        }
    }

    pub fn calculate_damage(&self, active_player: &active_player::Root, ability: &str, abilityranks: &AbilityRanks, resistance: dmg::Resistance) -> f64 {
        let mut dmg = Vec::new();
        let raw_damage = self.calculate_rd(active_player, abilityranks);

        for i in ability.chars() {
            match i {
                'Q' => dmg.push(dmg::calculate_mitigation(raw_damage.q, resistance.magic_resist)),
                'W' => dmg.push(dmg::calculate_mitigation(raw_damage.w, resistance.magic_resist)),
                'E' => dmg.push(dmg::calculate_mitigation(raw_damage.e, resistance.magic_resist)),
                'R' => dmg.push(dmg::calculate_mitigation(raw_damage.r, resistance.magic_resist)),
                'P' => dmg.push(dmg::calculate_mitigation(raw_damage.p, resistance.magic_resist)),
                'A' => dmg.push(dmg::calculate_mitigation(raw_damage.aa, resistance.armor)),
                _ => println!("Invalid ability"),
            }
        }

        dmg.iter().sum()
    }
}

#[derive(Debug)]
struct Stats {
    q_dmg: Vec<f64>,
    w_dmg: Vec<f64>,
    e_dmg: Vec<f64>,
    r_dmg: Vec<f64>,
    p_dmg: Vec<f64>,
}

impl Stats {
    fn new(q_dmg: Vec<f64>, w_dmg: Vec<f64>, e_dmg: Vec<f64>, r_dmg: Vec<f64>, p_dmg: Vec<f64>) -> Self {
        Stats { q_dmg, w_dmg, e_dmg, r_dmg, p_dmg }
    }
}


pub struct RawDamage {
    q: f64,
    w: f64,
    e: f64,
    r: f64,
    p: f64,
    aa: f64,
}