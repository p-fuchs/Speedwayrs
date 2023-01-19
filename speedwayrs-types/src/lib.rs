pub mod scraper_types;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MatchResult {
    team_1: String,
    team_2: String,
    score_1: u32,
    score_2: u32,

    date: time::OffsetDateTime,
    place: String,

    runs: Vec<RunInfo>,
    player_results: Vec<Player>
}

impl MatchResult {
    pub fn new(team_1: String, team_2: String, score_1: u32, score_2: u32, place: String, date: time::OffsetDateTime, runs: Vec<RunInfo>, player_results: Vec<Player>) -> Self {
        Self {
            team_1,
            team_2,
            score_1,
            score_2,
            date,
            place,
            runs,
            player_results
        }
    }
    pub fn first_team_name(&self) -> &str {
        &self.team_1
    }

    pub fn second_team_name(&self) -> &str {
        &self.team_2
    }

    pub fn first_team_score(&self) -> u32 {
        self.score_1
    }

    pub fn second_team_score(&self) -> u32 {
        self.score_2
    }
}

#[derive(Serialize, Deserialize)]
pub struct RunInfo {
    number: u8,
    time: Option<(u32, u16)>,
    // First field represents player name and second field represents player's score.
    player_scores: Vec<(String, String)>
}

#[derive(Serialize, Deserialize)]
pub enum PlayerResult {
    Score(u8),
    ScoreWithStar(u8),
    Fall,
    Reserve,
    Defect,
    Tape,
    NotFinished,
    None,
}

impl PlayerResult {
    pub fn from_str(source: &str) -> Option<Self> {
        serde_json::from_str(source).ok()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    surname: String,

    scores: Vec<(u8, PlayerResult)>,
}

impl Player {
    pub fn new(name: String, surname: String, mut scores: Vec<(u8, PlayerResult)>) -> Self {
        scores.sort_unstable_by_key(|record| record.0);

        Self {
            name,
            surname,
            scores
        }
    }
}

impl RunInfo {
    pub fn new(number: u8, time: Option<(u32, u16)>, player_scores: Vec<(String, String)>) -> Self {
        Self { number, time, player_scores }
    }
}
