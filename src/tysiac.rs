use crate::common::MultipleOf;
use rocket::{
    form::{Form, FromForm},
    response::Redirect,
    State,
};
use rocket_dyn_templates::Template;
use serde::Serialize;
use sqlx::{Pool, Postgres};
use tokio::try_join;

#[derive(Serialize)]
struct Game {
    game_id: i32,
    next: Option<i32>,
    prev: Option<i32>,
    player_names: (String, String, String),
    round_scores: Vec<(i32, i32, i32)>,
}

#[derive(Serialize)]
struct GameContext<'a> {
    game_id: i32,
    next: Option<i32>,
    prev: Option<i32>,
    player_names: &'a (String, String, String),
    round_scores: &'a [(i32, i32, i32)],
    cumulative_round_scores: Vec<(i32, i32, i32)>,
}

impl<'a> From<&'a Game> for GameContext<'a> {
    fn from(game: &'a Game) -> Self {
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
            next: game.next,
            prev: game.prev,
            game_id: game.game_id,
            player_names: &game.player_names,
            round_scores: &game.round_scores,
            cumulative_round_scores,
        }
    }
}

#[get("/<game_id>")]
pub async fn index(game_id: i32, pool: &State<Pool<Postgres>>) -> Option<Template> {
    let game = sqlx::query!(
        "SELECT id, player_1, player_2, player_3
         FROM tysiac_games
         WHERE id = $1",
        game_id
    )
    .fetch_one(&**pool);

    let scores = sqlx::query!(
        "SELECT player_1, player_2, player_3
         FROM tysiac_scores
         WHERE game_id = $1
         ORDER BY index",
        game_id
    )
    .fetch_all(&**pool);

    let next_game = sqlx::query!(
        "SELECT id
         FROM tysiac_games
         WHERE id > $1
         ORDER BY id ASC",
        game_id
    )
    .fetch_optional(&**pool);
    let prev_game = sqlx::query!(
        "SELECT id
         FROM tysiac_games
         WHERE id < $1
         ORDER BY id DESC",
        game_id
    )
    .fetch_optional(&**pool);

    let (game, scores, next_game, prev_game) =
        try_join!(game, scores, next_game, prev_game).ok()?;

    let game = Game {
        next: next_game.map(|x| x.id),
        prev: prev_game.map(|x| x.id),
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

    let context: GameContext = (&game).into();
    Some(Template::render("tysiac/game", &context))
}

#[derive(FromForm)]
pub struct PlayerNames<'a> {
    #[field(name = "player-1-name")]
    player_1_name: &'a str,
    #[field(name = "player-2-name")]
    player_2_name: &'a str,
    #[field(name = "player-3-name")]
    player_3_name: &'a str,
}

#[post("/new", data = "<player_names>")]
pub async fn create(
    player_names: Form<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
) -> Option<Redirect> {
    let result = sqlx::query!(
        "INSERT INTO tysiac_games (player_1, player_2, player_3)
         VALUES ($1, $2, $3) RETURNING id",
        player_names.player_1_name,
        player_names.player_2_name,
        player_names.player_3_name,
    )
    .fetch_one(&**pool)
    .await
    .ok()?;

    Some(Redirect::to(uri!(index(result.id))))
}

#[get("/new")]
pub async fn new() -> Template {
    Template::render(
        "tysiac/new",
        &Game {
            next: None,
            prev: None,
            game_id: 0,
            player_names: ("".into(), "".into(), "".into()),
            round_scores: vec![],
        },
    )
}

#[derive(FromForm)]
pub struct RoundScores {
    #[field(name = "player-1-score")]
    player_1_score: MultipleOf<5>,
    #[field(name = "player-2-score")]
    player_2_score: MultipleOf<5>,
    #[field(name = "player-3-score")]
    player_3_score: MultipleOf<5>,
}

#[post("/<game_id>/add-scores", data = "<player_scores>")]
pub async fn add_scores(
    game_id: i32,
    player_scores: Form<RoundScores>,
    pool: &State<Pool<Postgres>>,
) -> Option<Redirect> {
    sqlx::query!(
        "INSERT INTO tysiac_scores (game_id, player_1, player_2, player_3 )
     VALUES ($1, $2, $3, $4)",
        game_id,
        player_scores.player_1_score.value(),
        player_scores.player_2_score.value(),
        player_scores.player_3_score.value(),
    )
    .execute(&**pool)
    .await
    .ok()?;

    Some(Redirect::to(uri!(index(game_id))))
}
