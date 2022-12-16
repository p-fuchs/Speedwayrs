mod player;

use anyhow::Context;
use anyhow::Result;
use once_cell::sync::OnceCell;

pub use player::Player;
pub use player::PlayerScore;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    name: String,
    points: u16,
    players: Vec<Player>,
}

fn team_one_selector() -> &'static Selector {
    static TEAM_ONE_SELECTOR: OnceCell<Selector> = OnceCell::new();

    TEAM_ONE_SELECTOR.get_or_init(|| Selector::parse(".coveragetab__speedwaytables > div:nth-child(1) > table:nth-child(2) > tbody:nth-child(2)").unwrap())
}

fn team_two_selector() -> &'static Selector {
    static TEAM_TWO_SELECTOR: OnceCell<Selector> = OnceCell::new();

    TEAM_TWO_SELECTOR.get_or_init(|| Selector::parse(".coveragetab__speedwaytables > div:nth-child(2) > table:nth-child(2) > tbody:nth-child(2)").unwrap())
}

impl Team {
    fn sum_score(players: &Vec<Player>) -> u16 {
        let mut sum = 0;
        for player in players.iter() {
            sum += player.sum_score().base();
        }

        sum
    }

    fn parse_players(table: &ElementRef) -> Result<Vec<Player>> {
        static TR_SELECTOR: OnceCell<Selector> = OnceCell::new();
        let tr_selector = TR_SELECTOR.get_or_init(|| Selector::parse("tr").unwrap());

        let mut result_players = Vec::new();
        for selected_player in table.select(tr_selector) {
            result_players.push(Player::parse_player(selected_player)?);
        }

        Ok(result_players)
    }

    pub fn parse_teams(parsed_body: &Html) -> Result<(Team, Team)> {
        static TEAM_NAME_SELECTOR: OnceCell<Selector> = OnceCell::new();

        let name_selector =
            TEAM_NAME_SELECTOR.get_or_init(|| Selector::parse(".mclabel__name > .name").unwrap());

        let mut team_names = parsed_body.select(&name_selector);
        let team1_name = team_names
            .next()
            .context("Unable to find team 1 name.")?
            .inner_html();
        let team2_name = team_names
            .next()
            .context("Unable to find team 2 name.")?
            .inner_html();

        let team_1 = parsed_body
            .select(team_one_selector())
            .next()
            .context("Unable to find team 1 players.")?;
        let team_2 = parsed_body
            .select(team_two_selector())
            .next()
            .context("Unable to find team 2 players.")?;

        let team_1 = Self::parse_players(&team_1)?;
        let team_2 = Self::parse_players(&team_2)?;

        let team_one = Self {
            name: team1_name,
            points: Self::sum_score(&team_1),
            players: team_1,
        };

        let team_two = Self {
            name: team2_name,
            points: Self::sum_score(&team_2),
            players: team_2,
        };

        Ok((team_one, team_two))
    }
}

#[test]
fn test() {
    let body = include_str!("s.tmp");
    let parsed_body = Html::parse_document(body);

    let (x, y) = Team::parse_teams(&parsed_body).unwrap();
    println!("TEAM1 {x:?}");
    println!("=====");
    println!("TEAM2 {y:?}");
}
