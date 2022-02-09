use crate::common::MultipleOf;
use rocket::{
    form::{Form, FromForm},
    response::{
        stream::{Event, EventStream},
        Redirect,
    },
    serde::json::Json,
    State,
};
use rocket_dyn_templates::Template;
use rocket_okapi::{
    okapi::schemars::{self, JsonSchema},
    openapi,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tokio::{
    sync::broadcast::{self, Sender},
    try_join,
};

#[derive(Clone, Serialize, JsonSchema)]
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

#[get("/events")]
pub fn stream(context: &State<TysiacContext>) -> EventStream![] {
    let mut receiver = context.sender.subscribe();

    EventStream! {
        loop {
            if let Ok(event) = receiver.recv().await {
                yield Event::json(&event);
            }
        }
    }
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[sqlx(type_name = "tysiac_player", rename_all = "lowercase")]
enum Player {
    One,
    Two,
    Three,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RoundScores {
    player_1: i32,
    player_2: i32,
    player_3: i32,
    bid_winner: Option<Player>,
    winning_bid: Option<i32>,
    played_bid: Option<i32>,
}

#[derive(Serialize, JsonSchema)]
pub struct Game {
    game_id: i32,
    next: Option<i32>,
    prev: Option<i32>,
    player_names: (String, String, String),
    round_scores: Vec<RoundScores>,
}

#[derive(Serialize, JsonSchema)]
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

async fn load_game(game_id: i32, pool: &State<Pool<Postgres>>) -> Option<Game> {
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

    Some(game)
}

#[openapi]
#[get("/json/<game_id>", format = "json")]
pub async fn get_game_data(game_id: i32, pool: &State<Pool<Postgres>>) -> Option<Json<Game>> {
    let game = load_game(game_id, pool).await?;
    Some(Json(game))
}

#[get("/<game_id>")]
pub async fn index(game_id: i32, pool: &State<Pool<Postgres>>) -> Option<Template> {
    let game = load_game(game_id, pool).await?;
    let context: GameContext = (&game).into();
    Some(Template::render("tysiac/game", &context))
}

#[derive(FromForm, Deserialize, JsonSchema)]
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
) -> Option<i32> {
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

    let _ = context.sender.clone().send(TysiacEvent::NewGame);

    Some(result.id)
}

#[post("/new", data = "<player_names>")]
pub async fn create(
    player_names: Form<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Option<Redirect> {
    Some(Redirect::to(uri!(
        "/tysiac",
        index(create_game(&player_names, pool, context).await?)
    )))
}

#[openapi]
#[post("/json/new", data = "<player_names>", format = "json")]
pub async fn create_json(
    player_names: Json<PlayerNames<'_>>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Option<Json<i32>> {
    Some(Json(create_game(&player_names, pool, context).await?))
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

#[get("/play-with-sse/<game_id>")]
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

#[derive(FromForm, JsonSchema)]
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

impl FormRoundScores {
    fn for_player(&self, player: Player) -> i32 {
        match player {
            Player::One => self.player_1_score.value(),
            Player::Two => self.player_2_score.value(),
            Player::Three => self.player_3_score.value(),
        }
    }
}

impl TryFrom<&RoundScores> for FormRoundScores {
    type Error = ();

    fn try_from(value: &RoundScores) -> Result<Self, Self::Error> {
        Ok(Self {
            player_1_score: value.player_1.try_into()?,
            player_2_score: value.player_2.try_into()?,
            player_3_score: value.player_3.try_into()?,
            bid_winner: value.bid_winner.ok_or(())? as i32,
            winning_bid: value.winning_bid.ok_or(())?,
            playing_bid: value.played_bid.ok_or(())?,
        })
    }
}

async fn do_add_scores(
    game_id: i32,
    player_scores: &FormRoundScores,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Option<()> {
    let player = match player_scores.bid_winner {
        1 => Player::One,
        2 => Player::Two,
        3 => Player::Three,
        _ => return None,
    };

    let winners_score = player_scores.for_player(player);

    let sums = sqlx::query!(
        "SELECT sum(player_1) as player_1, sum(player_2) as player_2, sum(player_3) as player_3 FROM tysiac_scores WHERE game_id = $1", game_id
    )
    .fetch_one(&**pool)
    .await
    .ok()?;

    let p1_sum = sums
        .player_1
        .map(|x| (x as i32) + player_scores.player_1_score.value());
    let p2_sum = sums
        .player_2
        .map(|x| (x as i32) + player_scores.player_2_score.value());
    let p3_sum = sums
        .player_3
        .map(|x| (x as i32) + player_scores.player_3_score.value());

    let (winners_sum, loser_sums) = match player {
        Player::One => (p1_sum, [p2_sum, p3_sum]),
        Player::Two => (p2_sum, [p1_sum, p3_sum]),
        Player::Three => (p3_sum, [p1_sum, p2_sum]),
    };

    if loser_sums.iter().filter_map(|x| *x).any(|x| x > 880) {
        return None;
    }

    if winners_score.abs() != player_scores.playing_bid && winners_sum != Some(880) {
        return None;
    }

    if winners_sum > Some(880) && winners_sum != Some(1000) {
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
    .await.ok()?;

    let _ = context.sender.clone().send(TysiacEvent::ScoreUpdated);

    Some(())
}

#[post("/<game_id>/add-scores", data = "<player_scores>")]
pub async fn add_scores(
    game_id: i32,
    player_scores: Form<FormRoundScores>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Option<Redirect> {
    do_add_scores(game_id, &player_scores, pool, context).await?;

    Some(Redirect::to(uri!("/tysiac", index(game_id))))
}

#[openapi]
#[post(
    "/json/<game_id>/add-scores",
    data = "<player_scores>",
    format = "json"
)]
pub async fn add_scores_json(
    game_id: i32,
    player_scores: Json<RoundScores>,
    pool: &State<Pool<Postgres>>,
    context: &State<TysiacContext>,
) -> Option<Json<()>> {
    do_add_scores(
        game_id,
        &((&*player_scores).try_into().ok()?),
        pool,
        context,
    )
    .await?;

    Some(Json(()))
}
