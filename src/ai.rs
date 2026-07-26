use crate::game::{model, Action, Difficulty, Game, GameError, Phase, MODELS};

const PURCHASE_RESERVE: i32 = 200;
const IMPROVEMENT_RESERVE: i32 = 300;
const EASY_PURCHASE_RESERVE: i32 = 500;
const EASY_IMPROVEMENT_RESERVE: i32 = 600;

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
            let purchase_reserve = purchase_reserve(game);
            let cap = (definition.price() + completion_bonus)
                .min(game.players[actor].credits.saturating_sub(purchase_reserve));
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
                if game.players[player].credits - price >= purchase_reserve(game) {
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
    let improvement_reserve = improvement_reserve(game);
    for definition in &MODELS {
        let state = &game.models[&definition.tile];
        if state.owner == Some(player) && state.archived {
            let cost = (definition.archive_value() * 110 + 99) / 100;
            if game.players[player].credits - cost >= improvement_reserve {
                return Some(Action::Restore(definition.tile));
            }
        }
    }
    for definition in &MODELS {
        let state = &game.models[&definition.tile];
        if state.owner == Some(player)
            && game.has_tensor_access(player, definition.family)
            && state.tensors < 4
            && game.players[player].credits - definition.tensor_cost >= improvement_reserve
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

fn purchase_reserve(game: &Game) -> i32 {
    if game.config.difficulty == Difficulty::Easy {
        EASY_PURCHASE_RESERVE
    } else {
        PURCHASE_RESERVE
    }
}

fn improvement_reserve(game: &Game) -> i32 {
    if game.config.difficulty == Difficulty::Easy {
        EASY_IMPROVEMENT_RESERVE
    } else {
        IMPROVEMENT_RESERVE
    }
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
    fn easy_mode_uses_more_conservative_bot_purchase_reserve() {
        let mut easy = Game::new(GameConfig {
            difficulty: Difficulty::Easy,
            ..GameConfig::default()
        })
        .unwrap();
        easy.current_player = 1;
        easy.players[1].credits = 550;
        easy.phase = Phase::OfferPurchase { tile: 1 };
        drive_bots(&mut easy).unwrap();
        assert_eq!(easy.models[&1].owner, None);
        assert_eq!(easy.phase, Phase::Auction);
        assert_eq!(easy.auction_actor(), Some(0));

        let mut standard = Game::new(GameConfig::default()).unwrap();
        standard.current_player = 1;
        standard.players[1].credits = 550;
        standard.phase = Phase::OfferPurchase { tile: 1 };
        drive_bots(&mut standard).unwrap();
        assert_eq!(standard.models[&1].owner, Some(1));
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
