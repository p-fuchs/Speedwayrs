mod run;
mod team;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use regex::Regex;
use scraper::{Html, Selector};

use self::{run::Run, team::Team};

#[derive(Debug)]
pub struct GameSite {
    url: String,
}

impl GameSite {
    fn new<T: AsRef<str>>(relative_url: T) -> Self {
        Self {
            url: format!("{}{}", crate::BASE_SITE, relative_url.as_ref()),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn parse_match_schedule(parsed_body: &Html) -> Result<Vec<Self>> {
        let selector = Selector::parse(".cmatch__link").unwrap();

        let mut result_vec = Vec::new();

        for game_info in parsed_body.select(&selector) {
            let relative_path = game_info
                .value()
                .attr("href")
                .with_context(|| "Unable to find <href> attribute of game site.")?;

            result_vec.push(Self::new(relative_path));
        }

        Ok(result_vec)
    }
}

#[derive(Debug)]
pub struct ScraperGameInfo {
    team1: Team,
    team2: Team,
    runs: Vec<Run>,
}

fn parse_score(score_inner_html: &str) -> Result<(u32, u32)> {
    static SCORE_REGEX: OnceCell<Regex> = OnceCell::new();

    let regex =
        SCORE_REGEX.get_or_init(|| Regex::new(r"(?P<SCORE1>\d+)(\D*)(?P<SCORE2>\d+)").unwrap());

    let mut captures = regex.captures_iter(score_inner_html);
    let matches = captures.next().unwrap();

    let score_1 = &matches["SCORE1"];
    let score_2 = &matches["SCORE2"];

    Ok((
        score_1
            .parse()
            .with_context(|| format!("Unable to parse score of first team [{score_1}]."))?,
        score_2
            .parse()
            .with_context(|| format!("Unable to parse score of second team [{score_2}]."))?,
    ))
}

impl ScraperGameInfo {
    pub fn parse_site(body: &str) -> Result<Self> {
        let parsed_body = Html::parse_document(body);

        let (team1, team2) = Team::parse_teams(&parsed_body)?;
        let runs = run::run_iterator(&parsed_body).collect();

        Ok(Self { team1, team2, runs })
    }
}
