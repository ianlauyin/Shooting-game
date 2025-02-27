use std::collections::HashMap;

use rocket::tokio::sync::Mutex;
use shooting_game_shared::util::EdgeUtil;

#[derive(Default)]
pub struct Players(Mutex<HashMap<u8, PlayerInfo>>);

impl Players {
    pub async fn new_player(&self) -> u8 {
        let mut players = self.0.lock().await;
        let player_tag = players.len() as u8 + 1;
        let player = PlayerInfo::default();
        players.insert(player_tag, player);
        player_tag
    }

    pub async fn matched(&self) -> bool {
        let players = self.0.lock().await;
        players.len() == 2
    }

    pub async fn ready(&self) -> bool {
        let players = self.0.lock().await;
        let mut ready_count = 0;
        for player in players.values() {
            let edge_util = EdgeUtil::spaceship();
            if !edge_util.over_bottom_in(player.position.1) {
                ready_count += 1;
            }
        }
        ready_count == players.len()
    }

    pub async fn update_player_info(
        &self,
        player_tag: u8,
        position: (f32, f32),
        bullets: Vec<(f32, f32)>,
    ) {
        let mut players = self.0.lock().await;
        players.entry(player_tag).and_modify(|player| {
            player.position = position;
            player.bullets = bullets;
        });
    }
}

#[derive(Debug)]
struct PlayerInfo {
    score: u16,
    health: u16,
    position: (f32, f32),
    bullets: Vec<(f32, f32)>,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            score: 0,
            health: 3,
            position: (0.0, 0.0),
            bullets: Vec::new(),
        }
    }
}
