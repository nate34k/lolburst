pub mod orianna;

#[derive(Debug)]
pub enum ActiveChampion {
    Orianna(orianna::Orianna),
    None,
}

impl ActiveChampion {
    pub fn match_champion(name: &str) -> ActiveChampion {
        let champion = match name {
            "Orianna" => ActiveChampion::Orianna(orianna::Orianna::build()),
            _ => ActiveChampion::None,
        };
        champion
    }
}