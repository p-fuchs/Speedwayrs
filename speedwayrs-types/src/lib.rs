use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchResult {
    team_1: String,
    team_2: String,
    score_1: u32,
    score_2: u32,

    date: time::OffsetDateTime,
    place: String,

    runs: Vec<RunInfo>,
    player_results: Vec<Player>,
}

impl MatchResult {
    pub fn new(
        team_1: String,
        team_2: String,
        score_1: u32,
        score_2: u32,
        place: String,
        date: time::OffsetDateTime,
        runs: Vec<RunInfo>,
        player_results: Vec<Player>,
    ) -> Self {
        Self {
            team_1,
            team_2,
            score_1,
            score_2,
            date,
            place,
            runs,
            player_results,
        }
    }
    pub fn first_team_name(&self) -> &str {
        &self.team_1
    }

    pub fn sort_runs(&mut self) {
        self.runs.sort_by(|a, b| a.number.cmp(&b.number));
    }

    pub fn runs(&self) -> &[RunInfo] {
        &self.runs
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RunInfo {
    number: u8,
    time: Option<(u32, u16)>,
    // First field represents player name and second field represents player's score.
    player_scores: Vec<(i32, String, String)>,
}

impl RunInfo {
    pub fn scores(&self) -> &[(i32, String, String)] {
        &self.player_scores
    }

    pub fn get_time(&self) -> Option<(u32, u16)> {
        self.time
    }

    pub fn time_string(&self) -> String {
        if let Some(t) = self.time {
            format!("{}.{} s", t.0, t.1)
        } else {
            "".into()
        }
    }

    pub fn number(&self) -> u8 {
        self.number
    }
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

    pub fn to_pretty(&self) -> String {
        match self {
            PlayerResult::Score(score) => format!("{}", score),
            PlayerResult::ScoreWithStar(score) => format!("{}*", score),
            PlayerResult::Fall => "Upadek".into(),
            PlayerResult::Reserve => "Rezerwa".into(),
            PlayerResult::Defect => "Defekt".into(),
            PlayerResult::Tape => "Taśma".into(),
            PlayerResult::NotFinished => "Nie ukończono".into(),
            PlayerResult::None => "Brak wyniku?".into()
        }
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
            scores,
        }
    }
}

impl RunInfo {
    pub fn new(number: u8, time: Option<(u32, u16)>, player_scores: Vec<(i32, String, String)>) -> Self {
        Self {
            number,
            time,
            player_scores,
        }
    }
}
