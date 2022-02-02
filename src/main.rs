#[macro_use]
extern crate rocket;
use rocket::{
    form::{Form, FromFormField, Result, ValueField},
    response::Redirect,
    State,
};
use rocket_dyn_templates::Template;
use serde::Serialize;
use std::sync::RwLock;

#[derive(Serialize)]
struct TysiacGame {
    player_names: (String, String, String),
    round_scores: Vec<(i32, i32, i32)>,
}

#[derive(Serialize)]
struct TysiacGameContext<'a> {
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
            player_names: &game.player_names,
            round_scores: &game.round_scores,
            cumulative_round_scores,
        }
    }
}

struct MultipleOfFive(i32);

impl<'r> FromFormField<'r> for MultipleOfFive {
    fn from_value(value: ValueField<'r>) -> Result<'r, Self> {
        let x = value.value.parse()?;
        if x % 5 != 0 {
            Err(rocket::form::Error::validation("not a multiple of 5").into())
        } else {
            Ok(MultipleOfFive(x))
        }
    }
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

#[get("/")]
fn index(game: &State<RwLock<TysiacGame>>) -> Template {
    let game: &TysiacGame = &game.read().unwrap();
    let context: TysiacGameContext = game.into();
    Template::render("index", &context)
}

#[post("/add-scores", data = "<player_scores>")]
fn add_scores(player_scores: Form<RoundScores>, game: &State<RwLock<TysiacGame>>) -> Redirect {
    game.write().unwrap().round_scores.push((
        player_scores.player_1_score.0,
        player_scores.player_2_score.0,
        player_scores.player_3_score.0,
    ));
    Redirect::to(uri!(index()))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(RwLock::new(TysiacGame {
            player_names: ("Sheyne".into(), "Daniel".into(), "Alissa".into()),
            round_scores: vec![],
        }))
        .attach(Template::fairing())
        .mount("/", routes![index, add_scores])
}
