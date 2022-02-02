#[macro_use]
extern crate rocket;
use anyhow::Result;
use rocket::{
    form::{Form, FromFormField, ValueField},
    response::Redirect,
    State,
};
use rocket_dyn_templates::Template;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use tokio::try_join;

#[derive(Serialize)]
struct TysiacGame {
    game_id: i32,
    player_names: (String, String, String),
    round_scores: Vec<(i32, i32, i32)>,
}

#[derive(Serialize)]
struct TysiacGameContext<'a> {
    game_id: i32,
    player_names: &'a (String, String, String),
    round_scores: &'a [(i32, i32, i32)],
    cumulative_round_scores: Vec<(i32, i32, i32)>,
}

impl<'a> From<&'a TysiacGame> for TysiacGameContext<'a> {
    fn from(game: &'a TysiacGame) -> Self {
        let cumulative_round_scores = game
            .round_scores
            .iter()
            .scan((0, 0, 0), |(o1, o2, o3), (n1, n2, n3)| {
                *o1 += *n1;
                *o2 += *n2;
                *o3 += *n3;
                Some((*o1, *o2, *o3))
            })
            .collect();

        Self {
            game_id: game.game_id,
            player_names: &game.player_names,
            round_scores: &game.round_scores,
            cumulative_round_scores,
        }
    }
}

struct MultipleOfFive(i32);

impl<'r> FromFormField<'r> for MultipleOfFive {
    fn from_value(value: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        let x = value.value.parse()?;
        if x % 5 != 0 {
            Err(rocket::form::Error::validation("not a multiple of 5").into())
        } else {
            Ok(MultipleOfFive(x))
        }
    }
}

#[get("/games/<game_id>")]
async fn get_game(game_id: i32, pool: &State<Pool<Postgres>>) -> Option<Template> {
    let game = sqlx::query!(
        "
        SELECT id, player_1, player_2, player_3 FROM games where id = $1
        ",
        game_id
    )
    .fetch_one(&**pool);

    let scores = sqlx::query!(
        "
        SELECT player_1, player_2, player_3 FROM scores where game_id = $1 order by index
        ",
        game_id
    )
    .fetch_all(&**pool);

    let (game, scores) = try_join!(game, scores).ok()?;

    let game = TysiacGame {
        game_id,
        player_names: (game.player_1, game.player_2, game.player_3),
        round_scores: scores
            .into_iter()
            .map(|r| {
                (
                    r.player_1.unwrap(),
                    r.player_2.unwrap(),
                    r.player_3.unwrap(),
                )
            })
            .collect(),
    };

    let context: TysiacGameContext = (&game).into();
    Some(Template::render("game", &context))
}

#[derive(FromForm)]
struct PlayerNames<'a> {
    #[field(name = "player-1-name")]
    player_1_name: &'a str,
    #[field(name = "player-2-name")]
    player_2_name: &'a str,
    #[field(name = "player-3-name")]
    player_3_name: &'a str,
}

#[post("/games/new", data = "<player_names>")]
async fn games_new(
    player_names: Form<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
) -> Option<Redirect> {
    let result = sqlx::query!(
        "insert into games (player_1, player_2, player_3) values ($1, $2, $3) returning id",
        player_names.player_1_name,
        player_names.player_2_name,
        player_names.player_3_name,
    )
    .fetch_one(&**pool)
    .await
    .ok()?;

    Some(Redirect::to(uri!(get_game(result.id))))
}

#[get("/games/new")]
async fn games_new_get() -> Template {
    Template::render(
        "game_new",
        &TysiacGame {
            game_id: 0,
            player_names: ("".into(), "".into(), "".into()),
            round_scores: vec![],
        },
    )
}

#[derive(FromForm)]
struct RoundScores {
    #[field(name = "player-1-score")]
    player_1_score: MultipleOfFive,
    #[field(name = "player-2-score")]
    player_2_score: MultipleOfFive,
    #[field(name = "player-3-score")]
    player_3_score: MultipleOfFive,
}

#[post("/games/<game_id>/add-scores", data = "<player_scores>")]
async fn add_scores(
    game_id: i32,
    player_scores: Form<RoundScores>,
    pool: &State<Pool<Postgres>>,
) -> Option<Redirect> {
    sqlx::query!(
        "insert into scores (game_id, player_1, player_2, player_3 ) values ($1, $2, $3, $4)",
        game_id,
        player_scores.player_1_score.0,
        player_scores.player_2_score.0,
        player_scores.player_3_score.0,
    )
    .execute(&**pool)
    .await
    .ok()?;

    Some(Redirect::to(uri!(get_game(game_id))))
}

#[rocket::main]
async fn main() -> Result<()> {
    let database_url = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    rocket::build()
        .attach(Template::fairing())
        .manage(pool)
        .mount("/", routes![games_new_get, get_game, games_new, add_scores])
        .launch()
        .await?;

    Ok(())
}
