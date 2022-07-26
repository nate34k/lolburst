use crate::champions::ActiveChampion;

pub fn calculate_rd(active_champion: &ActiveChampion, ap: &f64, ability_ranks: &super::AbilityRanks) -> f64 {
    match active_champion {
        ActiveChampion::Orianna(orianna) => crate::orianna::Orianna::calculate_rd(&orianna, ap, ability_ranks),
        _ => 0.0,
    }
}