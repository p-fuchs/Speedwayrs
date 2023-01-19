use std::hash::Hash;

use serde::{Deserialize, Serialize};
use speedwayrs_types::PlayerResult;
use time::PrimitiveDateTime;

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

    scores: Vec<PlayerResult>,
}

impl Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        state.write(self.surname.as_bytes());
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.surname == other.surname
    }
}

impl Eq for Player {}

impl Player {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn surname(&self) -> &str {
        &self.surname
    }

    pub fn scores(&self) -> &[PlayerResult] {
        self.scores.as_slice()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    name: String,
    points: u16,
    players: Vec<Player>,
}

impl Team {
    pub fn players(&self) -> &[Player] {
        self.players.as_slice()
    }
}

impl Team {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn score(&self) -> u16 {
        self.points
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameInfo {
    team1: Team,
    team2: Team,
    stadium: String,
    date: time::PrimitiveDateTime,
    runs: Vec<Run>,
}

impl GameInfo {
    pub fn team_one(&self) -> &Team {
        &self.team1
    }

    pub fn date(&self) -> &time::PrimitiveDateTime {
        &self.date
    }

    pub fn team_two(&self) -> &Team {
        &self.team2
    }

    pub fn place(&self) -> &str {
        &self.stadium
    }

    pub fn runs(&self) -> &[Run] {
        self.runs.as_slice()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerRunScore {
    name: String,
    score: PlayerResult,
    helmet: Option<Helmet>,
}

impl PlayerRunScore {
    pub fn score(&self) -> &PlayerResult {
        &self.score
    }

    pub fn name(&self) -> (&str, &str) {
        self.name.trim().split_once(' ').unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Run {
    number: u8,
    time: Option<(u16, u8)>,
    player_score: Vec<PlayerRunScore>,
}

impl Run {
    pub fn position(&self) -> u8 {
        self.number
    }

    pub fn time(&self) -> (Option<i32>, Option<i32>) {
        match self.time {
            None => (None, None),
            Some((int, dec)) => (Some(int as i32), Some(dec as i32)),
        }
    }

    pub fn player_scores(&self) -> &[PlayerRunScore] {
        self.player_score.as_slice()
    }
}
