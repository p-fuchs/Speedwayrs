use std::collections::HashMap;

pub struct MatchResult {
    team_1: String,
    team_2: String,
    score_1: u32,
    score_2: u32,

    rounds: u8,

    team_1_squad: Vec<Player>,
    team_2_squad: Vec<Player>,
}

pub enum PlayerResult {
    Crossed,
    Scored(u32),
    ScoredWithStar(u32),
}

pub struct Player {
    name: String,
    surname: String,

    scores: HashMap<u8, PlayerResult>,
}
