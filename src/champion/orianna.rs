use crate::{active_player, data::ActivePlayer, dmg, AbilityRanks};

use super::{CalculateDamageStruct, CalculateDamageTrait};

#[derive(Debug, Clone)]
pub struct Orianna<'a> {
    pub name: &'a str,
    pub stats: Stats,
}

impl Orianna<'_> {
    pub fn default() -> Self {
        Orianna {
            name: "Orianna",
            stats: Stats::default(),
        }
    }

    pub fn calculate_rd(
        &self,
        active_player: &ActivePlayer,
        abilityranks: &AbilityRanks,
    ) -> RawDamage {
        let q_rank = abilityranks.q_rank;
        let w_rank = abilityranks.w_rank;
        let e_rank = abilityranks.e_rank;
        let r_rank = abilityranks.r_rank;
        let ap = active_player.champion_stats.ability_power;
        let ad = active_player.champion_stats.attack_damage;
        let level = active_player.level;
        RawDamage {
            q: (self.stats.q_dmg[q_rank as usize]) + (self.stats.q_dmg[6] * ap),
            w: (self.stats.w_dmg[w_rank as usize]) + (self.stats.w_dmg[6] * ap),
            e: (self.stats.e_dmg[e_rank as usize]) + (self.stats.e_dmg[6] * ap),
            r: (self.stats.r_dmg[r_rank as usize]) + (self.stats.r_dmg[4] * ap),
            p: self.stats.p_dmg[(((level - 1) / 3) as f64).floor() as usize],
            aa: ad,
        }
    }
}

impl CalculateDamageTrait for Orianna<'_> {
    fn calculate_damage(&self, cdi: CalculateDamageStruct) -> f64 {
        let mut dmg = Vec::new();
        let raw_damage = self.calculate_rd(cdi.active_player, cdi.ability_ranks);

        for i in cdi.rotation.chars() {
            match i {
                'Q' => dmg.push(dmg::mitigate_damage_by_magic_resist(
                    raw_damage.q,
                    cdi.resistance.magic_resist,
                )),
                'W' => dmg.push(dmg::mitigate_damage_by_magic_resist(
                    raw_damage.w,
                    cdi.resistance.magic_resist,
                )),
                'E' => dmg.push(dmg::mitigate_damage_by_magic_resist(
                    raw_damage.e,
                    cdi.resistance.magic_resist,
                )),
                'R' => dmg.push(dmg::mitigate_damage_by_magic_resist(
                    raw_damage.r,
                    cdi.resistance.magic_resist,
                )),
                'P' => dmg.push(dmg::mitigate_damage_by_magic_resist(
                    raw_damage.p,
                    cdi.resistance.magic_resist,
                )),
                'A' => dmg.push(dmg::mitigate_damage_by_armor(
                    raw_damage.aa,
                    cdi.resistance.armor,
                )),
                _ => {
                    info!("Unknown rotation: {}", i);
                    dmg.push(0.0)
                }
            }
        }

        dmg.iter().sum()
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    q_dmg: Vec<f64>,
    w_dmg: Vec<f64>,
    e_dmg: Vec<f64>,
    r_dmg: Vec<f64>,
    p_dmg: Vec<f64>,
}

impl Stats {
    pub fn default() -> Self {
        Stats {
            q_dmg: vec![0.0, 60.0, 90.0, 120.0, 150.0, 180.0, 0.5],
            w_dmg: vec![0.0, 60.0, 105.0, 150.0, 195.0, 240.0, 0.7],
            e_dmg: vec![0.0, 60.0, 90.0, 120.0, 150.0, 180.0, 0.3],
            r_dmg: vec![0.0, 200.0, 275.0, 350.0, 0.8],
            p_dmg: vec![10.0, 18.0, 26.0, 34.0, 42.0, 50.0, 0.15, 1.2],
        }
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
