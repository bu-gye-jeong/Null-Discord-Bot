use crate::{game::Game, Context, Data, Player};
use anyhow::Result;
use futures::StreamExt;
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

/// 게임 시작
#[poise::command(slash_command)]
pub async fn start(
  ctx: Context<'_>,
  player1: serenity::User,
  player2: serenity::User,
  player3: serenity::User,
  player4: Option<serenity::User>,
) -> Result<()> {
  let mut players = vec![player1, player2, player3];
  if let Some(p4) = player4 {
    players.push(p4);
  };
  let mut registered_players = vec![];
  let mut unregistered_players = vec![];
  for (i, result) in futures::stream::iter(&players)
    .then(|p| async move { ctx.data().player_db.get_by_id(&p.id).await })
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .enumerate()
  {
    match result {
      Ok(p) => registered_players.push(p),
      Err(_) => unregistered_players.push(&players[i].name),
    }
  }
  if !unregistered_players.is_empty() {
    ctx
      .say(format!("미등록 사용자: {:?}", unregistered_players))
      .await?;
    return Ok(());
  }
  let game = Game::new(registered_players)?;
  game.draw("msg.png".to_string())?;
  let mut data_game = ctx.data().game.lock().await;
  *data_game = Some(game);
  ctx.say("게임을 시작합니다.").await?;
  ctx
    .channel_id()
    .send_files(ctx, vec!["msg.png"], |m| m.content(""))
    .await?;
  Ok(())
}

pub fn commands() -> Vec<poise::Command<Data, anyhow::Error>> {
  vec![register(), who(), start()]
}
