use anyhow::Result;
use poise::serenity_prelude as serenity;
use thiserror::Error;

use crate::Player;

#[derive(Clone, Debug)]
enum Cell {
  Player(Player),
  None,
}

#[derive(Error, Debug)]
pub enum GameError {
  #[error("Player count must be in range 3..=4")]
  PlayerCountError,
}

#[derive(Debug)]
struct PlayerData {
  discord_id: serenity::UserId,
  display_name: String,
}
#[derive(Debug)]
struct Game {
  field: Vec<Vec<Cell>>,
  players: Vec<PlayerData>,
}

impl Game {
  pub fn new(players: Vec<Player>) -> Result<Game> {
    let mut field = vec![vec![Cell::None; 7]; 7];
    field[1][1] = Cell::Player(players[0].clone());
    field[1][5] = Cell::Player(players[1].clone());
    field[5][1] = Cell::Player(players[2].clone());
    field[5][5] = match players.len() {
      3 => Cell::None,
      4 => Cell::Player(players[3].clone()),
      _ => return Err(GameError::PlayerCountError.into()),
    };
    let players = players
      .into_iter()
      .map(|p| -> Result<PlayerData> {
        Ok(PlayerData {
          discord_id: p.discord_id.parse()?,
          display_name: p.display_name,
        })
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(Game { field, players })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() -> Result<()> {
    let game = Game::new(vec![
      Player {
        discord_id: "123123123".to_string(),
        display_name: "테스트용 인간".to_string()
      };
      4
    ])?;
    println!("{:#?}", game);
    Ok(())
  }
}
