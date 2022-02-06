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

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "tysiac_player", rename_all = "lowercase")]
#[derive(Serialize)]
enum Player {
    One,
    Two,
    Three,
}

#[derive(Serialize)]
pub struct RoundScores {
    player_1: i32,
    player_2: i32,
    player_3: i32,
    bid_winner: Option<Player>,
    winning_bid: Option<i32>,
    played_bid: Option<i32>,
}

#[derive(Serialize)]
struct Game {
    game_id: i32,
    next: Option<i32>,
    prev: Option<i32>,
    player_names: (String, String, String),
    round_scores: Vec<RoundScores>,
}

#[derive(Serialize)]
struct GameContext<'a> {
    game_id: i32,
    next: Option<i32>,
    prev: Option<i32>,
    player_names: &'a (String, String, String),
    round_scores: &'a [RoundScores],
    cumulative_round_scores: Vec<(i32, i32, i32)>,
}

impl<'a> From<&'a Game> for GameContext<'a> {
    fn from(game: &'a Game) -> Self {
        let cumulative_round_scores = game
            .round_scores
            .iter()
            .scan((0, 0, 0), |(o1, o2, o3), scores| {
                *o1 += scores.player_1;
                *o2 += scores.player_2;
                *o3 += scores.player_3;
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

    let scores = sqlx::query_as!(RoundScores,
        r#"SELECT player_1, player_2, player_3, bid_winner  as "bid_winner: _", winning_bid, played_bid
         FROM tysiac_scores
         WHERE game_id = $1
         ORDER BY index"#,
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
        round_scores: scores,
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

    Some(Redirect::to(uri!("/tysiac", index(result.id))))
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
pub struct FormRoundScores {
    #[field(name = "player-1-score")]
    player_1_score: MultipleOf<5>,
    #[field(name = "player-2-score")]
    player_2_score: MultipleOf<5>,
    #[field(name = "player-3-score")]
    player_3_score: MultipleOf<5>,
    #[field(name = "bid-winner")]
    bid_winner: i32,
    #[field(name = "winning-bid")]
    winning_bid: i32,
    #[field(name = "playing-bid")]
    playing_bid: i32,
}

#[post("/<game_id>/add-scores", data = "<player_scores>")]
pub async fn add_scores(
    game_id: i32,
    player_scores: Form<FormRoundScores>,
    pool: &State<Pool<Postgres>>,
) -> Option<Redirect> {
    let (player, winners_score) = match player_scores.bid_winner {
        1 => (Player::One, player_scores.player_1_score.value()),
        2 => (Player::Two, player_scores.player_2_score.value()),
        3 => (Player::Three, player_scores.player_3_score.value()),
        _ => return None,
    };

    if winners_score.abs() != player_scores.playing_bid {
        return None;
    }

    if player_scores.playing_bid < player_scores.winning_bid {
        return None;
    }

    sqlx::query!(
        "INSERT INTO tysiac_scores (game_id, player_1, player_2, player_3, bid_winner, winning_bid, played_bid )
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        game_id,
        player_scores.player_1_score.value(),
        player_scores.player_2_score.value(),
        player_scores.player_3_score.value(),
        player as _,
        player_scores.winning_bid,
        player_scores.playing_bid,
    )
    .execute(&**pool)
    .await
    .ok()?;

    Some(Redirect::to(uri!("/tysiac", index(game_id))))
}
