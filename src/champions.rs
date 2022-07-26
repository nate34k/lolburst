pub mod orianna;

#[derive(Debug)]
pub enum ActiveChampion {
    Orianna(orianna::Orianna),
    None,
}

pub fn match_champion(name: &str) -> ActiveChampion {
    let champion = match name {
        "Orianna" => return ActiveChampion::Orianna(orianna::Orianna::build()),
        _ => return ActiveChampion::None,
    };
}