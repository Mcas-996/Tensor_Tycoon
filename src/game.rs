use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const START_CASH: i32 = 1_500;
pub const PASS_START_BONUS: i32 = 200;
pub const JAIL_FINE: i32 = 50;
pub const MAX_HOUSES: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    ZhCn,
    En,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameConfig {
    pub human_name: String,
    pub bot_count: u8,
    pub round_limit: u16,
    pub seed: u64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            human_name: "Player".into(),
            bot_count: 1,
            round_limit: 100,
            seed: 0x4d4f_4e4f_504f_4c59,
        }
    }
}

impl GameConfig {
    pub fn validate(&self) -> Result<(), GameError> {
        if self.human_name.trim().is_empty() {
            return Err(GameError::InvalidConfig("player name is empty".into()));
        }
        if !(1..=3).contains(&self.bot_count) {
            return Err(GameError::InvalidConfig("bot count must be 1..=3".into()));
        }
        if !(20..=500).contains(&self.round_limit) {
            return Err(GameError::InvalidConfig(
                "round limit must be 20..=500".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Street { group: u8 },
    Station,
    Utility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDef {
    pub tile: usize,
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub kind: AssetKind,
    pub price: i32,
    pub base_rent: i32,
    pub house_cost: i32,
}

impl AssetDef {
    pub fn name(&self, language: Language) -> &'static str {
        match language {
            Language::ZhCn => self.name_zh,
            Language::En => self.name_en,
        }
    }

    pub fn mortgage_value(&self) -> i32 {
        self.price / 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Start,
    Asset(usize),
    Event,
    Tax(i32),
    Jail,
    FreeParking,
    GoToJail,
}

pub const BOARD: [Space; 20] = [
    Space::Start,
    Space::Asset(1),
    Space::Event,
    Space::Asset(3),
    Space::Tax(100),
    Space::Jail,
    Space::Asset(6),
    Space::Asset(7),
    Space::Asset(8),
    Space::Asset(9),
    Space::FreeParking,
    Space::Asset(11),
    Space::Event,
    Space::Asset(13),
    Space::Tax(200),
    Space::GoToJail,
    Space::Asset(16),
    Space::Asset(17),
    Space::Asset(18),
    Space::Asset(19),
];

pub const ASSETS: [AssetDef; 12] = [
    AssetDef {
        tile: 1,
        name_zh: "晨曦巷",
        name_en: "Dawn Lane",
        kind: AssetKind::Street { group: 0 },
        price: 100,
        base_rent: 10,
        house_cost: 50,
    },
    AssetDef {
        tile: 3,
        name_zh: "青枫街",
        name_en: "Maple Street",
        kind: AssetKind::Street { group: 0 },
        price: 120,
        base_rent: 12,
        house_cost: 50,
    },
    AssetDef {
        tile: 6,
        name_zh: "潮汐路",
        name_en: "Tide Road",
        kind: AssetKind::Street { group: 1 },
        price: 160,
        base_rent: 16,
        house_cost: 100,
    },
    AssetDef {
        tile: 7,
        name_zh: "北环车站",
        name_en: "North Loop Station",
        kind: AssetKind::Station,
        price: 200,
        base_rent: 25,
        house_cost: 0,
    },
    AssetDef {
        tile: 8,
        name_zh: "海湾大道",
        name_en: "Harbour Avenue",
        kind: AssetKind::Street { group: 1 },
        price: 180,
        base_rent: 18,
        house_cost: 100,
    },
    AssetDef {
        tile: 9,
        name_zh: "清泉水务",
        name_en: "Clearwater Utility",
        kind: AssetKind::Utility,
        price: 150,
        base_rent: 0,
        house_cost: 0,
    },
    AssetDef {
        tile: 11,
        name_zh: "金桂街",
        name_en: "Golden Laurel Street",
        kind: AssetKind::Street { group: 2 },
        price: 220,
        base_rent: 22,
        house_cost: 150,
    },
    AssetDef {
        tile: 13,
        name_zh: "云庭路",
        name_en: "Cloud Court",
        kind: AssetKind::Street { group: 2 },
        price: 240,
        base_rent: 24,
        house_cost: 150,
    },
    AssetDef {
        tile: 16,
        name_zh: "星港大道",
        name_en: "Starport Avenue",
        kind: AssetKind::Street { group: 3 },
        price: 280,
        base_rent: 28,
        house_cost: 200,
    },
    AssetDef {
        tile: 17,
        name_zh: "南环车站",
        name_en: "South Loop Station",
        kind: AssetKind::Station,
        price: 200,
        base_rent: 25,
        house_cost: 0,
    },
    AssetDef {
        tile: 18,
        name_zh: "银河广场",
        name_en: "Galaxy Plaza",
        kind: AssetKind::Street { group: 3 },
        price: 300,
        base_rent: 30,
        house_cost: 200,
    },
    AssetDef {
        tile: 19,
        name_zh: "曙光电力",
        name_en: "Daybreak Power",
        kind: AssetKind::Utility,
        price: 150,
        base_rent: 0,
        house_cost: 0,
    },
];

pub fn asset(tile: usize) -> Option<&'static AssetDef> {
    ASSETS.iter().find(|item| item.tile == tile)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetState {
    pub owner: Option<usize>,
    pub houses: u8,
    pub mortgaged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub is_human: bool,
    pub cash: i32,
    pub position: usize,
    pub jail_turns: u8,
    pub get_out_cards: u8,
    pub bankrupt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    AwaitRoll,
    OfferPurchase { tile: usize },
    Auction,
    Manage,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Gain50,
    Gain100,
    Gain150,
    Pay50,
    Pay100,
    Collect25Each,
    Pay25Each,
    AdvanceStart,
    AdvanceStation,
    BackThree,
    GoToJail,
    GetOutOfJail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuctionState {
    pub tile: usize,
    pub active: Vec<usize>,
    pub bidder_index: usize,
    pub high_bid: i32,
    pub high_bidder: Option<usize>,
}

impl AuctionState {
    pub fn current_bidder(&self) -> Option<usize> {
        self.active.get(self.bidder_index).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameLog {
    Rolled {
        player: usize,
        first: u8,
        second: u8,
    },
    Moved {
        player: usize,
        position: usize,
    },
    Bought {
        player: usize,
        tile: usize,
        price: i32,
    },
    Rent {
        from: usize,
        to: usize,
        amount: i32,
    },
    Cash {
        player: usize,
        amount: i32,
    },
    Tax {
        player: usize,
        amount: i32,
    },
    Drew {
        player: usize,
        card: Card,
    },
    Jailed {
        player: usize,
    },
    Built {
        player: usize,
        tile: usize,
        houses: u8,
    },
    SoldHouse {
        player: usize,
        tile: usize,
        houses: u8,
    },
    Mortgaged {
        player: usize,
        tile: usize,
    },
    Unmortgaged {
        player: usize,
        tile: usize,
    },
    Bankrupt {
        player: usize,
        creditor: Option<usize>,
    },
    Won {
        players: Vec<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn die(&mut self) -> u8 {
        (self.next() % 6 + 1) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub config: GameConfig,
    pub players: Vec<Player>,
    pub assets: BTreeMap<usize, AssetState>,
    pub deck: Vec<Card>,
    pub deck_index: usize,
    pub rng: SimpleRng,
    pub current_player: usize,
    pub round: u16,
    pub phase: Phase,
    pub auction: Option<AuctionState>,
    pub pending_bank_auctions: Vec<usize>,
    pub last_roll: Option<(u8, u8)>,
    pub doubles_streak: u8,
    pub extra_turn: bool,
    pub logs: Vec<GameLog>,
    pub winners: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidConfig(String),
    InvalidPhase,
    InvalidAction(String),
    NotOwner,
    InsufficientFunds,
    UnknownAsset,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(message) | Self::InvalidAction(message) => write!(f, "{message}"),
            Self::InvalidPhase => write!(f, "action is not available in the current phase"),
            Self::NotOwner => write!(f, "player does not own this asset"),
            Self::InsufficientFunds => write!(f, "insufficient funds"),
            Self::UnknownAsset => write!(f, "unknown asset"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Roll,
    PayJail,
    UseJailCard,
    Buy,
    Decline,
    AuctionBid(i32),
    AuctionPass,
    Build(usize),
    SellHouse(usize),
    Mortgage(usize),
    Unmortgage(usize),
    EndTurn,
}

impl Game {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        config.validate()?;
        let mut players = vec![Player {
            id: 0,
            name: config.human_name.clone(),
            is_human: true,
            cash: START_CASH,
            position: 0,
            jail_turns: 0,
            get_out_cards: 0,
            bankrupt: false,
        }];
        for index in 0..config.bot_count {
            players.push(Player {
                id: index as usize + 1,
                name: format!("Bot {}", index + 1),
                is_human: false,
                cash: START_CASH,
                position: 0,
                jail_turns: 0,
                get_out_cards: 0,
                bankrupt: false,
            });
        }
        let mut rng = SimpleRng::new(config.seed);
        let mut deck = vec![
            Card::Gain50,
            Card::Gain100,
            Card::Gain150,
            Card::Pay50,
            Card::Pay100,
            Card::Collect25Each,
            Card::Pay25Each,
            Card::AdvanceStart,
            Card::AdvanceStation,
            Card::BackThree,
            Card::GoToJail,
            Card::GetOutOfJail,
        ];
        for i in (1..deck.len()).rev() {
            let j = (rng.next() as usize) % (i + 1);
            deck.swap(i, j);
        }
        let assets = ASSETS
            .iter()
            .map(|definition| (definition.tile, AssetState::default()))
            .collect();
        Ok(Self {
            config,
            players,
            assets,
            deck,
            deck_index: 0,
            rng,
            current_player: 0,
            round: 1,
            phase: Phase::AwaitRoll,
            auction: None,
            pending_bank_auctions: Vec::new(),
            last_roll: None,
            doubles_streak: 0,
            extra_turn: false,
            logs: Vec::new(),
            winners: Vec::new(),
        })
    }

    pub fn current(&self) -> &Player {
        &self.players[self.current_player]
    }

    pub fn auction_actor(&self) -> Option<usize> {
        self.auction.as_ref().and_then(AuctionState::current_bidder)
    }

    pub fn apply(&mut self, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::GameOver {
            return Err(GameError::InvalidPhase);
        }
        match action {
            Action::Roll => self.roll(),
            Action::PayJail => self.pay_jail(),
            Action::UseJailCard => self.use_jail_card(),
            Action::Buy => self.buy(),
            Action::Decline => self.decline(),
            Action::AuctionBid(amount) => self.auction_bid(amount),
            Action::AuctionPass => self.auction_pass(),
            Action::Build(tile) => self.build(tile),
            Action::SellHouse(tile) => self.sell_house(tile),
            Action::Mortgage(tile) => self.mortgage(tile),
            Action::Unmortgage(tile) => self.unmortgage(tile),
            Action::EndTurn => self.end_turn(),
        }
    }

    fn roll(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::AwaitRoll {
            return Err(GameError::InvalidPhase);
        }
        let player = self.current_player;
        let first = self.rng.die();
        let second = self.rng.die();
        self.last_roll = Some((first, second));
        self.logs.push(GameLog::Rolled {
            player,
            first,
            second,
        });
        let doubles = first == second;

        if self.players[player].jail_turns > 0 {
            if doubles {
                self.players[player].jail_turns = 0;
                self.extra_turn = false;
                self.doubles_streak = 0;
                self.move_by(first + second);
                return Ok(());
            }
            self.players[player].jail_turns += 1;
            if self.players[player].jail_turns > 3 {
                self.charge(player, JAIL_FINE, None);
                if !self.players[player].bankrupt {
                    self.players[player].jail_turns = 0;
                    self.move_by(first + second);
                }
            } else {
                self.phase = Phase::Manage;
            }
            return Ok(());
        }

        if doubles {
            self.doubles_streak += 1;
            self.extra_turn = true;
            if self.doubles_streak >= 3 {
                self.send_to_jail(player);
                self.extra_turn = false;
                self.phase = Phase::Manage;
                return Ok(());
            }
        } else {
            self.doubles_streak = 0;
            self.extra_turn = false;
        }
        self.move_by(first + second);
        Ok(())
    }

    fn pay_jail(&mut self) -> Result<(), GameError> {
        let player = self.current_player;
        if self.phase != Phase::AwaitRoll || self.players[player].jail_turns == 0 {
            return Err(GameError::InvalidPhase);
        }
        if self.players[player].cash < JAIL_FINE {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].cash -= JAIL_FINE;
        self.players[player].jail_turns = 0;
        Ok(())
    }

    fn use_jail_card(&mut self) -> Result<(), GameError> {
        let player = self.current_player;
        if self.phase != Phase::AwaitRoll
            || self.players[player].jail_turns == 0
            || self.players[player].get_out_cards == 0
        {
            return Err(GameError::InvalidPhase);
        }
        self.players[player].get_out_cards -= 1;
        self.players[player].jail_turns = 0;
        Ok(())
    }

    fn move_by(&mut self, amount: u8) {
        let player = self.current_player;
        let old = self.players[player].position;
        let new = (old + amount as usize) % BOARD.len();
        if new < old {
            self.players[player].cash += PASS_START_BONUS;
            self.logs.push(GameLog::Cash {
                player,
                amount: PASS_START_BONUS,
            });
        }
        self.players[player].position = new;
        self.logs.push(GameLog::Moved {
            player,
            position: new,
        });
        self.resolve_space(new);
    }

    fn resolve_space(&mut self, position: usize) {
        let player = self.current_player;
        match BOARD[position] {
            Space::Start | Space::Jail | Space::FreeParking => self.phase = Phase::Manage,
            Space::Tax(amount) => {
                self.logs.push(GameLog::Tax { player, amount });
                self.charge(player, amount, None);
                if !matches!(self.phase, Phase::GameOver | Phase::Auction) {
                    self.phase = Phase::Manage;
                }
            }
            Space::GoToJail => {
                self.send_to_jail(player);
                self.extra_turn = false;
                self.phase = Phase::Manage;
            }
            Space::Event => {
                self.draw_card();
                if !matches!(
                    self.phase,
                    Phase::GameOver | Phase::Auction | Phase::OfferPurchase { .. }
                ) {
                    self.phase = Phase::Manage;
                }
            }
            Space::Asset(tile) => {
                let state = &self.assets[&tile];
                match state.owner {
                    None => self.phase = Phase::OfferPurchase { tile },
                    Some(owner) if owner == player || state.mortgaged => self.phase = Phase::Manage,
                    Some(owner) => {
                        let rent = self.rent_for(tile);
                        self.logs.push(GameLog::Rent {
                            from: player,
                            to: owner,
                            amount: rent,
                        });
                        self.charge(player, rent, Some(owner));
                        if self.phase != Phase::GameOver {
                            self.phase = Phase::Manage;
                        }
                    }
                }
            }
        }
    }

    fn buy(&mut self) -> Result<(), GameError> {
        let Phase::OfferPurchase { tile } = self.phase else {
            return Err(GameError::InvalidPhase);
        };
        let price = asset(tile).ok_or(GameError::UnknownAsset)?.price;
        let player = self.current_player;
        if self.players[player].cash < price {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].cash -= price;
        self.assets.get_mut(&tile).unwrap().owner = Some(player);
        self.logs.push(GameLog::Bought {
            player,
            tile,
            price,
        });
        self.phase = Phase::Manage;
        Ok(())
    }

    fn decline(&mut self) -> Result<(), GameError> {
        let Phase::OfferPurchase { tile } = self.phase else {
            return Err(GameError::InvalidPhase);
        };
        self.start_auction(tile);
        Ok(())
    }

    fn start_auction(&mut self, tile: usize) {
        let active = self
            .players
            .iter()
            .filter(|p| !p.bankrupt)
            .map(|p| p.id)
            .collect();
        self.auction = Some(AuctionState {
            tile,
            active,
            bidder_index: 0,
            high_bid: 0,
            high_bidder: None,
        });
        self.phase = Phase::Auction;
        self.normalize_auction();
    }

    fn auction_bid(&mut self, amount: i32) -> Result<(), GameError> {
        if self.phase != Phase::Auction {
            return Err(GameError::InvalidPhase);
        }
        let auction = self.auction.as_mut().ok_or(GameError::InvalidPhase)?;
        let bidder = auction.current_bidder().ok_or(GameError::InvalidPhase)?;
        let minimum = if auction.high_bid == 0 {
            10
        } else {
            auction.high_bid + 10
        };
        if amount < minimum || amount > self.players[bidder].cash {
            return Err(GameError::InvalidAction(format!(
                "bid must be at least {minimum} and affordable"
            )));
        }
        auction.high_bid = amount;
        auction.high_bidder = Some(bidder);
        auction.bidder_index = (auction.bidder_index + 1) % auction.active.len();
        if auction.active.len() > 1 && auction.current_bidder() == auction.high_bidder {
            auction.bidder_index = (auction.bidder_index + 1) % auction.active.len();
        }
        self.normalize_auction();
        Ok(())
    }

    fn auction_pass(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Auction {
            return Err(GameError::InvalidPhase);
        }
        let auction = self.auction.as_mut().ok_or(GameError::InvalidPhase)?;
        if auction.active.is_empty() {
            return Err(GameError::InvalidPhase);
        }
        let removed = auction.active.remove(auction.bidder_index);
        debug_assert_ne!(auction.high_bidder, Some(removed));
        if !auction.active.is_empty() {
            auction.bidder_index %= auction.active.len();
            if auction.active.len() > 1 && auction.current_bidder() == auction.high_bidder {
                auction.bidder_index = (auction.bidder_index + 1) % auction.active.len();
            }
        }
        self.normalize_auction();
        Ok(())
    }

    fn normalize_auction(&mut self) {
        let finish = match self.auction.as_ref() {
            Some(a) if a.active.is_empty() => true,
            Some(a) if a.active.len() == 1 && a.high_bidder == Some(a.active[0]) => true,
            _ => false,
        };
        if !finish {
            return;
        }
        let auction = self.auction.take().unwrap();
        if let Some(winner) = auction.high_bidder {
            self.players[winner].cash -= auction.high_bid;
            self.assets.get_mut(&auction.tile).unwrap().owner = Some(winner);
            self.logs.push(GameLog::Bought {
                player: winner,
                tile: auction.tile,
                price: auction.high_bid,
            });
        }
        if let Some(tile) = self.pending_bank_auctions.pop() {
            self.start_auction(tile);
        } else {
            self.phase = Phase::Manage;
        }
    }

    fn build(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = asset(tile).ok_or(GameError::UnknownAsset)?;
        let AssetKind::Street { group } = definition.kind else {
            return Err(GameError::InvalidAction(
                "only streets can have houses".into(),
            ));
        };
        let player = self.current_player;
        if !self.owns_group(player, group) {
            return Err(GameError::InvalidAction(
                "complete color group required".into(),
            ));
        }
        if self.group_tiles(group).any(|t| self.assets[&t].mortgaged) {
            return Err(GameError::InvalidAction(
                "a mortgaged group cannot be improved".into(),
            ));
        }
        let current = self.assets[&tile].houses;
        let min = self
            .group_tiles(group)
            .map(|t| self.assets[&t].houses)
            .min()
            .unwrap_or(0);
        if current >= MAX_HOUSES || current != min {
            return Err(GameError::InvalidAction(
                "houses must be built evenly".into(),
            ));
        }
        if self.players[player].cash < definition.house_cost {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].cash -= definition.house_cost;
        let state = self.assets.get_mut(&tile).unwrap();
        state.houses += 1;
        self.logs.push(GameLog::Built {
            player,
            tile,
            houses: state.houses,
        });
        Ok(())
    }

    fn sell_house(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = asset(tile).ok_or(GameError::UnknownAsset)?;
        let AssetKind::Street { group } = definition.kind else {
            return Err(GameError::InvalidAction(
                "only streets can have houses".into(),
            ));
        };
        let player = self.current_player;
        if self.assets[&tile].owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        let current = self.assets[&tile].houses;
        let max = self
            .group_tiles(group)
            .map(|t| self.assets[&t].houses)
            .max()
            .unwrap_or(0);
        if current == 0 || current != max {
            return Err(GameError::InvalidAction(
                "houses must be sold evenly".into(),
            ));
        }
        self.players[player].cash += definition.house_cost / 2;
        let state = self.assets.get_mut(&tile).unwrap();
        state.houses -= 1;
        self.logs.push(GameLog::SoldHouse {
            player,
            tile,
            houses: state.houses,
        });
        Ok(())
    }

    fn mortgage(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = asset(tile).ok_or(GameError::UnknownAsset)?;
        let player = self.current_player;
        let state = &self.assets[&tile];
        if state.owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        if state.mortgaged {
            return Err(GameError::InvalidAction("already mortgaged".into()));
        }
        if let AssetKind::Street { group } = definition.kind {
            if self.group_tiles(group).any(|t| self.assets[&t].houses > 0) {
                return Err(GameError::InvalidAction(
                    "sell all houses in the group first".into(),
                ));
            }
        }
        self.assets.get_mut(&tile).unwrap().mortgaged = true;
        self.players[player].cash += definition.mortgage_value();
        self.logs.push(GameLog::Mortgaged { player, tile });
        Ok(())
    }

    fn unmortgage(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = asset(tile).ok_or(GameError::UnknownAsset)?;
        let player = self.current_player;
        let state = &self.assets[&tile];
        if state.owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        if !state.mortgaged {
            return Err(GameError::InvalidAction("asset is not mortgaged".into()));
        }
        let cost = (definition.mortgage_value() * 110 + 99) / 100;
        if self.players[player].cash < cost {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].cash -= cost;
        self.assets.get_mut(&tile).unwrap().mortgaged = false;
        self.logs.push(GameLog::Unmortgaged { player, tile });
        Ok(())
    }

    fn end_turn(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        if self.extra_turn && !self.players[self.current_player].bankrupt {
            self.extra_turn = false;
            self.phase = Phase::AwaitRoll;
            return Ok(());
        }
        self.doubles_streak = 0;
        self.last_roll = None;
        let old = self.current_player;
        let mut next = old;
        loop {
            next = (next + 1) % self.players.len();
            if !self.players[next].bankrupt || next == old {
                break;
            }
        }
        if next <= old {
            if self.round >= self.config.round_limit {
                self.finish_by_net_worth();
                return Ok(());
            }
            self.round += 1;
        }
        self.current_player = next;
        self.phase = Phase::AwaitRoll;
        self.check_last_survivor();
        Ok(())
    }

    fn group_tiles(&self, group: u8) -> impl Iterator<Item = usize> + '_ {
        ASSETS
            .iter()
            .filter_map(move |definition| match definition.kind {
                AssetKind::Street { group: candidate } if candidate == group => {
                    Some(definition.tile)
                }
                _ => None,
            })
    }

    pub fn owns_group(&self, player: usize, group: u8) -> bool {
        self.group_tiles(group)
            .all(|tile| self.assets[&tile].owner == Some(player))
    }

    pub fn rent_for(&self, tile: usize) -> i32 {
        let definition = asset(tile).expect("board asset must exist");
        let state = &self.assets[&tile];
        let owner = match state.owner {
            Some(owner) => owner,
            None => return 0,
        };
        if state.mortgaged {
            return 0;
        }
        match definition.kind {
            AssetKind::Street { group } => {
                if state.houses == 0 {
                    definition.base_rent * if self.owns_group(owner, group) { 2 } else { 1 }
                } else {
                    let multiplier = [1, 5, 15, 45, 80][state.houses as usize];
                    definition.base_rent * multiplier
                }
            }
            AssetKind::Station => {
                let count = ASSETS
                    .iter()
                    .filter(|a| {
                        a.kind == AssetKind::Station && self.assets[&a.tile].owner == Some(owner)
                    })
                    .count();
                25 * (1 << count.saturating_sub(1))
            }
            AssetKind::Utility => {
                let count = ASSETS
                    .iter()
                    .filter(|a| {
                        a.kind == AssetKind::Utility && self.assets[&a.tile].owner == Some(owner)
                    })
                    .count();
                let dice = self
                    .last_roll
                    .map(|(a, b)| a as i32 + b as i32)
                    .unwrap_or(0);
                dice * if count >= 2 { 10 } else { 4 }
            }
        }
    }

    fn draw_card(&mut self) {
        let player = self.current_player;
        let card = self.deck[self.deck_index];
        self.deck_index = (self.deck_index + 1) % self.deck.len();
        self.logs.push(GameLog::Drew { player, card });
        match card {
            Card::Gain50 => self.add_cash(player, 50),
            Card::Gain100 => self.add_cash(player, 100),
            Card::Gain150 => self.add_cash(player, 150),
            Card::Pay50 => self.charge(player, 50, None),
            Card::Pay100 => self.charge(player, 100, None),
            Card::Collect25Each => {
                for other in 0..self.players.len() {
                    if other != player && !self.players[other].bankrupt {
                        self.charge(other, 25, Some(player));
                    }
                }
            }
            Card::Pay25Each => {
                for other in 0..self.players.len() {
                    if other != player
                        && !self.players[other].bankrupt
                        && !self.players[player].bankrupt
                    {
                        self.charge(player, 25, Some(other));
                    }
                }
            }
            Card::AdvanceStart => {
                self.players[player].position = 0;
                self.add_cash(player, PASS_START_BONUS);
            }
            Card::AdvanceStation => {
                let current = self.players[player].position;
                let target = [7, 17].into_iter().find(|p| *p > current).unwrap_or(7);
                if target < current {
                    self.add_cash(player, PASS_START_BONUS);
                }
                self.players[player].position = target;
                self.logs.push(GameLog::Moved {
                    player,
                    position: target,
                });
                self.resolve_space(target);
            }
            Card::BackThree => {
                let target = (self.players[player].position + BOARD.len() - 3) % BOARD.len();
                self.players[player].position = target;
                self.logs.push(GameLog::Moved {
                    player,
                    position: target,
                });
                self.resolve_space(target);
            }
            Card::GoToJail => self.send_to_jail(player),
            Card::GetOutOfJail => self.players[player].get_out_cards += 1,
        }
    }

    fn add_cash(&mut self, player: usize, amount: i32) {
        self.players[player].cash += amount;
        self.logs.push(GameLog::Cash { player, amount });
    }

    fn send_to_jail(&mut self, player: usize) {
        self.players[player].position = 5;
        self.players[player].jail_turns = 1;
        self.logs.push(GameLog::Jailed { player });
    }

    fn charge(&mut self, player: usize, amount: i32, creditor: Option<usize>) {
        self.raise_cash(player, amount);
        let paid = amount.min(self.players[player].cash.max(0));
        self.players[player].cash -= paid;
        if let Some(to) = creditor {
            if !self.players[to].bankrupt {
                self.players[to].cash += paid;
            }
        }
        if paid < amount {
            self.bankrupt(player, creditor);
            if self.phase != Phase::GameOver && self.auction.is_none() {
                if let Some(tile) = self.pending_bank_auctions.pop() {
                    self.start_auction(tile);
                }
            }
        }
    }

    fn raise_cash(&mut self, player: usize, target: i32) {
        while self.players[player].cash < target {
            let house_tile = ASSETS
                .iter()
                .filter(|a| {
                    self.assets[&a.tile].owner == Some(player) && self.assets[&a.tile].houses > 0
                })
                .max_by_key(|a| self.assets[&a.tile].houses)
                .map(|a| a.tile);
            if let Some(tile) = house_tile {
                let definition = asset(tile).unwrap();
                self.assets.get_mut(&tile).unwrap().houses -= 1;
                self.players[player].cash += definition.house_cost / 2;
                continue;
            }
            let mortgage_tile = ASSETS
                .iter()
                .filter(|a| {
                    let state = &self.assets[&a.tile];
                    state.owner == Some(player) && !state.mortgaged && state.houses == 0
                })
                .min_by_key(|a| a.base_rent)
                .map(|a| a.tile);
            if let Some(tile) = mortgage_tile {
                self.assets.get_mut(&tile).unwrap().mortgaged = true;
                self.players[player].cash += asset(tile).unwrap().mortgage_value();
                continue;
            }
            break;
        }
    }

    fn bankrupt(&mut self, player: usize, creditor: Option<usize>) {
        self.players[player].bankrupt = true;
        self.players[player].cash = 0;
        for definition in &ASSETS {
            let state = self.assets.get_mut(&definition.tile).unwrap();
            if state.owner == Some(player) {
                state.houses = 0;
                state.owner = creditor.filter(|id| !self.players[*id].bankrupt);
                if state.owner.is_none() {
                    state.mortgaged = false;
                    self.pending_bank_auctions.push(definition.tile);
                }
            }
        }
        self.logs.push(GameLog::Bankrupt { player, creditor });
        self.check_last_survivor();
    }

    fn check_last_survivor(&mut self) {
        let alive: Vec<_> = self
            .players
            .iter()
            .filter(|p| !p.bankrupt)
            .map(|p| p.id)
            .collect();
        if alive.len() == 1 {
            self.winners = alive;
            self.phase = Phase::GameOver;
            self.logs.push(GameLog::Won {
                players: self.winners.clone(),
            });
        }
    }

    pub fn net_worth(&self, player: usize) -> i32 {
        let mut total = self.players[player].cash;
        for definition in &ASSETS {
            let state = &self.assets[&definition.tile];
            if state.owner == Some(player) {
                total += if state.mortgaged {
                    definition.mortgage_value()
                } else {
                    definition.price
                };
                total += state.houses as i32 * (definition.house_cost / 2);
            }
        }
        total
    }

    fn finish_by_net_worth(&mut self) {
        let best = self
            .players
            .iter()
            .filter(|p| !p.bankrupt)
            .map(|p| self.net_worth(p.id))
            .max()
            .unwrap_or(0);
        self.winners = self
            .players
            .iter()
            .filter(|p| !p.bankrupt && self.net_worth(p.id) == best)
            .map(|p| p.id)
            .collect();
        self.phase = Phase::GameOver;
        self.logs.push(GameLog::Won {
            players: self.winners.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(GameConfig::default()).unwrap()
    }

    #[test]
    fn validates_config() {
        let config = GameConfig {
            bot_count: 0,
            ..GameConfig::default()
        };
        assert!(config.validate().is_err());
        let config = GameConfig {
            round_limit: 19,
            ..GameConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn buying_and_group_rent_work() {
        let mut game = game();
        game.phase = Phase::OfferPurchase { tile: 1 };
        game.apply(Action::Buy).unwrap();
        assert_eq!(game.assets[&1].owner, Some(0));
        game.assets.get_mut(&3).unwrap().owner = Some(0);
        assert_eq!(game.rent_for(1), 20);
    }

    #[test]
    fn houses_must_be_even() {
        let mut game = game();
        game.assets.get_mut(&1).unwrap().owner = Some(0);
        game.assets.get_mut(&3).unwrap().owner = Some(0);
        game.phase = Phase::Manage;
        game.apply(Action::Build(1)).unwrap();
        assert!(game.apply(Action::Build(1)).is_err());
        game.apply(Action::Build(3)).unwrap();
    }

    #[test]
    fn mortgage_and_unmortgage_round_trip() {
        let mut game = game();
        game.assets.get_mut(&1).unwrap().owner = Some(0);
        game.phase = Phase::Manage;
        let before = game.players[0].cash;
        game.apply(Action::Mortgage(1)).unwrap();
        assert_eq!(game.players[0].cash, before + 50);
        game.apply(Action::Unmortgage(1)).unwrap();
        assert_eq!(game.players[0].cash, before - 5);
    }

    #[test]
    fn auction_awards_the_highest_bid() {
        let mut game = game();
        game.phase = Phase::OfferPurchase { tile: 1 };
        game.apply(Action::Decline).unwrap();
        game.apply(Action::AuctionBid(100)).unwrap();
        game.apply(Action::AuctionPass).unwrap();
        assert_eq!(game.assets[&1].owner, Some(0));
        assert_eq!(game.players[0].cash, START_CASH - 100);
        assert_eq!(game.phase, Phase::Manage);
    }

    #[test]
    fn utility_rent_uses_dice_and_ownership_count() {
        let mut game = game();
        game.last_roll = Some((3, 4));
        game.assets.get_mut(&9).unwrap().owner = Some(0);
        assert_eq!(game.rent_for(9), 28);
        game.assets.get_mut(&19).unwrap().owner = Some(0);
        assert_eq!(game.rent_for(9), 70);
    }

    #[test]
    fn third_failed_jail_roll_pays_and_moves() {
        let mut game = game();
        let seed = (1..10_000)
            .find(|seed| {
                let mut rng = SimpleRng::new(*seed);
                (0..3).all(|_| rng.die() != rng.die())
            })
            .unwrap();
        game.rng = SimpleRng::new(seed);
        game.players[0].position = 5;
        game.players[0].jail_turns = 1;
        for attempt in 0..3 {
            game.phase = Phase::AwaitRoll;
            game.apply(Action::Roll).unwrap();
            if attempt < 2 {
                assert!(game.players[0].jail_turns > 0);
            }
        }
        assert_eq!(game.players[0].jail_turns, 0);
        assert_eq!(game.players[0].cash, START_CASH - JAIL_FINE);
        assert_ne!(game.players[0].position, 5);
    }

    #[test]
    fn round_limit_uses_net_worth() {
        let mut game = game();
        game.round = game.config.round_limit;
        game.current_player = game.players.len() - 1;
        game.phase = Phase::Manage;
        game.players[1].cash += 1;
        game.apply(Action::EndTurn).unwrap();
        assert_eq!(game.phase, Phase::GameOver);
        assert_eq!(game.winners, vec![1]);
    }

    #[test]
    fn saveable_rng_is_deterministic() {
        let mut first = game();
        let mut second = first.clone();
        first.apply(Action::Roll).unwrap();
        second.apply(Action::Roll).unwrap();
        assert_eq!(first.last_roll, second.last_roll);
        assert_eq!(first.phase, second.phase);
    }

    #[test]
    fn assets_returned_to_bank_are_auctioned() {
        let mut game = Game::new(GameConfig {
            bot_count: 2,
            ..GameConfig::default()
        })
        .unwrap();
        game.assets.get_mut(&1).unwrap().owner = Some(0);
        game.players[0].cash = 0;
        game.charge(0, 1_000, None);
        assert!(game.players[0].bankrupt);
        assert_eq!(game.phase, Phase::Auction);
        assert_eq!(game.auction.as_ref().unwrap().tile, 1);
    }

    #[test]
    fn creditor_receives_assets_on_bankruptcy() {
        let mut game = Game::new(GameConfig {
            bot_count: 2,
            ..GameConfig::default()
        })
        .unwrap();
        game.assets.get_mut(&1).unwrap().owner = Some(0);
        game.players[0].cash = 0;
        game.charge(0, 1_000, Some(1));
        assert!(game.players[0].bankrupt);
        assert_eq!(game.assets[&1].owner, Some(1));
        assert!(game.auction.is_none());
    }
}
