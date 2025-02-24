use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::components::{Player, Score};
use crate::states::GameState;
use crate::util::cleanup_components;

pub struct ScoreDisplayPlugin;

impl Plugin for ScoreDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InPlay), display_score)
            .add_systems(
                Update,
                update_score_text.run_if(in_state(GameState::InPlay)),
            )
            .add_systems(
                OnExit(GameState::InPlay),
                cleanup_components::<ScoreDisplay>,
            );
    }
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct PlayerScoreText(u8);

fn display_score(mut commands: Commands, score_q: Query<(&Score, &Player)>) {
    commands
        .spawn((
            ScoreDisplay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(5.),
                top: Val::Px(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|score_display| {
            score_display.spawn(Text::new("Score:"));
            for (score, player) in score_q.iter() {
                score_display
                    .spawn((Text::new(format!("Player {}: ", player.0)),))
                    .with_child((
                        PlayerScoreText(player.0),
                        TextSpan::new(score.0.to_string()),
                    ));
            }
        });
}

fn update_score_text(
    score_q: Query<(&Score, &Player), Changed<Score>>,
    mut player_score_text_q: Query<(&mut TextSpan, &PlayerScoreText)>,
) {
    for (score, player) in score_q.iter() {
        for (mut text_span, player_score_text) in player_score_text_q.iter_mut() {
            if player_score_text.0 == player.0 {
                text_span.0 = score.0.to_string();
            }
        }
    }
}
