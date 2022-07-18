use std::borrow::Cow;

use crate::common::{MultipleOf, MultipleOfError};
use rocket::{
    form::{
        error::ErrorKind, Error as RocketFormError, Form, FromForm, FromFormField,
        Result as RocketResult, ValueField,
    },
    http::Status,
    response::{
        stream::{Event, EventStream},
        Redirect, Responder,
    },
    serde::json::Json,
    State,
};
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use thiserror::Error;
use tokio::{
    sync::broadcast::{self, Sender},
    try_join,
};

#[derive(Clone, Serialize)]
enum TysiacEvent {
    ScoreUpdated,
    NewGame,
}

pub struct TysiacContext {
    sender: Sender<TysiacEvent>,
}

impl Default for TysiacContext {
    fn default() -> Self {
        let (s, _) = broadcast::channel(16);

        Self { sender: s }
    }
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "tysiac_player", rename_all = "lowercase")]
enum Player {
    One = 1,
    Two = 2,
    Three = 3,
}

impl<'v> FromFormField<'v> for Player {
    fn from_value(field: ValueField<'v>) -> RocketResult<'v, Self> {
        let i: i32 = field.value.parse().unwrap_or_else(|_| match field.value {
            "One" => 1,
            "Two" => 2,
            "Three" => 3,
            _ => -1, // Invalid player,
        });
        i.try_into().map_err(|_| {
            RocketFormError::from(ErrorKind::Validation(Cow::Borrowed("Not a valid player"))).into()
        })
    }
}

impl TryFrom<i32> for Player {
    fn try_from(num: i32) -> Result<Self> {
        Ok(match num {
            1 => Player::One,
            2 => Player::Two,
            3 => Player::Three,
            n => Err(ApiError::NotAValidPlayer { n })?,
        })
    }

    type Error = ApiError;
}

#[derive(Serialize, Deserialize)]
pub struct RoundScores {
    index: i32,
    player_1: i32,
    player_2: i32,
    player_3: i32,
    bid_winner: Option<Player>,
    winning_bid: Option<i32>,
    played_bid: Option<i32>,
}

#[derive(Serialize)]
pub struct Game {
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
    min_score: i32,
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

        let min_score = game
            .round_scores
            .iter()
            .flat_map(|x| [x.player_1, x.player_2, x.player_3])
            .min()
            .unwrap_or(0);

        Self {
            next: game.next,
            prev: game.prev,
            game_id: game.game_id,
            player_names: &game.player_names,
            round_scores: &game.round_scores,
            cumulative_round_scores,
            min_score,
        }
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("The entered password wasn't valid")]
    InvalidPassword,

    #[error("The server wasn't started with an admin password")]
    NoConfiguredPassword,

    #[error("Not a valid player {n}")]
    NotAValidPlayer { n: i32 },

    #[error("SQL Error: {0}")]
    SqlError(#[from] sqlx::Error),

    #[error("Score too high")]
    ScoreTooHigh,

    #[error("Playing bid must be higher than the winning bid")]
    PlayingBidMustBeHigher,

    #[error("A required values was not present")]
    MissingValue,

    #[error("{0}")]
    MultipleOfError(#[from] MultipleOfError),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiError {
    fn respond_to(
        self,
        request: &'r rocket::Request<'_>,
    ) -> std::result::Result<rocket::Response<'o>, Status> {
        (Status::InternalServerError, format!("{}", self)).respond_to(request)
    }
}

type Result<T> = std::result::Result<T, ApiError>;

async fn load_game(game_id: i32, pool: &State<Pool<Postgres>>) -> Result<Game> {
    let game = sqlx::query!(
        "SELECT id, player_1, player_2, player_3
         FROM tysiac_games
         WHERE id = $1",
        game_id
    )
    .fetch_one(&**pool);

    let scores = sqlx::query_as!(RoundScores,
        r#"SELECT index, player_1, player_2, player_3, bid_winner  as "bid_winner: _", winning_bid, played_bid
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

    let (game, scores, next_game, prev_game) = try_join!(game, scores, next_game, prev_game)?;

    let game = Game {
        next: next_game.map(|x| x.id),
        prev: prev_game.map(|x| x.id),
        game_id,
        player_names: (game.player_1, game.player_2, game.player_3),
        round_scores: scores,
    };

    Ok(game)
}

#[derive(FromForm, Deserialize)]
pub struct PlayerNames<'a> {
    #[field(name = "player-1-name")]
    player_1_name: &'a str,
    #[field(name = "player-2-name")]
    player_2_name: &'a str,
    #[field(name = "player-3-name")]
    player_3_name: &'a str,
}

async fn create_game(
    player_names: &PlayerNames<'_>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<i32> {
    let result = sqlx::query!(
        "INSERT INTO tysiac_games (player_1, player_2, player_3)
         VALUES ($1, $2, $3) RETURNING id",
        player_names.player_1_name,
        player_names.player_2_name,
        player_names.player_3_name,
    )
    .fetch_one(&**pool)
    .await?;

    let _ = context.sender.clone().send(TysiacEvent::NewGame);

    Ok(result.id)
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
    bid_winner: Player,
    #[field(name = "winning-bid")]
    winning_bid: i32,
    #[field(name = "playing-bid")]
    playing_bid: i32,
}

#[derive(FromForm)]
pub struct FormEditRoundScores {
    index: i32,
    scores: FormRoundScores,
    delete: bool,
}

#[derive(FromForm)]
pub struct FormEditAllScores {
    all_scores: Vec<FormEditRoundScores>,
    password: String,
}

impl FormRoundScores {
    fn for_player(&self, player: Player) -> i32 {
        match player {
            Player::One => self.player_1_score.value(),
            Player::Two => self.player_2_score.value(),
            Player::Three => self.player_3_score.value(),
        }
    }

    fn validate_scores(&self, p1_sum: i32, p2_sum: i32, p3_sum: i32) -> Result<()> {
        let winning_player = self.bid_winner;
        let winners_score = self.for_player(winning_player);
        let (winners_sum, loser_sums) = match winning_player {
            Player::One => (p1_sum, [p2_sum, p3_sum]),
            Player::Two => (p2_sum, [p1_sum, p3_sum]),
            Player::Three => (p3_sum, [p1_sum, p2_sum]),
        };

        if loser_sums.iter().any(|x| *x > 880) {
            return Err(ApiError::ScoreTooHigh);
        }

        if winners_score.abs() != self.playing_bid && winners_sum != 880 && winners_sum != 1000 {
            return Err(ApiError::ScoreTooHigh);
        }

        if winners_sum > 880 && winners_sum != 1000 {
            return Err(ApiError::ScoreTooHigh);
        }

        if self.playing_bid < self.winning_bid {
            return Err(ApiError::PlayingBidMustBeHigher);
        }

        Ok(())
    }
}

impl TryFrom<&RoundScores> for FormRoundScores {
    type Error = ApiError;

    fn try_from(value: &RoundScores) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            player_1_score: value.player_1.try_into()?,
            player_2_score: value.player_2.try_into()?,
            player_3_score: value.player_3.try_into()?,
            bid_winner: value.bid_winner.ok_or(ApiError::MissingValue)?,
            winning_bid: value.winning_bid.ok_or(ApiError::MissingValue)?,
            playing_bid: value.played_bid.ok_or(ApiError::MissingValue)?,
        })
    }
}

async fn do_add_scores(
    game_id: i32,
    player_scores: &FormRoundScores,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<()> {
    let sums = sqlx::query!(
        "SELECT sum(player_1) as player_1, sum(player_2) as player_2, sum(player_3) as player_3 FROM tysiac_scores WHERE game_id = $1", game_id
    )
    .fetch_one(&**pool)
    .await?;

    let p1_sum = sums.player_1.unwrap_or(0) as i32 + player_scores.player_1_score.value();
    let p2_sum = sums.player_2.unwrap_or(0) as i32 + player_scores.player_2_score.value();
    let p3_sum = sums.player_3.unwrap_or(0) as i32 + player_scores.player_3_score.value();

    player_scores.validate_scores(p1_sum, p2_sum, p3_sum)?;

    sqlx::query!(
        "INSERT INTO tysiac_scores (game_id, player_1, player_2, player_3, bid_winner, winning_bid, played_bid )
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        game_id,
        player_scores.player_1_score.value(),
        player_scores.player_2_score.value(),
        player_scores.player_3_score.value(),
        player_scores.bid_winner as _,
        player_scores.winning_bid,
        player_scores.playing_bid,
    )
    .execute(&**pool)
    .await?;

    let _ = context.sender.clone().send(TysiacEvent::ScoreUpdated);

    Ok(())
}

async fn do_edit_scores(
    player_scores: &[FormEditRoundScores],
    pool: &State<Pool<Postgres>>,
) -> Result<()> {
    player_scores.iter().try_fold(
        (0, 0, 0),
        |(p1_sum, p2_sum, p3_sum),
         FormEditRoundScores {
             index: _,
             scores,
             delete: _,
         }| {
            scores.validate_scores(
                p1_sum + scores.for_player(Player::One),
                p2_sum + scores.for_player(Player::Two),
                p3_sum + scores.for_player(Player::Three),
            )?;
            Result::Ok((
                p1_sum + scores.for_player(Player::One),
                p2_sum + scores.for_player(Player::Two),
                p3_sum + scores.for_player(Player::Three),
            ))
        },
    )?;

    futures::future::join_all(player_scores.iter().map(
        |FormEditRoundScores {
             index,
             scores,
             delete,
         }| {
            if *delete {
                sqlx::query!(
                    "DELETE FROM tysiac_scores
                WHERE index=$1;",
                    index,
                )
                .execute(&**pool)
            } else {
                sqlx::query!(
                    "UPDATE tysiac_scores
                SET player_1=$1, player_2=$2, player_3=$3, bid_winner=$4, winning_bid=$5, played_bid=$6
                WHERE index=$7;",
                    scores.player_1_score.value(),
                    scores.player_2_score.value(),
                    scores.player_3_score.value(),
                    scores.bid_winner as _,
                    scores.winning_bid,
                    scores.playing_bid,
                    index,
                )
                .execute(&**pool)
            }
        },
    ))
    .await
    .into_iter()
    .try_fold((), |_, x| {
        x.map_err(ApiError::SqlError)?;
        Result::Ok(())
    })
}

#[get("/events", format = "json")]
pub fn events(context: &State<TysiacContext>) -> EventStream![] {
    let mut receiver = context.sender.subscribe();

    EventStream! {
        loop {
            if let Ok(event) = receiver.recv().await {
                yield Event::json(&event);
            }
        }
    }
}

#[get("/<game_id>", format = "json")]
pub async fn load(game_id: i32, pool: &State<Pool<Postgres>>) -> Result<Json<Game>> {
    let game = load_game(game_id, pool).await?;
    Ok(Json(game))
}

#[put("/new", data = "<player_names>", format = "json")]
pub async fn new(
    player_names: Json<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<Json<i32>> {
    Ok(Json(create_game(&player_names, pool, context).await?))
}

#[put("/<game_id>/add-scores", data = "<player_scores>", format = "json")]
pub async fn add_scores(
    game_id: i32,
    player_scores: Json<RoundScores>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<Json<()>> {
    do_add_scores(game_id, &((&*player_scores).try_into()?), pool, context).await?;

    Ok(Json(()))
}

#[get("/<game_id>", format = "html", rank = 1)]
pub async fn index(game_id: i32, pool: &State<Pool<Postgres>>) -> Result<Template> {
    let game = load_game(game_id, pool).await?;
    let context: GameContext = (&game).into();
    Ok(Template::render("tysiac/game", &context))
}

#[get("/<game_id>/edit", format = "html", rank = 2)]
pub async fn edit(game_id: i32, pool: &State<Pool<Postgres>>) -> Result<Template> {
    let game = load_game(game_id, pool).await?;
    let context: GameContext = (&game).into();
    Ok(Template::render("tysiac/edit", &context))
}

#[get("/new", format = "html")]
pub async fn new_html() -> Template {
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

#[get("/play-with-sse/<game_id>", format = "html")]
pub async fn play_with_sse(game_id: i32) -> Template {
    Template::render(
        "tysiac/play-with-sse",
        &Game {
            next: None,
            prev: None,
            game_id,
            player_names: ("".into(), "".into(), "".into()),
            round_scores: vec![],
        },
    )
}

#[post(
    "/new",
    data = "<player_names>",
    format = "application/x-www-form-urlencoded"
)]
pub async fn create_html(
    player_names: Form<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<Redirect> {
    Ok(Redirect::to(uri!(
        "/tysiac",
        index(create_game(&player_names, pool, context).await?)
    )))
}

#[post(
    "/<game_id>/add-scores",
    data = "<player_scores>",
    format = "application/x-www-form-urlencoded"
)]
pub async fn add_scores_html(
    game_id: i32,
    player_scores: Form<FormRoundScores>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Result<Redirect> {
    do_add_scores(game_id, &player_scores, pool, context).await?;

    Ok(Redirect::to(uri!("/tysiac", index(game_id))))
}

#[post(
    "/<game_id>/edit",
    data = "<edit_scores>",
    format = "application/x-www-form-urlencoded"
)]
pub async fn edit_scores_post(
    game_id: i32,
    edit_scores: Form<FormEditAllScores>,
    pool: &State<Pool<Postgres>>,
) -> Result<Redirect> {
    let password = std::env::var("ADMIN_PASSWORD").map_err(|_|ApiError::NoConfiguredPassword)?;
    let mut password_hasher = Sha256::new();
    password_hasher.update(password);

    let mut input_hasher = Sha256::new();
    input_hasher.update(&edit_scores.password);

    if input_hasher.finalize() != password_hasher.finalize() {
        return Err(ApiError::InvalidPassword);
    }

    do_edit_scores(&edit_scores.all_scores, pool).await?;
    Ok(Redirect::to(uri!("/tysiac", index(game_id))))
}
