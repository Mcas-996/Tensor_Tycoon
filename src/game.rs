use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const START_CREDITS: i32 = 1_500;
pub const PASS_START_BONUS: i32 = 200;
pub const COOLDOWN_FEE: i32 = 50;
pub const MAX_TENSORS: u8 = 4;

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
pub enum ModelFamily {
    Qwen,
    Llama,
    DeepSeek,
    Kimi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDef {
    pub tile: usize,
    pub short_name: &'static str,
    pub hf_id: &'static str,
    pub family: ModelFamily,
    pub parameter_count: u64,
    pub tensor_cost: i32,
}

impl ModelDef {
    pub fn name(&self, language: Language) -> &'static str {
        let _ = language;
        self.short_name
    }

    pub fn price(&self) -> i32 {
        model_price(self.parameter_count)
    }

    pub fn base_fee(&self) -> i32 {
        self.price() / 10
    }

    pub fn archive_value(&self) -> i32 {
        self.price() / 2
    }
}

pub fn model_price(parameter_count: u64) -> i32 {
    const MIN_PARAMS: f64 = 600_000_000.0;
    const MAX_PARAMS: f64 = 1_000_000_000_000.0;
    let params = (parameter_count as f64).clamp(MIN_PARAMS, MAX_PARAMS);
    let normalized =
        (params.log10() - MIN_PARAMS.log10()) / (MAX_PARAMS.log10() - MIN_PARAMS.log10());
    ((100.0 + normalized * 200.0) / 10.0).round() as i32 * 10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Hub,
    Model(usize),
    RandomSeed,
    ComputeBill(i32),
    Cooldown,
    CacheHit,
    ContextOverflow,
}

pub const BOARD: [Space; 24] = [
    Space::Hub,
    Space::Model(1),
    Space::Model(2),
    Space::RandomSeed,
    Space::Model(4),
    Space::Model(5),
    Space::Cooldown,
    Space::Model(7),
    Space::Model(8),
    Space::ComputeBill(100),
    Space::Model(10),
    Space::Model(11),
    Space::CacheHit,
    Space::Model(13),
    Space::Model(14),
    Space::RandomSeed,
    Space::Model(16),
    Space::Model(17),
    Space::ContextOverflow,
    Space::Model(19),
    Space::Model(20),
    Space::ComputeBill(200),
    Space::Model(22),
    Space::Model(23),
];

pub const MODELS: [ModelDef; 16] = [
    ModelDef {
        tile: 1,
        short_name: "Qwen3 0.6B",
        hf_id: "Qwen/Qwen3-0.6B",
        family: ModelFamily::Qwen,
        parameter_count: 600_000_000,
        tensor_cost: 50,
    },
    ModelDef {
        tile: 2,
        short_name: "Qwen3 1.7B",
        hf_id: "Qwen/Qwen3-1.7B",
        family: ModelFamily::Qwen,
        parameter_count: 1_700_000_000,
        tensor_cost: 50,
    },
    ModelDef {
        tile: 4,
        short_name: "Qwen3 4B",
        hf_id: "Qwen/Qwen3-4B",
        family: ModelFamily::Qwen,
        parameter_count: 4_000_000_000,
        tensor_cost: 50,
    },
    ModelDef {
        tile: 5,
        short_name: "Qwen3 32B",
        hf_id: "Qwen/Qwen3-32B",
        family: ModelFamily::Qwen,
        parameter_count: 32_000_000_000,
        tensor_cost: 50,
    },
    ModelDef {
        tile: 7,
        short_name: "Llama 3.2 1B",
        hf_id: "meta-llama/Llama-3.2-1B",
        family: ModelFamily::Llama,
        parameter_count: 1_000_000_000,
        tensor_cost: 100,
    },
    ModelDef {
        tile: 8,
        short_name: "Llama 3.2 3B",
        hf_id: "meta-llama/Llama-3.2-3B",
        family: ModelFamily::Llama,
        parameter_count: 3_000_000_000,
        tensor_cost: 100,
    },
    ModelDef {
        tile: 10,
        short_name: "Llama 3.1 8B",
        hf_id: "meta-llama/Meta-Llama-3.1-8B",
        family: ModelFamily::Llama,
        parameter_count: 8_000_000_000,
        tensor_cost: 100,
    },
    ModelDef {
        tile: 11,
        short_name: "Llama 3.3 70B",
        hf_id: "meta-llama/Llama-3.3-70B-Instruct",
        family: ModelFamily::Llama,
        parameter_count: 70_000_000_000,
        tensor_cost: 100,
    },
    ModelDef {
        tile: 13,
        short_name: "DS-R1 1.5B",
        hf_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B",
        family: ModelFamily::DeepSeek,
        parameter_count: 1_500_000_000,
        tensor_cost: 150,
    },
    ModelDef {
        tile: 14,
        short_name: "DS-R1 7B",
        hf_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-7B",
        family: ModelFamily::DeepSeek,
        parameter_count: 7_000_000_000,
        tensor_cost: 150,
    },
    ModelDef {
        tile: 16,
        short_name: "DS-R1 32B",
        hf_id: "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B",
        family: ModelFamily::DeepSeek,
        parameter_count: 32_000_000_000,
        tensor_cost: 150,
    },
    ModelDef {
        tile: 17,
        short_name: "DeepSeek-R1",
        hf_id: "deepseek-ai/DeepSeek-R1",
        family: ModelFamily::DeepSeek,
        parameter_count: 671_000_000_000,
        tensor_cost: 150,
    },
    ModelDef {
        tile: 19,
        short_name: "Moonlight 16B",
        hf_id: "moonshotai/Moonlight-16B-A3B-Instruct",
        family: ModelFamily::Kimi,
        parameter_count: 16_000_000_000,
        tensor_cost: 200,
    },
    ModelDef {
        tile: 20,
        short_name: "Kimi Linear 49B",
        hf_id: "moonshotai/Kimi-Linear-48B-A3B-Instruct",
        family: ModelFamily::Kimi,
        parameter_count: 49_000_000_000,
        tensor_cost: 200,
    },
    ModelDef {
        tile: 22,
        short_name: "Kimi Dev 73B",
        hf_id: "moonshotai/Kimi-Dev-72B",
        family: ModelFamily::Kimi,
        parameter_count: 73_000_000_000,
        tensor_cost: 200,
    },
    ModelDef {
        tile: 23,
        short_name: "Kimi K2 1T",
        hf_id: "moonshotai/Kimi-K2-Instruct",
        family: ModelFamily::Kimi,
        parameter_count: 1_000_000_000_000,
        tensor_cost: 200,
    },
];

pub fn model(tile: usize) -> Option<&'static ModelDef> {
    MODELS.iter().find(|item| item.tile == tile)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelState {
    pub owner: Option<usize>,
    #[serde(alias = "houses")]
    pub tensors: u8,
    #[serde(alias = "mortgaged")]
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub is_human: bool,
    #[serde(alias = "cash")]
    pub credits: i32,
    pub position: usize,
    #[serde(alias = "jail_turns")]
    pub cooldown_turns: u8,
    #[serde(alias = "get_out_cards")]
    pub bypass_tokens: u8,
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
    #[serde(alias = "AdvanceStart")]
    AdvanceHub,
    #[serde(alias = "AdvanceStation")]
    AdvanceFlagship,
    BackThree,
    #[serde(alias = "GoToJail")]
    EnterCooldown,
    #[serde(alias = "GetOutOfJail")]
    BypassToken,
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
    #[serde(alias = "Rent")]
    UsageFee {
        from: usize,
        to: usize,
        amount: i32,
    },
    #[serde(alias = "Cash")]
    Credits {
        player: usize,
        amount: i32,
    },
    #[serde(alias = "Tax")]
    ComputeBill {
        player: usize,
        amount: i32,
    },
    Drew {
        player: usize,
        card: Card,
    },
    #[serde(alias = "Jailed")]
    CooldownStarted {
        player: usize,
    },
    #[serde(alias = "Built")]
    TensorAllocated {
        player: usize,
        tile: usize,
        #[serde(alias = "houses")]
        tensors: u8,
    },
    #[serde(alias = "SoldHouse")]
    ReleasedTensor {
        player: usize,
        tile: usize,
        #[serde(alias = "houses")]
        tensors: u8,
    },
    #[serde(alias = "Mortgaged")]
    Archived {
        player: usize,
        tile: usize,
    },
    #[serde(alias = "Unmortgaged")]
    Restored {
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
    #[serde(alias = "assets")]
    pub models: BTreeMap<usize, ModelState>,
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
    UnknownModel,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(message) | Self::InvalidAction(message) => write!(f, "{message}"),
            Self::InvalidPhase => write!(f, "action is not available in the current phase"),
            Self::NotOwner => write!(f, "player does not own this model"),
            Self::InsufficientFunds => write!(f, "insufficient funds"),
            Self::UnknownModel => write!(f, "unknown model"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Roll,
    PayCooldown,
    UseBypass,
    Buy,
    Decline,
    AuctionBid(i32),
    AuctionPass,
    AllocateTensor(usize),
    ReleaseTensor(usize),
    Archive(usize),
    Restore(usize),
    EndTurn,
}

impl Game {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        config.validate()?;
        let mut players = vec![Player {
            id: 0,
            name: config.human_name.clone(),
            is_human: true,
            credits: START_CREDITS,
            position: 0,
            cooldown_turns: 0,
            bypass_tokens: 0,
            bankrupt: false,
        }];
        for index in 0..config.bot_count {
            players.push(Player {
                id: index as usize + 1,
                name: format!("Bot {}", index + 1),
                is_human: false,
                credits: START_CREDITS,
                position: 0,
                cooldown_turns: 0,
                bypass_tokens: 0,
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
            Card::AdvanceHub,
            Card::AdvanceFlagship,
            Card::BackThree,
            Card::EnterCooldown,
            Card::BypassToken,
        ];
        for i in (1..deck.len()).rev() {
            let j = (rng.next() as usize) % (i + 1);
            deck.swap(i, j);
        }
        let models = MODELS
            .iter()
            .map(|definition| (definition.tile, ModelState::default()))
            .collect();
        Ok(Self {
            config,
            players,
            models,
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
            Action::PayCooldown => self.pay_cooldown(),
            Action::UseBypass => self.use_bypass(),
            Action::Buy => self.buy(),
            Action::Decline => self.decline(),
            Action::AuctionBid(amount) => self.auction_bid(amount),
            Action::AuctionPass => self.auction_pass(),
            Action::AllocateTensor(tile) => self.allocate_tensor(tile),
            Action::ReleaseTensor(tile) => self.release_tensor(tile),
            Action::Archive(tile) => self.archive(tile),
            Action::Restore(tile) => self.restore(tile),
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

        if self.players[player].cooldown_turns > 0 {
            if doubles {
                self.players[player].cooldown_turns = 0;
                self.extra_turn = false;
                self.doubles_streak = 0;
                self.move_by(first + second);
                return Ok(());
            }
            self.players[player].cooldown_turns += 1;
            if self.players[player].cooldown_turns > 3 {
                self.charge(player, COOLDOWN_FEE, None);
                if !self.players[player].bankrupt {
                    self.players[player].cooldown_turns = 0;
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
                self.send_to_cooldown(player);
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

    fn pay_cooldown(&mut self) -> Result<(), GameError> {
        let player = self.current_player;
        if self.phase != Phase::AwaitRoll || self.players[player].cooldown_turns == 0 {
            return Err(GameError::InvalidPhase);
        }
        if self.players[player].credits < COOLDOWN_FEE {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].credits -= COOLDOWN_FEE;
        self.players[player].cooldown_turns = 0;
        Ok(())
    }

    fn use_bypass(&mut self) -> Result<(), GameError> {
        let player = self.current_player;
        if self.phase != Phase::AwaitRoll
            || self.players[player].cooldown_turns == 0
            || self.players[player].bypass_tokens == 0
        {
            return Err(GameError::InvalidPhase);
        }
        self.players[player].bypass_tokens -= 1;
        self.players[player].cooldown_turns = 0;
        Ok(())
    }

    fn move_by(&mut self, amount: u8) {
        let player = self.current_player;
        let old = self.players[player].position;
        let new = (old + amount as usize) % BOARD.len();
        if new < old {
            self.players[player].credits += PASS_START_BONUS;
            self.logs.push(GameLog::Credits {
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
            Space::Hub | Space::Cooldown | Space::CacheHit => self.phase = Phase::Manage,
            Space::ComputeBill(amount) => {
                self.logs.push(GameLog::ComputeBill { player, amount });
                self.charge(player, amount, None);
                if !matches!(self.phase, Phase::GameOver | Phase::Auction) {
                    self.phase = Phase::Manage;
                }
            }
            Space::ContextOverflow => {
                self.send_to_cooldown(player);
                self.extra_turn = false;
                self.phase = Phase::Manage;
            }
            Space::RandomSeed => {
                self.draw_card();
                if !matches!(
                    self.phase,
                    Phase::GameOver | Phase::Auction | Phase::OfferPurchase { .. }
                ) {
                    self.phase = Phase::Manage;
                }
            }
            Space::Model(tile) => {
                let state = &self.models[&tile];
                match state.owner {
                    None => self.phase = Phase::OfferPurchase { tile },
                    Some(owner) if owner == player || state.archived => self.phase = Phase::Manage,
                    Some(owner) => {
                        let usage_fee = self.usage_fee_for(tile);
                        self.logs.push(GameLog::UsageFee {
                            from: player,
                            to: owner,
                            amount: usage_fee,
                        });
                        self.charge(player, usage_fee, Some(owner));
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
        let price = model(tile).ok_or(GameError::UnknownModel)?.price();
        let player = self.current_player;
        if self.players[player].credits < price {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].credits -= price;
        self.models.get_mut(&tile).unwrap().owner = Some(player);
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
        if amount < minimum || amount > self.players[bidder].credits {
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
            self.players[winner].credits -= auction.high_bid;
            self.models.get_mut(&auction.tile).unwrap().owner = Some(winner);
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

    fn allocate_tensor(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = model(tile).ok_or(GameError::UnknownModel)?;
        let player = self.current_player;
        if self.models[&tile].owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        if !self.has_tensor_access(player, definition.family) {
            return Err(GameError::InvalidAction(
                "three models from the same family are required".into(),
            ));
        }
        if self
            .owned_family_tiles(player, definition.family)
            .any(|t| self.models[&t].archived)
        {
            return Err(GameError::InvalidAction(
                "an archived family cannot allocate tensors".into(),
            ));
        }
        let current = self.models[&tile].tensors;
        let min = self
            .owned_family_tiles(player, definition.family)
            .map(|t| self.models[&t].tensors)
            .min()
            .unwrap_or(0);
        if current >= MAX_TENSORS || current != min {
            return Err(GameError::InvalidAction(
                "tensors must be allocated evenly".into(),
            ));
        }
        if self.players[player].credits < definition.tensor_cost {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].credits -= definition.tensor_cost;
        let state = self.models.get_mut(&tile).unwrap();
        state.tensors += 1;
        self.logs.push(GameLog::TensorAllocated {
            player,
            tile,
            tensors: state.tensors,
        });
        Ok(())
    }

    fn release_tensor(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = model(tile).ok_or(GameError::UnknownModel)?;
        let player = self.current_player;
        if self.models[&tile].owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        let current = self.models[&tile].tensors;
        let max = self
            .owned_family_tiles(player, definition.family)
            .map(|t| self.models[&t].tensors)
            .max()
            .unwrap_or(0);
        if current == 0 || current != max {
            return Err(GameError::InvalidAction(
                "tensors must be released evenly".into(),
            ));
        }
        self.players[player].credits += definition.tensor_cost / 2;
        let state = self.models.get_mut(&tile).unwrap();
        state.tensors -= 1;
        self.logs.push(GameLog::ReleasedTensor {
            player,
            tile,
            tensors: state.tensors,
        });
        Ok(())
    }

    fn archive(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = model(tile).ok_or(GameError::UnknownModel)?;
        let player = self.current_player;
        let state = &self.models[&tile];
        if state.owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        if state.archived {
            return Err(GameError::InvalidAction("already archived".into()));
        }
        if self
            .owned_family_tiles(player, definition.family)
            .any(|t| self.models[&t].tensors > 0)
        {
            return Err(GameError::InvalidAction(
                "release all tensors in the family first".into(),
            ));
        }
        self.models.get_mut(&tile).unwrap().archived = true;
        self.players[player].credits += definition.archive_value();
        self.logs.push(GameLog::Archived { player, tile });
        Ok(())
    }

    fn restore(&mut self, tile: usize) -> Result<(), GameError> {
        if self.phase != Phase::Manage {
            return Err(GameError::InvalidPhase);
        }
        let definition = model(tile).ok_or(GameError::UnknownModel)?;
        let player = self.current_player;
        let state = &self.models[&tile];
        if state.owner != Some(player) {
            return Err(GameError::NotOwner);
        }
        if !state.archived {
            return Err(GameError::InvalidAction("model is not archived".into()));
        }
        let cost = (definition.archive_value() * 110 + 99) / 100;
        if self.players[player].credits < cost {
            return Err(GameError::InsufficientFunds);
        }
        self.players[player].credits -= cost;
        self.models.get_mut(&tile).unwrap().archived = false;
        self.logs.push(GameLog::Restored { player, tile });
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

    fn family_tiles(&self, family: ModelFamily) -> impl Iterator<Item = usize> + '_ {
        MODELS
            .iter()
            .filter(move |definition| definition.family == family)
            .map(|definition| definition.tile)
    }

    fn owned_family_tiles(
        &self,
        player: usize,
        family: ModelFamily,
    ) -> impl Iterator<Item = usize> + '_ {
        self.family_tiles(family)
            .filter(move |tile| self.models[tile].owner == Some(player))
    }

    pub fn family_model_count(&self, player: usize, family: ModelFamily) -> usize {
        self.owned_family_tiles(player, family).count()
    }

    pub fn has_tensor_access(&self, player: usize, family: ModelFamily) -> bool {
        self.family_model_count(player, family) >= 3
    }

    pub fn usage_fee_for(&self, tile: usize) -> i32 {
        let definition = model(tile).expect("board model must exist");
        let state = &self.models[&tile];
        let owner = match state.owner {
            Some(owner) => owner,
            None => return 0,
        };
        if state.archived {
            return 0;
        }
        if state.tensors == 0 {
            definition.base_fee()
                * if self.has_tensor_access(owner, definition.family) {
                    2
                } else {
                    1
                }
        } else {
            let multiplier = [1, 5, 15, 45, 80][state.tensors as usize];
            definition.base_fee() * multiplier
        }
    }

    fn draw_card(&mut self) {
        let player = self.current_player;
        let card = self.deck[self.deck_index];
        self.deck_index = (self.deck_index + 1) % self.deck.len();
        self.logs.push(GameLog::Drew { player, card });
        match card {
            Card::Gain50 => self.add_credits(player, 50),
            Card::Gain100 => self.add_credits(player, 100),
            Card::Gain150 => self.add_credits(player, 150),
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
            Card::AdvanceHub => {
                self.players[player].position = 0;
                self.add_credits(player, PASS_START_BONUS);
            }
            Card::AdvanceFlagship => {
                let current = self.players[player].position;
                let target = [5, 11, 17, 23]
                    .into_iter()
                    .find(|p| *p > current)
                    .unwrap_or(5);
                if target < current {
                    self.add_credits(player, PASS_START_BONUS);
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
            Card::EnterCooldown => self.send_to_cooldown(player),
            Card::BypassToken => self.players[player].bypass_tokens += 1,
        }
    }

    fn add_credits(&mut self, player: usize, amount: i32) {
        self.players[player].credits += amount;
        self.logs.push(GameLog::Credits { player, amount });
    }

    fn send_to_cooldown(&mut self, player: usize) {
        self.players[player].position = 6;
        self.players[player].cooldown_turns = 1;
        self.logs.push(GameLog::CooldownStarted { player });
    }

    fn charge(&mut self, player: usize, amount: i32, creditor: Option<usize>) {
        self.raise_credits(player, amount);
        let paid = amount.min(self.players[player].credits.max(0));
        self.players[player].credits -= paid;
        if let Some(to) = creditor {
            if !self.players[to].bankrupt {
                self.players[to].credits += paid;
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

    fn raise_credits(&mut self, player: usize, target: i32) {
        while self.players[player].credits < target {
            let tensor_tile = MODELS
                .iter()
                .filter(|a| {
                    self.models[&a.tile].owner == Some(player) && self.models[&a.tile].tensors > 0
                })
                .max_by_key(|a| self.models[&a.tile].tensors)
                .map(|a| a.tile);
            if let Some(tile) = tensor_tile {
                let definition = model(tile).unwrap();
                self.models.get_mut(&tile).unwrap().tensors -= 1;
                self.players[player].credits += definition.tensor_cost / 2;
                continue;
            }
            let archive_tile = MODELS
                .iter()
                .filter(|a| {
                    let state = &self.models[&a.tile];
                    state.owner == Some(player) && !state.archived && state.tensors == 0
                })
                .min_by_key(|a| a.base_fee())
                .map(|a| a.tile);
            if let Some(tile) = archive_tile {
                self.models.get_mut(&tile).unwrap().archived = true;
                self.players[player].credits += model(tile).unwrap().archive_value();
                continue;
            }
            break;
        }
    }

    fn bankrupt(&mut self, player: usize, creditor: Option<usize>) {
        self.players[player].bankrupt = true;
        self.players[player].credits = 0;
        for definition in &MODELS {
            let state = self.models.get_mut(&definition.tile).unwrap();
            if state.owner == Some(player) {
                state.tensors = 0;
                state.owner = creditor.filter(|id| !self.players[*id].bankrupt);
                if state.owner.is_none() {
                    state.archived = false;
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
        let mut total = self.players[player].credits;
        for definition in &MODELS {
            let state = &self.models[&definition.tile];
            if state.owner == Some(player) {
                total += if state.archived {
                    definition.archive_value()
                } else {
                    definition.price()
                };
                total += state.tensors as i32 * (definition.tensor_cost / 2);
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
    fn model_roster_and_log_prices_are_stable() {
        assert_eq!(BOARD.len(), 24);
        assert_eq!(MODELS.len(), 16);
        let prices: Vec<_> = MODELS.iter().map(ModelDef::price).collect();
        assert_eq!(
            prices,
            vec![100, 130, 150, 210, 110, 140, 170, 230, 120, 170, 210, 290, 190, 220, 230, 300,]
        );
        assert!(MODELS
            .iter()
            .all(|definition| BOARD[definition.tile] == Space::Model(definition.tile)));
    }

    #[test]
    fn buying_and_family_usage_fee_work() {
        let mut game = game();
        game.phase = Phase::OfferPurchase { tile: 1 };
        game.apply(Action::Buy).unwrap();
        assert_eq!(game.models[&1].owner, Some(0));
        game.models.get_mut(&2).unwrap().owner = Some(0);
        game.models.get_mut(&4).unwrap().owner = Some(0);
        assert_eq!(game.usage_fee_for(1), 20);
    }

    #[test]
    fn tensors_must_be_even() {
        let mut game = game();
        for tile in [1, 2, 4] {
            game.models.get_mut(&tile).unwrap().owner = Some(0);
        }
        game.phase = Phase::Manage;
        game.apply(Action::AllocateTensor(1)).unwrap();
        assert!(game.apply(Action::AllocateTensor(1)).is_err());
        game.apply(Action::AllocateTensor(2)).unwrap();
        game.apply(Action::AllocateTensor(4)).unwrap();
    }

    #[test]
    fn archive_and_restore_round_trip() {
        let mut game = game();
        game.models.get_mut(&1).unwrap().owner = Some(0);
        game.phase = Phase::Manage;
        let before = game.players[0].credits;
        game.apply(Action::Archive(1)).unwrap();
        assert_eq!(game.players[0].credits, before + 50);
        game.apply(Action::Restore(1)).unwrap();
        assert_eq!(game.players[0].credits, before - 5);
    }

    #[test]
    fn auction_awards_the_highest_bid() {
        let mut game = game();
        game.phase = Phase::OfferPurchase { tile: 1 };
        game.apply(Action::Decline).unwrap();
        game.apply(Action::AuctionBid(100)).unwrap();
        game.apply(Action::AuctionPass).unwrap();
        assert_eq!(game.models[&1].owner, Some(0));
        assert_eq!(game.players[0].credits, START_CREDITS - 100);
        assert_eq!(game.phase, Phase::Manage);
    }

    #[test]
    fn fourth_model_must_catch_up_tensor_level() {
        let mut game = game();
        for tile in [1, 2, 4] {
            let state = game.models.get_mut(&tile).unwrap();
            state.owner = Some(0);
            state.tensors = 1;
        }
        game.models.get_mut(&5).unwrap().owner = Some(0);
        game.phase = Phase::Manage;
        assert!(game.apply(Action::AllocateTensor(1)).is_err());
        game.apply(Action::AllocateTensor(5)).unwrap();
    }

    #[test]
    fn third_failed_cooldown_roll_pays_and_moves() {
        let mut game = game();
        let seed = (1..10_000)
            .find(|seed| {
                let mut rng = SimpleRng::new(*seed);
                (0..3).all(|_| rng.die() != rng.die())
            })
            .unwrap();
        game.rng = SimpleRng::new(seed);
        game.players[0].position = 6;
        game.players[0].cooldown_turns = 1;
        for attempt in 0..3 {
            game.phase = Phase::AwaitRoll;
            game.apply(Action::Roll).unwrap();
            if attempt < 2 {
                assert!(game.players[0].cooldown_turns > 0);
            }
        }
        assert_eq!(game.players[0].cooldown_turns, 0);
        assert_eq!(game.players[0].credits, START_CREDITS - COOLDOWN_FEE);
        assert_ne!(game.players[0].position, 6);
    }

    #[test]
    fn round_limit_uses_net_worth() {
        let mut game = game();
        game.round = game.config.round_limit;
        game.current_player = game.players.len() - 1;
        game.phase = Phase::Manage;
        game.players[1].credits += 1;
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
    fn models_returned_to_bank_are_auctioned() {
        let mut game = Game::new(GameConfig {
            bot_count: 2,
            ..GameConfig::default()
        })
        .unwrap();
        game.models.get_mut(&1).unwrap().owner = Some(0);
        game.players[0].credits = 0;
        game.charge(0, 1_000, None);
        assert!(game.players[0].bankrupt);
        assert_eq!(game.phase, Phase::Auction);
        assert_eq!(game.auction.as_ref().unwrap().tile, 1);
    }

    #[test]
    fn creditor_receives_models_on_bankruptcy() {
        let mut game = Game::new(GameConfig {
            bot_count: 2,
            ..GameConfig::default()
        })
        .unwrap();
        game.models.get_mut(&1).unwrap().owner = Some(0);
        game.players[0].credits = 0;
        game.charge(0, 1_000, Some(1));
        assert!(game.players[0].bankrupt);
        assert_eq!(game.models[&1].owner, Some(1));
        assert!(game.auction.is_none());
    }
}
