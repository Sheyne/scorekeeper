#[macro_use]
extern crate rocket;
use anyhow::Result;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

mod common;
mod tysiac;

#[derive(Serialize)]
struct Tysiac {
    latest: i32,
}

#[derive(Serialize)]
struct IndexContext {
    tysiac: Tysiac,
}

#[get("/")]
async fn index(pool: &State<Pool<Postgres>>) -> Option<Template> {
    let result = sqlx::query!("select id from tysiac_games order by id desc",)
        .fetch_one(&**pool)
        .await
        .ok()?;

    Some(Template::render(
        "index",
        &IndexContext {
            tysiac: Tysiac { latest: result.id },
        },
    ))
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
        .mount("/", routes![index])
        .mount(
            "/tysiac",
            routes![
                tysiac::index,
                tysiac::new,
                tysiac::create,
                tysiac::add_scores
            ],
        )
        .launch()
        .await?;

    Ok(())
}
