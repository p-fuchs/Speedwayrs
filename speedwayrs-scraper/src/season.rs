use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use regex::Regex;
use scraper::{Html, Selector};

use super::*;

#[derive(Debug)]
pub struct Season {
    year: u32,
    site: String,
}

impl Season {
    pub fn site(&self) -> &str {
        &self.site
    }

    fn new(year_str: &str, relative_path: &str) -> Result<Self> {
        static SEASON_REGEX: OnceCell<Regex> = OnceCell::new();

        let regex = SEASON_REGEX.get_or_init(|| Regex::new(r"Sezon (\d+)").unwrap());

        let season = &regex.captures_iter(year_str).next().with_context(|| {
            format!(
                "Season string is invalid. Expected 'SEZON {{}}'. GOT [{}].",
                year_str
            )
        })?[1];

        let year = season
            .trim()
            .parse()
            .with_context(|| format!("Season number is invalid. Got [{}]", season))?;
        let site = format!("{BASE_SITE}{relative_path}");

        Ok(Self { year, site })
    }

    fn parse_current_season(parsed_body: &Html) -> Result<Self> {
        let selector = Selector::parse("li.filtersitem:nth-child(1) > div:nth-child(1)").unwrap();

        let current_season_info = parsed_body
            .select(&selector)
            .next()
            .with_context(|| "Unable to find current season info.")?;
        let season_info = current_season_info.inner_html();

        Ok(Self::new(&season_info, RELATIVE_MAIN)?)
    }

    pub fn parse_site() -> Result<Vec<Self>> {
        let body = http::HttpRequester::make_request(MAIN_SITE)?;
        let parsed_body = Html::parse_document(&body);

        let selector_dropdown_menu =
            Selector::parse("li.filtersitem:nth-child(1) > div:nth-child(2) > ul:nth-child(2)")
                .unwrap();
        let selector_li = Selector::parse("li").unwrap();
        let selector_a = Selector::parse("a").unwrap();

        let mut result_vec = Vec::new();
        result_vec.push(Self::parse_current_season(&parsed_body)?);

        let li_father = parsed_body
            .select(&selector_dropdown_menu)
            .next()
            .with_context(|| "Unable to find dropdown menu with season dates.")?;
        for li_element in li_father.select(&selector_li) {
            let a_element = li_element
                .select(&selector_a)
                .next()
                .with_context(|| "Unable to find <a> element.")?;

            let season_number = a_element.inner_html();
            let season_href = a_element
                .value()
                .attr("href")
                .with_context(|| "Unable to find <href> element.")?;

            result_vec.push(Self::new(&season_number, season_href)?);
        }

        Ok(result_vec)
    }
}
