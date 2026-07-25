use crate::game::{asset, Action, AssetKind, Game, GameError, Phase, ASSETS};

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
            let definition = asset(auction.tile).unwrap();
            let completion_bonus = match definition.kind {
                AssetKind::Street { group } => {
                    let owned = ASSETS
                        .iter()
                        .filter(|a| matches!(a.kind, AssetKind::Street { group: g } if g == group))
                        .filter(|a| game.assets[&a.tile].owner == Some(actor))
                        .count();
                    if owned > 0 {
                        definition.price / 2
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            let cap = (definition.price + completion_bonus)
                .min(game.players[actor].cash.saturating_sub(PURCHASE_RESERVE));
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
                if game.players[player].jail_turns > 0 && game.players[player].get_out_cards > 0 {
                    game.apply(Action::UseJailCard)?;
                }
                game.apply(Action::Roll)?;
            }
            Phase::OfferPurchase { tile } => {
                let price = asset(tile).unwrap().price;
                if game.players[player].cash - price >= PURCHASE_RESERVE {
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
    for definition in &ASSETS {
        let state = &game.assets[&definition.tile];
        if state.owner == Some(player) && state.mortgaged {
            let cost = (definition.mortgage_value() * 110 + 99) / 100;
            if game.players[player].cash - cost >= IMPROVEMENT_RESERVE {
                return Some(Action::Unmortgage(definition.tile));
            }
        }
    }
    for definition in &ASSETS {
        let AssetKind::Street { group } = definition.kind else {
            continue;
        };
        let state = &game.assets[&definition.tile];
        if state.owner == Some(player)
            && game.owns_group(player, group)
            && state.houses < 4
            && game.players[player].cash - definition.house_cost >= IMPROVEMENT_RESERVE
        {
            let min = ASSETS
                .iter()
                .filter(|a| matches!(a.kind, AssetKind::Street { group: g } if g == group))
                .map(|a| game.assets[&a.tile].houses)
                .min()
                .unwrap_or(0);
            if state.houses == min {
                return Some(Action::Build(definition.tile));
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
