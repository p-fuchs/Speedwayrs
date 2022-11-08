use std::io::Write;

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub enum PlayerScore {
    Score(u8),
    ScoreWithStar(u8),
    Reserve,
    None,
}

#[derive(Debug)]
pub struct Player {
    name: String,
    surname: String,
    number: u32,

    scores: Vec<PlayerScore>,
}

impl Player {
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

            match match_score.inner_html().trim() {
                "" => scores.push(PlayerScore::None),
                "-" => scores.push(PlayerScore::Reserve),
                num => {
                    if num.ends_with('*') {
                        let trimmed = num.trim_end_matches('*');
                        scores.push(PlayerScore::ScoreWithStar(trimmed.parse().map_err(|_| anyhow!("INAVLID T {trimmed}"))?));
                    } else {
                        scores.push(PlayerScore::Score(num.parse().map_err(|_| anyhow!("INVALID {num}"))?))
                    }
                }
            }
        }

        Ok(Player {
            name,
            surname,
            number,
            scores,
        })
    }
}

#[test]
fn tak() {
    let x = include_str!("../../mecz.tmp");

    let parsed = Html::parse_document(x);
    let selector = Selector::parse(
        ".coveragetab__speedwaytables > div:nth-child(1) > table:nth-child(2) > tbody:nth-child(2)",
    )
    .unwrap();

    let x = parsed.select(&selector).next().unwrap();
    let selp = Selector::parse("tr").unwrap();

    for player_data in x.select(&selp) {
        println!("PARSED {:?}", Player::parse_player(player_data));
    }

    // parse_player(parsed);
}
