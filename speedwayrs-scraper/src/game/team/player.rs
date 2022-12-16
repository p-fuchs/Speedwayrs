use std::io::Write;

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PlayerScore {
    Score(u8),
    ScoreWithStar(u8),
    Fall,
    Reserve,
    Resign,
    None,
}

impl PlayerScore {
    pub fn parse(text: &str) -> Result<Self> {
        Ok(match text.trim() {
            "" => PlayerScore::None,
            "-" => PlayerScore::Reserve,
            "u" | "U" | "w" => PlayerScore::Fall,
            "d" => PlayerScore::Resign,
            num => {
                if num.ends_with('*') {
                    let trimmed = num.trim_end_matches('*');
                    PlayerScore::ScoreWithStar(
                        trimmed
                            .parse()
                            .map_err(|_| anyhow!("INVALID T {trimmed}"))?,
                    )
                } else {
                    PlayerScore::Score(num.parse().map_err(|_| anyhow!("INVALID {num}"))?)
                }
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    name: String,
    surname: String,
    number: u32,

    scores: Vec<PlayerScore>,
}

pub struct PlayerSumScore {
    base: u16,
    bonus: Option<u16>,
}

impl PlayerSumScore {
    pub fn new(base: u16, bonus: Option<u16>) -> Self {
        Self { base, bonus }
    }

    pub fn base(&self) -> u16 {
        self.base
    }

    pub fn bonus(&self) -> Option<u16> {
        self.bonus
    }
}

impl Player {
    pub fn sum_score(&self) -> PlayerSumScore {
        let mut sum = 0;
        let mut bonus_sum = 0;
        for score in self.scores.iter() {
            match score {
                PlayerScore::Score(score) => sum += *score as u16,
                PlayerScore::ScoreWithStar(score) => {
                    sum += *score as u16;
                    bonus_sum += 1;
                }
                _ => {}
            }
        }

        if bonus_sum == 0 {
            PlayerSumScore::new(sum, None)
        } else {
            PlayerSumScore::new(sum, Some(bonus_sum))
        }
    }

    pub fn parse_player(element: ElementRef) -> Result<Player> {
        let info_selector = Selector::parse("td").unwrap();

        let mut selected = element.select(&info_selector);

        let number: u32;
        let name: String;
        let surname: String;

        // Parse number
        number = selected
            .next()
            .unwrap()
            .inner_html()
            .trim()
            .parse()
            .unwrap();

        // Parse name
        let credentials = selected
            .next()
            .unwrap()
            .select(&Selector::parse("a").unwrap())
            .next()
            .unwrap()
            .value()
            .attr("title")
            .unwrap();
        let mut splitted = credentials.trim().split_whitespace();

        name = splitted.next().unwrap().into();
        surname = splitted.next().unwrap().into();

        let mut scores = Vec::new();

        for (index, match_score) in selected.enumerate() {
            if index == 7 {
                break;
            }

            scores.push(PlayerScore::parse(&match_score.inner_html())?);
        }

        Ok(Player {
            name,
            surname,
            number,
            scores,
        })
    }
}
