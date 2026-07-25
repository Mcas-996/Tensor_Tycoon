use crate::game::{asset, Card, Game, GameLog, Language};

pub fn text(language: Language, key: &str) -> &'static str {
    let zh = match key {
        "title" => "终端大富翁",
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
        "cash" => "现金",
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
        "assets" => "资产管理",
        "controls" => "热键",
        "start_tile" => "起点",
        "event_tile" => "事件",
        "tax_tile" => "税费",
        "jail_tile" => "监狱/探访",
        "free_tile" => "免费停车",
        "go_jail_tile" => "前往监狱",
        _ => "?",
    };
    let en = match key {
        "title" => "Terminal Tycoon",
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
        "cash" => "Cash",
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
        "assets" => "Asset manager",
        "controls" => "Controls",
        "start_tile" => "Start",
        "event_tile" => "Event",
        "tax_tile" => "Tax",
        "jail_tile" => "Jail / Visiting",
        "free_tile" => "Free Parking",
        "go_jail_tile" => "Go to Jail",
        _ => "?",
    };
    match language {
        Language::ZhCn => zh,
        Language::En => en,
    }
}

pub fn card_name(language: Language, card: Card) -> &'static str {
    let (zh, en) = match card {
        Card::Gain50 => ("社区奖励 +50", "Community award +50"),
        Card::Gain100 => ("银行分红 +100", "Bank dividend +100"),
        Card::Gain150 => ("幸运抽奖 +150", "Lucky draw +150"),
        Card::Pay50 => ("医疗费用 -50", "Medical fee -50"),
        Card::Pay100 => ("城市维护费 -100", "City maintenance -100"),
        Card::Collect25Each => ("每人支付你 25", "Collect 25 from everyone"),
        Card::Pay25Each => ("向每人支付 25", "Pay everyone 25"),
        Card::AdvanceStart => ("前往起点", "Advance to Start"),
        Card::AdvanceStation => ("前往下一车站", "Advance to next station"),
        Card::BackThree => ("后退三格", "Move back three"),
        Card::GoToJail => ("直接入狱", "Go directly to jail"),
        Card::GetOutOfJail => ("免狱卡", "Get out of jail"),
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
            asset(*tile).map(|a| a.name(language)).unwrap_or("?")
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
            asset(*tile).map(|a| a.name(language)).unwrap_or("?"),
            price
        ),
        (Language::ZhCn, GameLog::Rent { from, to, amount }) => {
            format!("{} 向 {} 支付租金 {}", player(*from), player(*to), amount)
        }
        (Language::En, GameLog::Rent { from, to, amount }) => {
            format!("{} paid {} rent to {}", player(*from), amount, player(*to))
        }
        (_, GameLog::Drew { player: id, card }) => {
            format!("{}: {}", player(*id), card_name(language, *card))
        }
        (Language::ZhCn, GameLog::Jailed { player: id }) => format!("{} 入狱", player(*id)),
        (Language::En, GameLog::Jailed { player: id }) => format!("{} went to jail", player(*id)),
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
        (_, GameLog::Cash { player: id, amount }) => format!("{} {:+}", player(*id), amount),
        (_, GameLog::Tax { player: id, amount }) => format!("{} tax -{}", player(*id), amount),
        (
            _,
            GameLog::Built {
                player: id,
                tile,
                houses,
            },
        ) => format!(
            "{} {} 🏠{}",
            player(*id),
            asset(*tile).map(|a| a.name(language)).unwrap_or("?"),
            houses
        ),
        (
            _,
            GameLog::SoldHouse {
                player: id,
                tile,
                houses,
            },
        ) => format!(
            "{} {} 🏠{}",
            player(*id),
            asset(*tile).map(|a| a.name(language)).unwrap_or("?"),
            houses
        ),
        (_, GameLog::Mortgaged { player: id, tile }) => format!(
            "{} mortgage {}",
            player(*id),
            asset(*tile).map(|a| a.name(language)).unwrap_or("?")
        ),
        (_, GameLog::Unmortgaged { player: id, tile }) => format!(
            "{} unmortgage {}",
            player(*id),
            asset(*tile).map(|a| a.name(language)).unwrap_or("?")
        ),
    }
}
