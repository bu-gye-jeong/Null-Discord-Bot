use anyhow::{Context as _, Error, Result};
use null_discord_bot::{commands, Data, PlayerDB};
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
  let database_url = secret_store
    .get("DATABASE_URL")
    .context("'DATABASE_URL' was not found")?;
  let player_db = setup_db(database_url).await?;

  let discord_token = secret_store
    .get("DISCORD_TOKEN")
    .context("'DISCORD_TOKEN' was not found")?;
  let guild_ids = secret_store
    .get("GUILDS")
    .unwrap_or("".to_string())
    .split(',')
    .filter_map(|s| s.parse().ok().map(serenity::GuildId))
    .collect::<Vec<_>>();

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: commands::commands(),
      ..Default::default()
    })
    .token(discord_token)
    .intents(serenity::GatewayIntents::non_privileged())
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        for id in guild_ids {
          poise::builtins::register_in_guild(ctx, &framework.options().commands, id).await?;
        }
        Ok(Data { player_db })
      })
    })
    .build()
    .await
    .map_err(shuttle_runtime::CustomError::new)?;

  Ok(framework.into())
}

async fn setup_db(database_url: String) -> Result<PlayerDB> {
  let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
  sqlx::migrate!("./migrations").run(&pool).await?;
  Ok(PlayerDB::new(pool))
}
