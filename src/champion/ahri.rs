use crate::{active_player::AbilityRanks, data::ActivePlayer, dmg};

use super::{CalculateDamageStruct, CalculateDamageTrait};

#[derive(Debug, Clone)]
pub struct Ahri<'a> {
    pub name: &'a str,
    pub stats: Stats,
}

impl Ahri<'_> {
    pub fn default() -> Self {
        Ahri {
            name: "Ahri",
            stats: Stats::default(),
        }
    }

    pub fn calculate_rd(
        &self,
        active_player: &ActivePlayer,
        abilityranks: &AbilityRanks,
    ) -> RawDamage {
        let q_rank = abilityranks.q_rank;
        let (q_stats, q_scale) = &self.stats.q_dmg;
        let w_rank = abilityranks.w_rank;
        let (w_stats, w_scale) = &self.stats.w_dmg;
        let e_rank = abilityranks.e_rank;
        let (e_stats, e_scale) = &self.stats.e_dmg;
        let r_rank = abilityranks.r_rank;
        let (r_stats, r_scale) = &self.stats.r_dmg;
        let ap = active_player.champion_stats.ability_power;
        let ad = active_player.champion_stats.attack_damage;
        RawDamage {
            q: (q_stats[q_rank as usize]) + (q_scale * ap),
            w: (w_stats[w_rank as usize]) + (w_scale * ap),
            e: (e_stats[e_rank as usize]) + (e_scale * ap),
            r: (r_stats[r_rank as usize]) + (r_scale * ap),
            aa: ad,
        }
    }
}

impl CalculateDamageTrait for Ahri<'_> {
    fn calculate_damage(&self, cdi: CalculateDamageStruct) -> f64 {
        let mut dmg = Vec::new();
        let raw_damage = self.calculate_rd(cdi.active_player, cdi.ability_ranks);

        for i in cdi.rotation.chars() {
            match i {
                // Ahri's Q is magic damage + true damage
                'Q' => dmg.push(
                    dmg::mitigate_damage_by_magic_resist(raw_damage.q, cdi.resistance.magic_resist)
                        + raw_damage.q,
                ),
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
                'A' => dmg.push(dmg::mitigate_damage_by_armor(
                    raw_damage.aa,
                    cdi.resistance.armor,
                )),
                _ => {
                    info!("Invalid ability: {}", i);
                    dmg.push(0.0);
                }
            }
        }
        dmg.iter().sum()
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub q_dmg: (Vec<f64>, f64),
    pub w_dmg: (Vec<f64>, f64),
    pub e_dmg: (Vec<f64>, f64),
    pub r_dmg: (Vec<f64>, f64),
}

impl Stats {
    fn default() -> Self {
        Stats {
            q_dmg: (vec![0.0, 40.0, 65.0, 90.0, 115.0, 140.0], 0.4),
            w_dmg: (vec![0.0, 80.0, 120.0, 160.0, 200.0, 240.0], 0.48),
            e_dmg: (vec![0.0, 80.0, 110.0, 140.0, 170.0, 200.0], 0.6),
            r_dmg: (vec![0.0, 60.0, 90.0, 120.0], 0.35),
        }
    }
}

pub struct RawDamage {
    pub q: f64,
    pub w: f64,
    pub e: f64,
    pub r: f64,
    pub aa: f64,
}
