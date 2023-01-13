mod run;
mod team;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Month};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ScraperGameInfo {
    team1: Team,
    team2: Team,
    stadium: String,
    date: time::PrimitiveDateTime,
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
    fn parse_stadium(parsed_body: &Html) -> Result<String> {
        static SELECTOR: OnceCell<Selector> = OnceCell::new();

        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("ul.bdcd > li:nth-child(3) > span:nth-child(2) > a:nth-child(1)")
                .unwrap()
        });

        let stadium = parsed_body
            .select(selector)
            .next()
            .context("Unable to select stadium")?;

        Ok(stadium.inner_html())
    }

    fn parse_date(parsed_body: &Html) -> Result<time::PrimitiveDateTime> {
        static SELECTOR: OnceCell<Selector> = OnceCell::new();

        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("ul.bdcd > li:nth-child(2) > span:nth-child(2)").unwrap()
        });

        let date = parsed_body
            .select(selector)
            .next()
            .context("Unable to select date.")?
            .inner_html();

        let mut splitted_date = date.split_ascii_whitespace();

        let day = splitted_date
            .next()
            .unwrap()
            .parse()
            .context("Unable to parse day.")?;
        let month = splitted_date.next().unwrap();
        let year = splitted_date
            .next()
            .unwrap()
            .trim_end_matches(',')
            .parse()
            .context("Unable to parse year.")?;
        let time = splitted_date.next().unwrap();

        let month = {
            match month.to_ascii_lowercase().as_str() {
                "stycznia" => Month::January,
                "lutego" => Month::February,
                "marca" => Month::March,
                "kwietnia" => Month::April,
                "maja" => Month::May,
                "czerwca" => Month::June,
                "lipca" => Month::July,
                "sierpnia" => Month::August,
                "września" => Month::September,
                "października" => Month::October,
                "listopada" => Month::November,
                "grudnia" => Month::December,
                other => {
                    panic!("Month {other} is not parsable.");
                }
            }
        };

        let mut time_splitter = time.split_terminator(':');

        let hour = time_splitter
            .next()
            .unwrap()
            .parse()
            .context("Unable to parse hour.")?;
        let minute = time_splitter
            .next()
            .unwrap()
            .parse()
            .context("Unable to parse minutes.")?;
        let time = time::Time::from_hms(hour, minute, 0).unwrap();

        let date = time::Date::from_calendar_date(year, month, day)
            .context("Unable to create date-time.")?;

        Ok(time::PrimitiveDateTime::new(date, time))
    }

    pub fn parse_site(body: &str, site: &str) -> Result<Self> {
        let parsed_body = Html::parse_document(body);

        let (team1, team2) = Team::parse_teams(&parsed_body)
            .with_context(|| format!("Program with team parsing at site {:?}.", site))?;
        let runs = run::run_iterator(&parsed_body, site).collect();
        let stadium = Self::parse_stadium(&parsed_body)
            .with_context(|| format!("Program with stadium parsing at site {:?}.", site))?;
        let date = Self::parse_date(&parsed_body)
            .with_context(|| format!("Program with date parsing at site {:?}.", site))?;

        Ok(Self {
            team1,
            team2,
            runs,
            stadium,
            date,
        })
    }
}
