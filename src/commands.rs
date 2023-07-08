use crate::{Context, Data, Player};
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// 플레이어 등록
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>, #[description = "표시명"] name: String) -> Result<()> {
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
pub async fn who(ctx: Context<'_>, #[description = "누구"] user: serenity::User) -> Result<()> {
  let response = match ctx.data().player_db.get_by_id(&user.id).await {
    Ok(player) => format!("Id: {}, 이름: {}", user.id, player.display_name),
    Err(_) => "미등록된 사용자입니다!".to_string(),
  };
  ctx.say(response).await?;
  Ok(())
}

pub fn commands() -> Vec<poise::Command<Data, anyhow::Error>> {
  vec![register(), who()]
}
