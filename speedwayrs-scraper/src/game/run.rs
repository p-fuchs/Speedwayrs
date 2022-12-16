use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use regex::Regex;
use scraper::{element_ref::Select, selector::Selector, ElementRef, Html};
use serde::{Serialize, Deserialize};

use super::team::PlayerScore;

const RUN_COUNT: u8 = 15;

#[derive(Debug, Serialize, Deserialize)]
pub enum Helmet {
    Red,
    Yellow,
    Blue,
    White,
}

impl Helmet {
    pub fn try_parsing(text: &str) -> Option<Self> {
        if text.contains("red") {
            Some(Self::Red)
        } else if text.contains("blue") {
            Some(Self::Blue)
        } else if text.contains("yellow") {
            Some(Self::Yellow)
        } else if text.contains("white") {
            Some(Self::White)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRunScore {
    name: String,
    score: PlayerScore,
    helmet: Option<Helmet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    number: u8,
    time: Option<(u16, u8)>,
    player_score: Vec<PlayerRunScore>,
}

impl Run {
    pub fn new(number: u8, time: Option<(u16, u8)>, player_score: Vec<PlayerRunScore>) -> Self {
        Self {
            number,
            time,
            player_score,
        }
    }
}

struct RunIterator<'a> {
    remaining: u8,
    players: Select<'a, 'static>,
    time: Select<'a, 'static>,
    site: String
}

impl<'a> RunIterator<'a> {
    fn new(parsed_body: &'a Html, site: &str) -> Self {
        let parsed_body = parsed_body.select(run_list_selector()).next().unwrap();
        let players = parsed_body.select(player_selector());
        let time = parsed_body.select(time_selector());

        Self {
            remaining: RUN_COUNT,
            players,
            time,
            site: site.to_string()
        }
    }
}

fn parse_time(text: &str) -> Option<(u16, u8)> {
    eprintln!("TIME: {:?}", text);
    static TIME_REGEX: OnceCell<Regex> = OnceCell::new();

    if text.len() == 0 {
        return None;
    }

    let regex = TIME_REGEX
        .get_or_init(|| Regex::new(r#"(?P<secs>\d+)(.(?P<ten_mills>\d+))? sek."#).unwrap());
    let captures = regex
        .captures(text)
        .with_context(|| format!("Unable to parse {text}"))
        .unwrap();

    Some((
        captures.name("secs").unwrap().as_str().parse().unwrap(),
        captures
            .name("ten_mills")
            .map(|res| res.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap(),
    ))
}

fn parse_name(text: &str) -> String {
    static NAME_REGEX: OnceCell<Regex> = OnceCell::new();

    let regex =
        NAME_REGEX.get_or_init(|| Regex::new(r#"(\s)*(?P<name>\S+) (?P<surname>\S+)"#).unwrap());
    let captures = regex
        .captures(text)
        .with_context(|| format!("Unable to parse names from {text}."))
        .unwrap();

    format!(
        "{} {}",
        captures.name("name").unwrap().as_str(),
        captures.name("surname").unwrap().as_str()
    )
}

fn parse_competitor(element: &ElementRef) -> Result<PlayerRunScore> {
    // Parsing name
    let name = element
        .select(player_name_selector())
        .next()
        .context("Unable to load player name.")?
        .inner_html();

    let name = parse_name(&name);

    // Parsing score
    let score_binding = element
        .select(player_score_selector())
        .next()
        .context("Unable to load player's score.")?
        .inner_html();

    let score;
    if score_binding.trim().len() <= 2 {
        score = score_binding.trim();
    } else {
        score = &score_binding.trim()[0..2];
    }

    let score = PlayerScore::parse(score)?;

    // Parsing helmet icon
    let helmet = Helmet::try_parsing(
        &element
            .select(player_helmet_selector())
            .next()
            .context("Unable to find helmet color.")?
            .html(),
    );

    Ok(PlayerRunScore {
        name,
        score,
        helmet,
    })
}

impl<'a> Iterator for RunIterator<'a> {
    type Item = Run;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let number = self.remaining;
        self.remaining -= 1;

        let time = self.time.next().unwrap();
        eprintln!("TIME ELEMENT: {:?} ON SITE {:?}", time.html(), self.site);
        let time = parse_time(&time.inner_html());

        let mut players_score = Vec::new();
        for _ in 0..4 {
            let competitor = self.players.next().unwrap();
            players_score.push(parse_competitor(&competitor).unwrap());
        }

        Some(Self::Item::new(number, time, players_score))
    }
}

pub fn run_iterator<'a>(parsed_body: &'a Html, site: &str) -> impl Iterator<Item = Run> + 'a {
    RunIterator::new(parsed_body, site)
}

fn player_name_selector() -> &'static Selector {
    static NAME_SELECTOR: OnceCell<Selector> = OnceCell::new();

    NAME_SELECTOR.get_or_init(|| Selector::parse(".competitor__name").unwrap())
}

fn player_score_selector() -> &'static Selector {
    static SCORE_SELECTOR: OnceCell<Selector> = OnceCell::new();

    SCORE_SELECTOR.get_or_init(|| Selector::parse(".competitor__score").unwrap())
}

fn player_helmet_selector() -> &'static Selector {
    static HELMET_SELECTOR: OnceCell<Selector> = OnceCell::new();

    HELMET_SELECTOR.get_or_init(|| Selector::parse(".icon-helmet").unwrap())
}

fn player_selector() -> &'static Selector {
    static PLAYER_SELECTOR: OnceCell<Selector> = OnceCell::new();

    PLAYER_SELECTOR.get_or_init(|| Selector::parse(".competitor").unwrap())
}

fn time_selector() -> &'static Selector {
    static TIME_SELECTOR: OnceCell<Selector> = OnceCell::new();

    TIME_SELECTOR.get_or_init(|| Selector::parse(".coventry__time").unwrap())
}

fn run_list_selector() -> &'static Selector {
    static RUN_LIST_SELECTOR: OnceCell<Selector> = OnceCell::new();

    RUN_LIST_SELECTOR.get_or_init(|| Selector::parse(".coveragelist").unwrap())
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use scraper::Html;

    use crate::game::team::PlayerScore;

    use super::run_iterator;

    #[test]
    fn parsing_time() {
        let time = "56.12 sek.";
        let parsed_time = super::parse_time(time);

        assert!(parsed_time.unwrap().0 == 56);
        assert!(parsed_time.unwrap().1 == 12);
    }

    #[test]
    fn iterator_test() {
        let site_fragment = r#"
            <div class="coveragelist">
                <span class="coventry__time">56.12 sek.</span>
                <li class="competitor">
                    <span class="competitor__name">John Test</span>
                    <span class="competitor__score">5</span>
                    <figure class="icon-helmet red"></figure>
                </li>
                <li class="competitor">
                    <span class="competitor__name">Adam Test</span>
                    <span class="competitor__score">6</span>
                    <figure class="icon-helmet white"></figure>
                </li>
                <li class="competitor">
                    <span class="competitor__name">Max Test</span>
                    <span class="competitor__score">7</span>
                    <figure class="icon-helmet blue"></figure>
                </li>
                <li class="competitor">
                    <span class="competitor__name">Barb Test</span>
                    <span class="competitor__score">8</span>
                    <figure class="icon-helmet yellow"></figure>
                </li>
            </div>
        "#;

        let html = Html::parse_fragment(site_fragment);
        let mut iter = run_iterator(&html, "a");

        let run = iter.next().context("Run iterator does not work").unwrap();

        assert_eq!(run.time.unwrap(), (56, 12));
        assert_eq!(run.number, 15);
        assert_eq!(run.player_score[1].name, "Adam Test");
        assert_eq!(run.player_score[2].score, PlayerScore::Score(7));
    }
}
