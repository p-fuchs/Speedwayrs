mod player;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use regex::Regex;
use scraper::{Html, Selector};

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
    team1: String,
    score1: u32,
    team2: String,
    score2: u32,
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
    pub fn parse_site(body: &str, url: &str) -> Result<Self> {
        let parsed_body = Html::parse_document(body);

        let team_selector = Selector::parse(".matchcoverage__name").unwrap();
        let mut selected_teams = parsed_body.select(&team_selector);

        let team1 = selected_teams
            .next()
            .with_context(|| format!("Unable to get info about first team on site [{}].", url))?
            .inner_html()
            .trim()
            .to_string();

        let team2 = selected_teams
            .next()
            .with_context(|| format!("Unable to get info about second team on site [{}].", url))?
            .inner_html()
            .trim()
            .to_string();

        let score_selector = Selector::parse(".matchcoverage__result").unwrap();
        let score_inner_html = parsed_body
            .select(&score_selector)
            .next()
            .with_context(|| format!("Unable to get score at site [{}].", url))?
            .inner_html();

        let (score1, score2) = parse_score(&score_inner_html)?;

        Ok(Self {
            team1,
            score1,
            team2,
            score2,
        })
    }
}
