use rocket::tokio::sync::RwLock;
use rocket_ws::result::Error;
use shooting_game_shared::game_related::{Stage, UFORandomGenerator};
use std::sync::Arc;

use crate::message::{Sender, ServerMessageHandler};

use super::players::Players;

pub type SharedGameState = Arc<RwLock<GameState>>;

#[derive(Default, Clone)]
pub enum Cycle {
    #[default]
    Matching,
    Ready,
    Playing,
}

#[derive(Default)]
pub struct GameState {
    cycle: Cycle,
    players: Players,
    stage: RwLock<Stage>,
    enemies: RwLock<Vec<u16>>,
    server_message_handler: ServerMessageHandler,
}

impl GameState {
    pub async fn new_player(&mut self, sender: Sender) -> u8 {
        let player_tag = self.players.new_player().await;
        if let Err((error, player_tag)) = self
            .server_message_handler
            .add_sender(player_tag, sender)
            .await
        {
            match error {
                Error::ConnectionClosed => self.remove_player(player_tag).await,
                e => println!("Error: {:?}", e),
            }
        }
        player_tag
    }

    pub async fn update_player_info(
        &self,
        player_tag: u8,
        position: Option<(f32, f32)>,
        bullets: Vec<(f32, f32)>,
    ) {
        self.players
            .update_player_info(player_tag, position, bullets.clone())
            .await;
    }

    pub async fn player_damaged(&mut self, player_tag: u8, enemy_tag: u16) {
        let mut enemies = self.enemies.write().await;
        if enemies.contains(&enemy_tag) {
            let health = self.players.damaged(player_tag).await;
            match self
                .server_message_handler
                .confirm_damaged(player_tag, enemy_tag, health)
                .await
            {
                Ok(_) => {
                    enemies.retain(|&tag| tag != enemy_tag);
                    drop(enemies);
                    self.check_game_over().await;
                }
                Err((error, _)) => match error {
                    Error::ConnectionClosed => {
                        drop(enemies);
                        self.interrupt_game().await;
                    }
                    e => println!("Error: {:?}", e),
                },
            }
        }
    }

    pub async fn destroy_enemy(&mut self, player_tag: u8, bullet_tag: u16, enemy_tag: u16) {
        let mut enemies = self.enemies.write().await;
        if enemies.contains(&enemy_tag) {
            let new_score = self.players.add_score(player_tag).await;
            match self
                .server_message_handler
                .confirm_destroy_enemy(player_tag, bullet_tag, enemy_tag, new_score)
                .await
            {
                Ok(_) => {
                    enemies.retain(|&tag| tag != enemy_tag);
                    drop(enemies);
                    self.update_stage().await;
                }
                Err((error, _)) => match error {
                    Error::ConnectionClosed => {
                        drop(enemies);
                        self.interrupt_game().await;
                    }
                    e => println!("Error: {:?}", e),
                },
            }
        }
    }

    // Private
    async fn notice_player_info(&mut self) -> Result<(), (Error, u8)> {
        let players = self.players.get_players_info().await;
        for (player_tag, position, bullets) in players {
            self.server_message_handler
                .notice_others_position(player_tag, position, bullets)
                .await?;
        }
        Ok(())
    }

    async fn spawn_enemy(&mut self) -> Result<(), (Error, u8)> {
        let mut enemies = self.enemies.write().await;
        let stage = self.stage.read().await;
        let ufo_numbers = enemies.len() + 1;
        if !stage.random_generator(ufo_numbers) {
            return Ok(());
        }
        let tag = UFORandomGenerator::tag();
        let position = UFORandomGenerator::position();
        let velocity = stage.get_ufo_velocity_tuple();
        if enemies.contains(&tag) {
            return Ok(());
        }
        enemies.push(tag);
        self.server_message_handler
            .enemy_spawn(tag, position, velocity)
            .await
    }

    async fn check_game_over(&mut self) {
        if self.players.all_players_dead().await {
            self.server_message_handler.game_over().await;
            self.cleanup().await;
        }
    }

    async fn cleanup(&mut self) {
        self.enemies.write().await.clear();
        self.players.clear_players().await;
        *self.stage.write().await = Stage::default();
        self.server_message_handler.clear_senders().await;
        self.cycle = Cycle::Matching;
    }

    async fn update_stage(&self) {
        let total_score = self.players.get_total_score().await;
        let new_stage = Stage::new(total_score);
        let mut stage = self.stage.write().await;
        *stage = new_stage;
    }

    async fn remove_player(&mut self, player_tag: u8) {
        self.players.remove_player(player_tag).await;
        self.server_message_handler.clear_sender(player_tag).await;
    }

    async fn interrupt_game(&mut self) {
        self.server_message_handler.game_interrupted().await;
        self.cleanup().await;
    }

    // Cycle Related (Not run in the main thread)
    pub async fn check_cycle(&mut self) -> Cycle {
        match self.cycle {
            Cycle::Matching => self.handle_cycle_matching().await,
            Cycle::Ready => self.handle_cycle_ready().await,
            Cycle::Playing => self.handle_cycle_playing().await,
        }
        self.cycle.clone()
    }

    async fn handle_cycle_matching(&mut self) {
        if self.players.matched().await {
            match self.server_message_handler.game_ready().await {
                Ok(_) => {
                    self.cycle = Cycle::Ready;
                }
                Err((error, _)) => match error {
                    Error::ConnectionClosed => self.interrupt_game().await,
                    e => println!("Error: {:?}", e),
                },
            }
        }
    }

    async fn handle_cycle_ready(&mut self) {
        if let Err((error, _)) = self.notice_player_info().await {
            match error {
                Error::ConnectionClosed => self.interrupt_game().await,
                e => println!("Error: {:?}", e),
            }
        }
        if self.players.ready().await {
            match self.server_message_handler.game_start().await {
                Ok(_) => {
                    self.cycle = Cycle::Playing;
                }
                Err((error, _)) => match error {
                    Error::ConnectionClosed => self.interrupt_game().await,
                    e => println!("Error: {:?}", e),
                },
            }
        }
    }

    async fn handle_cycle_playing(&mut self) {
        if let Err((error, _)) = self.notice_player_info().await {
            match error {
                Error::ConnectionClosed => self.interrupt_game().await,
                e => println!("Error: {:?}", e),
            }
        }
        if let Err((error, _)) = self.spawn_enemy().await {
            match error {
                Error::ConnectionClosed => self.interrupt_game().await,
                e => println!("Error: {:?}", e),
            }
        }
    }
}
