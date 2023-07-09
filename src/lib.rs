use anyhow::Error;
use futures::lock::Mutex;
use poise::serenity_prelude as serenity;
use sqlx::FromRow;

pub mod commands;
pub mod draw_ext;
pub mod game;

pub struct Data {
  pub player_db: PlayerDB,
  pub game: Mutex<Option<game::Game>>,
}
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(FromRow, Clone, Debug)]
pub struct Player {
  pub discord_id: String,
  pub display_name: String,
}

pub struct PlayerDB {
  pool: sqlx::PgPool,
}

impl PlayerDB {
  pub fn new(pool: sqlx::PgPool) -> Self {
    PlayerDB { pool }
  }

  pub async fn create(&self, player: &Player) -> Result<(), Error> {
    sqlx::query("INSERT INTO player (discord_id, display_name) VALUES ($1, $2)")
      .bind(player.discord_id.to_string())
      .bind(&player.display_name)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  pub async fn get_by_id(&self, id: &serenity::UserId) -> Result<Player, Error> {
    let player = sqlx::query_as::<_, Player>("SELECT * FROM player WHERE discord_id = $1")
      .bind(id.to_string())
      .fetch_one(&self.pool)
      .await?;
    Ok(player)
  }
}
