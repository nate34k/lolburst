use crate::{active_player, all_players};

// Returns a tuple of the index of the active player in all_players and the active players team.
pub fn get_active_player(
    active_player: &active_player::Root,
    players: &all_players::Root,
) -> (usize, String, String) {
    let mut res: (usize, String, String) = (0, String::from("None"), String::from("None"));
    for i in 0..players.all_players.len() {
        let n = players.all_players[i].summoner_name.clone();
        if n == active_player.summoner_name {
            res = (
                i,
                players.all_players[i].team.clone(),
                players.all_players[i].champion_name.clone(),
            );
            break;
        }
    }
    res
}

pub struct OpponantTeam {
    pub opponants: Vec<(String, i64)>,
}

impl OpponantTeam {
    pub fn new(active_player: &active_player::Root, players: &all_players::Root) -> Self {
        OpponantTeam {
            opponants: OpponantTeam::build_opponant_team(active_player, players),
        }
    }

    pub fn build_opponant_team(
        active_player: &active_player::Root,
        players: &all_players::Root,
    ) -> Vec<(String, i64)> {
        let mut opponant_list = Vec::new();
        for i in 0..players.all_players.len() {
            let team = players.all_players[i].team.clone();
            if get_active_player(active_player, players).1 != team {
                opponant_list.push((
                    players.all_players[i]
                        .champion_name
                        .clone()
                        .replace('\'', "")
                        .replace(' ', "")
                        .replace('.', ""),
                    players.all_players[i].level,
                ));
            }
        }
        opponant_list
    }
}
