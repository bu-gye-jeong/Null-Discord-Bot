use anyhow::{Context as _, Error};
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use sqlx::FromRow;

#[derive(FromRow)]
struct Player {
  pub discord_id: String,
  pub display_name: String,
}

struct PlayerDB {
  pool: sqlx::PgPool,
}

impl PlayerDB {
  fn new(pool: sqlx::PgPool) -> Self {
    PlayerDB { pool }
  }

  async fn create(&self, player: &Player) -> Result<(), Error> {
    sqlx::query("INSERT INTO player (discord_id, display_name) VALUES ($1, $2)")
      .bind(player.discord_id.to_string())
      .bind(&player.display_name)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn get_by_id(&self, id: &serenity::UserId) -> Result<Player, Error> {
    let player = sqlx::query_as::<_, Player>("SELECT * FROM player WHERE discord_id = $1")
      .bind(id.to_string())
      .fetch_one(&self.pool)
      .await?;
    Ok(player)
  }
}

struct Data {
  player_db: PlayerDB,
}
type Context<'a> = poise::Context<'a, Data, Error>;

/// 플레이어 등록
#[poise::command(slash_command)]
async fn register(
  ctx: Context<'_>, #[description = "표시명"] name: String
) -> Result<(), Error> {
  let player = Player {
    discord_id: ctx.author().id.to_string(),
    display_name: name,
  };
  let response = match player.display_name.len() {
    33.. => "이름이 너무 깁니다! (최대 32바이트)".to_string(),
    _ => match ctx.data().player_db.create(&player).await {
      Ok(_) => format!("이름 {}으로 등록 완료", &player.display_name),
      Err(_) => "이미 등록된 사용자입니다!".to_string(),
    },
  };

  ctx.say(response).await?;
  Ok(())
}

/// 플레이어 확인
#[poise::command(slash_command)]
async fn who(
  ctx: Context<'_>, #[description = "누구"] user: serenity::User
) -> Result<(), Error> {
  let response = match ctx.data().player_db.get_by_id(&user.id).await {
    Ok(player) => format!("Id: {}, 이름: {}", user.id, player.display_name),
    Err(_) => "미등록된 사용자입니다!".to_string(),
  };
  ctx.say(response).await?;
  Ok(())
}

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
      commands: vec![register(), who()],
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

async fn setup_db(database_url: String) -> Result<PlayerDB, Error> {
  let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
  sqlx::migrate!("./migrations").run(&pool).await?;
  Ok(PlayerDB::new(pool))
}
