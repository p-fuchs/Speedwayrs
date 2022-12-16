use std::collections::HashMap;

pub struct MatchResult {
    team_1: String,
    team_2: String,
    score_1: u32,
    score_2: u32,

    rounds: u8,

    team_one: Team,
    team_two: Team,
}

pub struct Team {
    name: String,
    points: u16,
    players: Vec<Player>,
}

pub enum PlayerResult {
    Score(u8),
    ScoreWithStar(u8),
    Fall,
    Reserve,
    None,
}

pub struct Player {
    name: String,
    surname: String,
    number: u32,

    scores: HashMap<u8, PlayerResult>,
}
