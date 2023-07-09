use anyhow::Result;
use font_kit::font::Font;
use poise::serenity_prelude as serenity;
use raqote::*;
use thiserror::Error;

use crate::{draw_ext::DrawTargetExt, Player};

#[derive(Debug, Clone)]
enum Cell {
  Player(PlayerData),
  None,
}

#[derive(Error, Debug)]
pub enum GameError {
  #[error("Player count must be in range 3..=4")]
  PlayerCountError,
}

#[derive(Debug, Clone)]
struct PlayerData {
  discord_id: serenity::UserId,
  display_name: String,
  health: f32,
}

#[derive(Debug)]
pub struct Game {
  field: Vec<Vec<Cell>>,
  players: Vec<PlayerData>,
}

const CELL_SIZE: i32 = 200;
impl Game {
  pub fn new(players: Vec<Player>) -> Result<Game> {
    let mut field = vec![vec![Cell::None; 7]; 7];
    let players = players
      .into_iter()
      .map(|p| -> Result<PlayerData> {
        Ok(PlayerData {
          discord_id: p.discord_id.parse()?,
          display_name: p.display_name,
          health: 123.5764846945,
        })
      })
      .collect::<Result<Vec<_>, _>>()?;
    field[1][1] = Cell::Player(players[0].clone());
    field[1][5] = Cell::Player(players[1].clone());
    field[5][1] = Cell::Player(players[2].clone());
    field[5][5] = match players.len() {
      3 => Cell::None,
      4 => Cell::Player(players[3].clone()),
      _ => return Err(GameError::PlayerCountError.into()),
    };
    Ok(Game { field, players })
  }

  pub fn draw(&self, file_path: String) -> Result<()> {
    let mut dt = DrawTarget::new(7 * CELL_SIZE, 7 * CELL_SIZE);
    let white = Color::new(0xff, 0xdd, 0xdd, 0xdd);
    let gray = Color::new(0xff, 0xbb, 0xbb, 0xbb);
    let blue = Color::new(0xff, 0x32, 0x48, 0xa8);
    let red = Color::new(0xff, 0xc7, 0x47, 0x36);
    let font = Font::from_path("fonts/SUITE-ExtraBold.otf", 0)?;
    for (i, row) in self.field.iter().enumerate() {
      for (j, cell) in row.iter().enumerate() {
        let color = match *cell {
          Cell::Player(_) => blue,
          Cell::None if (i + j) % 2 == 0 => white,
          Cell::None => gray,
        };
        dt.fill_rect(
          (j as i32 * CELL_SIZE) as f32,
          (i as i32 * CELL_SIZE) as f32,
          CELL_SIZE as f32,
          CELL_SIZE as f32,
          &Source::Solid(color.into()),
          &DrawOptions::new(),
        );
        if let Cell::Player(ref p) = *cell {
          dt.draw_text_fix(
            &font,
            50.,
            &*p.display_name,
            Point::new(
              (j as f32 + 0.5) * CELL_SIZE as f32,
              (i as f32 + 0.4) * CELL_SIZE as f32,
            ),
            &Source::Solid(white.into()),
            &DrawOptions::new(),
          );
          dt.draw_text_fix(
            &font,
            50.,
            &*format!("{:.1}", p.health),
            Point::new(
              (j as f32 + 0.5) * CELL_SIZE as f32,
              (i as f32 + 0.8) * CELL_SIZE as f32,
            ),
            &Source::Solid(red.into()),
            &DrawOptions::new(),
          );
        }
      }
    }

    dt.write_png(file_path).map_err(|e| e.into())
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
        display_name: "뉴민".to_string(),
      },
      Player {
        discord_id: "123123123".to_string(),
        display_name: "부계정".to_string(),
      },
      Player {
        discord_id: "123123123".to_string(),
        display_name: "Guraud".to_string(),
      },
    ])?;
    println!("{:#?}", game);
    game.draw("wasans.png".to_string())?;
    Ok(())
  }
}
