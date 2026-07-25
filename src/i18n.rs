use crate::game::{model, Card, Game, GameLog, Language};

pub fn text(language: Language, key: &str) -> &'static str {
    let zh = match key {
        "title" => "张量大亨",
        "new_game" => "新游戏",
        "load_game" => "存档管理",
        "language" => "切换语言",
        "quit" => "退出",
        "help" => "帮助",
        "player_name" => "玩家名称",
        "bots" => "电脑玩家",
        "round_limit" => "回合上限",
        "start" => "开始游戏",
        "back" => "返回",
        "round" => "回合",
        "credits" => "算力点数",
        "worth" => "净资产",
        "phase" => "阶段",
        "event_log" => "事件日志",
        "command" => "命令",
        "saves" => "存档",
        "no_saves" => "暂无存档",
        "corrupt" => "损坏",
        "winner" => "获胜者",
        "resize" => "终端至少需要 80×24，请调整窗口大小",
        "confirm_quit" => "再次按 q 确认退出",
        "confirm_delete" => "再次按 d 确认删除",
        "models" => "模型管理",
        "controls" => "热键",
        "hub_tile" => "Hub",
        "seed_tile" => "随机种子",
        "compute_tile" => "算力账单",
        "cooldown_tile" => "冷却区",
        "cache_tile" => "缓存命中",
        "overflow_tile" => "上下文溢出",
        _ => "?",
    };
    let en = match key {
        "title" => "Tensor Tycoon",
        "new_game" => "New game",
        "load_game" => "Save manager",
        "language" => "Switch language",
        "quit" => "Quit",
        "help" => "Help",
        "player_name" => "Player name",
        "bots" => "Bots",
        "round_limit" => "Round limit",
        "start" => "Start game",
        "back" => "Back",
        "round" => "Round",
        "credits" => "Credits",
        "worth" => "Net worth",
        "phase" => "Phase",
        "event_log" => "Event log",
        "command" => "Command",
        "saves" => "Saves",
        "no_saves" => "No saves",
        "corrupt" => "Corrupt",
        "winner" => "Winner",
        "resize" => "Terminal must be at least 80×24; please resize",
        "confirm_quit" => "Press q again to quit",
        "confirm_delete" => "Press d again to delete",
        "models" => "Model manager",
        "controls" => "Controls",
        "hub_tile" => "Hub",
        "seed_tile" => "Random Seed",
        "compute_tile" => "Compute Bill",
        "cooldown_tile" => "Cooldown",
        "cache_tile" => "Cache Hit",
        "overflow_tile" => "Context Overflow",
        _ => "?",
    };
    match language {
        Language::ZhCn => zh,
        Language::En => en,
    }
}

pub fn card_name(language: Language, card: Card) -> &'static str {
    let (zh, en) = match card {
        Card::Gain50 => ("数据缓存返还 +50", "Dataset cache rebate +50"),
        Card::Gain100 => ("推理额度奖励 +100", "Inference credit grant +100"),
        Card::Gain150 => ("基准测试奖金 +150", "Benchmark prize +150"),
        Card::Pay50 => ("存储账单 -50", "Storage bill -50"),
        Card::Pay100 => ("GPU 维护费 -100", "GPU maintenance -100"),
        Card::Collect25Each => (
            "每位玩家贡献 25 点数",
            "Collect 25 credits from each player",
        ),
        Card::Pay25Each => (
            "向每位玩家分配 25 点数",
            "Allocate 25 credits to each player",
        ),
        Card::AdvanceHub => ("返回 Hub", "Return to Hub"),
        Card::AdvanceFlagship => ("前往下一旗舰模型", "Advance to next flagship"),
        Card::BackThree => ("回滚三个提交", "Roll back three commits"),
        Card::EnterCooldown => ("触发限流，进入冷却区", "Rate limited: enter cooldown"),
        Card::BypassToken => ("获得绕过令牌", "Receive a bypass token"),
    };
    match language {
        Language::ZhCn => zh,
        Language::En => en,
    }
}

pub fn log_line(game: &Game, language: Language, log: &GameLog) -> String {
    let player = |id: usize| game.players.get(id).map(|p| p.name.as_str()).unwrap_or("?");
    match (language, log) {
        (
            Language::ZhCn,
            GameLog::Rolled {
                player: id,
                first,
                second,
            },
        ) => format!("{} 掷出 {}+{}", player(*id), first, second),
        (
            Language::En,
            GameLog::Rolled {
                player: id,
                first,
                second,
            },
        ) => format!("{} rolled {}+{}", player(*id), first, second),
        (
            Language::ZhCn,
            GameLog::Moved {
                player: id,
                position,
            },
        ) => format!("{} 移动到第 {} 格", player(*id), position),
        (
            Language::En,
            GameLog::Moved {
                player: id,
                position,
            },
        ) => format!("{} moved to tile {}", player(*id), position),
        (
            Language::ZhCn,
            GameLog::Bought {
                player: id,
                tile,
                price,
            },
        ) => format!(
            "{} 以 {} 买下 {}",
            player(*id),
            price,
            model(*tile).map(|a| a.name(language)).unwrap_or("?")
        ),
        (
            Language::En,
            GameLog::Bought {
                player: id,
                tile,
                price,
            },
        ) => format!(
            "{} bought {} for {}",
            player(*id),
            model(*tile).map(|a| a.name(language)).unwrap_or("?"),
            price
        ),
        (Language::ZhCn, GameLog::UsageFee { from, to, amount }) => {
            format!("{} 向 {} 支付使用费 {}", player(*from), player(*to), amount)
        }
        (Language::En, GameLog::UsageFee { from, to, amount }) => {
            format!(
                "{} paid {} usage credits to {}",
                player(*from),
                amount,
                player(*to)
            )
        }
        (_, GameLog::Drew { player: id, card }) => {
            format!("{}: {}", player(*id), card_name(language, *card))
        }
        (Language::ZhCn, GameLog::CooldownStarted { player: id }) => {
            format!("{} 进入冷却区", player(*id))
        }
        (Language::En, GameLog::CooldownStarted { player: id }) => {
            format!("{} entered cooldown", player(*id))
        }
        (Language::ZhCn, GameLog::Bankrupt { player: id, .. }) => format!("{} 破产", player(*id)),
        (Language::En, GameLog::Bankrupt { player: id, .. }) => {
            format!("{} is bankrupt", player(*id))
        }
        (Language::ZhCn, GameLog::Won { players }) => format!(
            "获胜：{}",
            players
                .iter()
                .map(|id| player(*id))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        (Language::En, GameLog::Won { players }) => format!(
            "Winner: {}",
            players
                .iter()
                .map(|id| player(*id))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        (_, GameLog::Credits { player: id, amount }) => format!("{} {:+}", player(*id), amount),
        (_, GameLog::ComputeBill { player: id, amount }) => {
            format!("{} compute -{}", player(*id), amount)
        }
        (
            _,
            GameLog::TensorAllocated {
                player: id,
                tile,
                tensors,
            },
        ) => format!(
            "{} {} T{}",
            player(*id),
            model(*tile).map(|a| a.name(language)).unwrap_or("?"),
            tensors
        ),
        (
            _,
            GameLog::ReleasedTensor {
                player: id,
                tile,
                tensors,
            },
        ) => format!(
            "{} {} T{}",
            player(*id),
            model(*tile).map(|a| a.name(language)).unwrap_or("?"),
            tensors
        ),
        (_, GameLog::Archived { player: id, tile }) => format!(
            "{} archived {}",
            player(*id),
            model(*tile).map(|a| a.name(language)).unwrap_or("?")
        ),
        (_, GameLog::Restored { player: id, tile }) => format!(
            "{} restored {}",
            player(*id),
            model(*tile).map(|a| a.name(language)).unwrap_or("?")
        ),
    }
}
