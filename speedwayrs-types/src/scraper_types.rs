use time::PrimitiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Helmet {
    Red,
    Yellow,
    Blue,
    White,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    surname: String,
    number: u32,

    scores: Vec<super::PlayerResult>,
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    name: String,
    points: u16,
    players: Vec<Player>
}

#[derive(Serialize, Deserialize)]
pub struct GameInfo {
    team1: Team,
    team2: Team,
    stadium: String,
    date: time::PrimitiveDateTime,
    runs: Vec<Run>
}

#[derive(Serialize, Deserialize)]
pub struct PlayerRunScore {
    name: String,
    score: super::PlayerResult,
    helmet: Option<Helmet>
}

#[derive(Serialize, Deserialize)]
pub struct Run {
    number: u8,
    time: Option<(u16, u8)>,
    player_score: Vec<PlayerRunScore>
}
