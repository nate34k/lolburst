use crate::AbilityRanks;

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

    pub fn calculate_rd(orianna: &Orianna, ap: &f64, abilityranks: &AbilityRanks) -> f64 {
        let qrank = abilityranks.q_rank;
        let wrank = abilityranks.w_rank;
        let erank = abilityranks.e_rank;
        let rrank = abilityranks.r_rank;
        let qrd = (orianna.stats.q_dmg[qrank as usize]) + (orianna.stats.q_dmg[6] * ap);
        let wrd = (orianna.stats.w_dmg[wrank as usize]) + (orianna.stats.w_dmg[6] * ap);
        let erd = (orianna.stats.e_dmg[erank as usize]) + (orianna.stats.e_dmg[6] * ap);
        let rrd = (orianna.stats.r_dmg[rrank as usize]) + (orianna.stats.r_dmg[4] * ap);
        qrd + wrd
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