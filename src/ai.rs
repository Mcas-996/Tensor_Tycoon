use crate::game::{model, Action, Game, GameError, Phase, MODELS};

const PURCHASE_RESERVE: i32 = 200;
const IMPROVEMENT_RESERVE: i32 = 300;

pub fn drive_bots(game: &mut Game) -> Result<(), GameError> {
    for _ in 0..10_000 {
        if game.phase == Phase::GameOver {
            return Ok(());
        }
        if game.phase == Phase::Auction {
            let Some(actor) = game.auction_actor() else {
                return Ok(());
            };
            if game.players[actor].is_human {
                return Ok(());
            }
            let auction = game.auction.as_ref().unwrap();
            let definition = model(auction.tile).unwrap();
            let owned = game.family_model_count(actor, definition.family);
            let completion_bonus = if owned == 2 {
                definition.price() / 2
            } else {
                0
            };
            let cap = (definition.price() + completion_bonus)
                .min(game.players[actor].credits.saturating_sub(PURCHASE_RESERVE));
            let minimum = if auction.high_bid == 0 {
                10
            } else {
                auction.high_bid + 10
            };
            if minimum <= cap {
                game.apply(Action::AuctionBid(minimum))?;
            } else {
                game.apply(Action::AuctionPass)?;
            }
            continue;
        }

        let player = game.current_player;
        if game.players[player].is_human {
            return Ok(());
        }
        match game.phase {
            Phase::AwaitRoll => {
                game.apply(Action::Roll)?;
            }
            Phase::OfferPurchase { tile } => {
                let price = model(tile).unwrap().price();
                if game.players[player].credits - price >= PURCHASE_RESERVE {
                    game.apply(Action::Buy)?;
                } else {
                    game.apply(Action::Decline)?;
                }
            }
            Phase::Manage => {
                if let Some(action) = management_action(game, player) {
                    game.apply(action)?;
                } else {
                    game.apply(Action::EndTurn)?;
                }
            }
            Phase::Auction | Phase::GameOver => {}
        }
    }
    Err(GameError::InvalidAction(
        "bot driver exceeded its safety limit".into(),
    ))
}

fn management_action(game: &Game, player: usize) -> Option<Action> {
    for definition in &MODELS {
        let state = &game.models[&definition.tile];
        if state.owner == Some(player) && state.archived {
            let cost = (definition.archive_value() * 110 + 99) / 100;
            if game.players[player].credits - cost >= IMPROVEMENT_RESERVE {
                return Some(Action::Restore(definition.tile));
            }
        }
    }
    for definition in &MODELS {
        let state = &game.models[&definition.tile];
        if state.owner == Some(player)
            && game.has_tensor_access(player, definition.family)
            && state.tensors < 4
            && game.players[player].credits - definition.tensor_cost >= IMPROVEMENT_RESERVE
        {
            let min = MODELS
                .iter()
                .filter(|a| a.family == definition.family)
                .filter(|a| game.models[&a.tile].owner == Some(player))
                .map(|a| game.models[&a.tile].tensors)
                .min()
                .unwrap_or(0);
            if state.tensors == min {
                return Some(Action::AllocateTensor(definition.tile));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameConfig;

    #[test]
    fn bots_return_control_to_human() {
        let mut game = Game::new(GameConfig {
            bot_count: 3,
            round_limit: 20,
            ..GameConfig::default()
        })
        .unwrap();
        game.phase = Phase::Manage;
        game.apply(Action::EndTurn).unwrap();
        drive_bots(&mut game).unwrap();
        assert!(
            game.phase == Phase::GameOver
                || game.current().is_human
                || game.auction_actor() == Some(0)
        );
    }

    #[test]
    fn deterministic_bot_game_reaches_round_limit() {
        let mut game = Game::new(GameConfig {
            bot_count: 1,
            round_limit: 20,
            ..GameConfig::default()
        })
        .unwrap();
        for _ in 0..2_000 {
            if game.phase == Phase::GameOver {
                break;
            }
            if game.current().is_human && game.phase != Phase::Auction {
                match game.phase {
                    Phase::AwaitRoll => game.apply(Action::Roll).unwrap(),
                    Phase::OfferPurchase { .. } => game
                        .apply(Action::Buy)
                        .unwrap_or_else(|_| game.apply(Action::Decline).unwrap()),
                    Phase::Manage => game.apply(Action::EndTurn).unwrap(),
                    _ => {}
                }
            } else if game.phase == Phase::Auction && game.auction_actor() == Some(0) {
                game.apply(Action::AuctionPass).unwrap();
            }
            drive_bots(&mut game).unwrap();
        }
        assert_eq!(game.phase, Phase::GameOver);
        assert!(!game.winners.is_empty());
    }
}
