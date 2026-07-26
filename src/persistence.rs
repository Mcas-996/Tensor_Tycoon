use crate::game::{Game, GameLog, Language, ModelState, Phase, MODELS};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SAVE_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveEnvelope {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub updated_at_ms: u128,
    pub game: Game,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub language: Language,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: Language::ZhCn,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveSummary {
    pub id: String,
    pub name: String,
    pub updated_at_ms: u128,
    pub round: u16,
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidName,
    UnsupportedVersion(u32),
    NotFound,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::InvalidName => write!(f, "save name must contain 1 to 32 characters"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported save version {version}"),
            Self::NotFound => write!(f, "save not found"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone)]
pub struct SaveStore {
    root: PathBuf,
}

impl SaveStore {
    pub fn discover() -> Result<Self, SaveError> {
        let project = ProjectDirs::from("org", "tensor-tycoon", "tensor_tycoon")
            .ok_or_else(|| SaveError::Io(io::Error::other("user data directory is unavailable")))?;
        let legacy = ProjectDirs::from("org", "monopoly-cli", "monopoly_cli").ok_or_else(|| {
            SaveError::Io(io::Error::other("legacy data directory is unavailable"))
        })?;
        migrate_legacy_root(legacy.data_local_dir(), project.data_local_dir())?;
        Ok(Self {
            root: project.data_local_dir().to_path_buf(),
        })
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn saves_dir(&self) -> PathBuf {
        self.root.join("saves")
    }

    fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    pub fn create(&self, name: &str, game: &Game) -> Result<String, SaveError> {
        validate_name(name)?;
        fs::create_dir_all(self.saves_dir())?;
        let base = Self::now_ms();
        let mut suffix = 0u32;
        let id = loop {
            let candidate = if suffix == 0 {
                format!("{base:x}")
            } else {
                format!("{base:x}-{suffix}")
            };
            if !self.path_for(&candidate).exists() {
                break candidate;
            }
            suffix += 1;
        };
        let envelope = SaveEnvelope {
            schema_version: SAVE_VERSION,
            id: id.clone(),
            name: name.trim().to_string(),
            updated_at_ms: Self::now_ms(),
            game: game.clone(),
        };
        self.write_atomic(&envelope)?;
        Ok(id)
    }

    pub fn overwrite(&self, id: &str, name: &str, game: &Game) -> Result<(), SaveError> {
        validate_id(id)?;
        validate_name(name)?;
        if !self.path_for(id).exists() {
            return Err(SaveError::NotFound);
        }
        let envelope = SaveEnvelope {
            schema_version: SAVE_VERSION,
            id: id.to_string(),
            name: name.trim().to_string(),
            updated_at_ms: Self::now_ms(),
            game: game.clone(),
        };
        self.write_atomic(&envelope)
    }

    pub fn load(&self, id: &str) -> Result<SaveEnvelope, SaveError> {
        validate_id(id)?;
        let path = self.path_for(id);
        if !path.exists() {
            return Err(SaveError::NotFound);
        }
        let mut envelope: SaveEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        if !matches!(envelope.schema_version, 1 | SAVE_VERSION) {
            return Err(SaveError::UnsupportedVersion(envelope.schema_version));
        }
        if envelope.schema_version == 1 {
            migrate_v1_game(&mut envelope.game);
            envelope.schema_version = SAVE_VERSION;
        }
        Ok(envelope)
    }

    pub fn list(&self) -> Result<Vec<SaveSummary>, SaveError> {
        if !self.saves_dir().exists() {
            return Ok(Vec::new());
        }
        let mut saves = Vec::new();
        for entry in fs::read_dir(self.saves_dir())? {
            let entry = entry?;
            if entry.path().extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            let fallback_id = entry
                .path()
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("?")
                .to_string();
            match serde_json::from_slice::<SaveEnvelope>(&fs::read(entry.path())?) {
                Ok(envelope) => saves.push(SaveSummary {
                    id: envelope.id,
                    name: envelope.name,
                    updated_at_ms: envelope.updated_at_ms,
                    round: envelope.game.round,
                    error: (!matches!(envelope.schema_version, 1 | SAVE_VERSION))
                        .then(|| format!("version {}", envelope.schema_version)),
                }),
                Err(error) => saves.push(SaveSummary {
                    id: fallback_id,
                    name: "<corrupt>".into(),
                    updated_at_ms: 0,
                    round: 0,
                    error: Some(error.to_string()),
                }),
            }
        }
        saves.sort_by_key(|save| std::cmp::Reverse(save.updated_at_ms));
        Ok(saves)
    }

    pub fn delete(&self, id: &str) -> Result<(), SaveError> {
        validate_id(id)?;
        let path = self.path_for(id);
        if !path.exists() {
            return Err(SaveError::NotFound);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn load_preferences(&self) -> Preferences {
        fs::read(self.root.join("config.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save_preferences(&self, preferences: &Preferences) -> Result<(), SaveError> {
        fs::create_dir_all(&self.root)?;
        let bytes = serde_json::to_vec_pretty(preferences)?;
        fs::write(self.root.join("config.json"), bytes)?;
        Ok(())
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.saves_dir().join(format!("{id}.json"))
    }

    fn write_atomic(&self, envelope: &SaveEnvelope) -> Result<(), SaveError> {
        fs::create_dir_all(self.saves_dir())?;
        let path = self.path_for(&envelope.id);
        let temp = self.saves_dir().join(format!(".{}.tmp", envelope.id));
        fs::write(&temp, serde_json::to_vec_pretty(envelope)?)?;
        if path.exists() {
            let backup = self.saves_dir().join(format!(".{}.bak", envelope.id));
            if backup.exists() {
                fs::remove_file(&backup)?;
            }
            fs::rename(&path, &backup)?;
            match fs::rename(&temp, &path) {
                Ok(()) => {
                    let _ = fs::remove_file(backup);
                }
                Err(error) => {
                    let _ = fs::rename(&backup, &path);
                    return Err(SaveError::Io(error));
                }
            }
        } else {
            fs::rename(temp, path)?;
        }
        Ok(())
    }
}

fn migrate_v1_game(game: &mut Game) {
    let old_models = std::mem::take(&mut game.models);
    game.models = MODELS
        .iter()
        .map(|definition| (definition.tile, ModelState::default()))
        .collect();
    for (old_tile, state) in old_models {
        if let Some(new_tile) = migrate_v1_tile(old_tile) {
            game.models.insert(new_tile, state);
        }
    }

    for player in &mut game.players {
        player.position = migrate_v1_position(player.position);
    }
    if let Phase::OfferPurchase { tile } = &mut game.phase {
        *tile = migrate_v1_tile(*tile).unwrap_or(*tile);
    }
    if let Some(auction) = &mut game.auction {
        auction.tile = migrate_v1_tile(auction.tile).unwrap_or(auction.tile);
    }
    for tile in &mut game.pending_bank_auctions {
        *tile = migrate_v1_tile(*tile).unwrap_or(*tile);
    }
    for log in &mut game.logs {
        match log {
            GameLog::Moved { position, .. } => {
                *position = migrate_v1_position(*position);
            }
            GameLog::Bought { tile, .. }
            | GameLog::TensorAllocated { tile, .. }
            | GameLog::ReleasedTensor { tile, .. }
            | GameLog::Archived { tile, .. }
            | GameLog::Restored { tile, .. } => {
                *tile = migrate_v1_tile(*tile).unwrap_or(*tile);
            }
            _ => {}
        }
    }
}

fn migrate_v1_tile(tile: usize) -> Option<usize> {
    Some(match tile {
        1 => 1,
        3 => 4,
        6 => 7,
        7 => 8,
        8 => 10,
        9 => 11,
        11 => 13,
        13 => 16,
        16 => 19,
        17 => 20,
        18 => 22,
        19 => 23,
        _ => return None,
    })
}

fn migrate_v1_position(position: usize) -> usize {
    const POSITIONS: [usize; 20] = [
        0, 1, 2, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23,
    ];
    POSITIONS.get(position).copied().unwrap_or(position % 24)
}

fn migrate_legacy_root(legacy: &std::path::Path, current: &std::path::Path) -> io::Result<()> {
    if current.exists() || !legacy.exists() {
        return Ok(());
    }
    let parent = current
        .parent()
        .ok_or_else(|| io::Error::other("data directory has no parent"))?;
    fs::create_dir_all(parent)?;
    let name = current
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("tensor_tycoon");
    let temp = parent.join(format!(
        ".{name}.migrating-{}-{}",
        std::process::id(),
        SaveStore::now_ms()
    ));
    copy_dir_recursive(legacy, &temp)?;
    fs::rename(temp, current)?;
    Ok(())
}

fn copy_dir_recursive(source: &std::path::Path, destination: &std::path::Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), SaveError> {
    let count = name.trim().chars().count();
    if !(1..=32).contains(&count) {
        Err(SaveError::InvalidName)
    } else {
        Ok(())
    }
}

fn validate_id(id: &str) -> Result<(), SaveError> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        Err(SaveError::NotFound)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Action, Difficulty, GameConfig};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_store() -> SaveStore {
        let path = std::env::temp_dir().join(format!(
            "tensor-tycoon-test-{}-{}-{}",
            std::process::id(),
            SaveStore::now_ms(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ));
        SaveStore::at(path)
    }

    #[test]
    fn round_trip_preserves_exact_game_state() {
        let store = temp_store();
        let mut game = Game::new(GameConfig::default()).unwrap();
        game.apply(Action::Roll).unwrap();
        let id = store.create("Test", &game).unwrap();
        let loaded = store.load(&id).unwrap();
        assert_eq!(loaded.game, game);
        store.delete(&id).unwrap();
    }

    #[test]
    fn version_two_save_without_difficulty_loads_as_standard() {
        let store = temp_store();
        fs::create_dir_all(store.saves_dir()).unwrap();
        let mut game = serde_json::to_value(Game::new(GameConfig::default()).unwrap()).unwrap();
        game["config"].as_object_mut().unwrap().remove("difficulty");
        let envelope = serde_json::json!({
            "schema_version": SAVE_VERSION,
            "id": "abc",
            "name": "Legacy v2",
            "updated_at_ms": 1,
            "game": game
        });
        fs::write(
            store.path_for("abc"),
            serde_json::to_vec_pretty(&envelope).unwrap(),
        )
        .unwrap();

        let loaded = store.load("abc").unwrap();

        assert_eq!(loaded.game.config.difficulty, Difficulty::Standard);
    }

    #[test]
    fn rejects_bad_names_and_ids() {
        let store = temp_store();
        let game = Game::new(GameConfig::default()).unwrap();
        assert!(store.create("", &game).is_err());
        assert!(store.load("../escape").is_err());
    }

    #[test]
    fn lists_and_overwrites_saves() {
        let store = temp_store();
        let mut game = Game::new(GameConfig::default()).unwrap();
        let id = store.create("First", &game).unwrap();
        game.round = 7;
        store.overwrite(&id, "Renamed", &game).unwrap();
        let saves = store.list().unwrap();
        assert_eq!(saves[0].name, "Renamed");
        assert_eq!(saves[0].round, 7);
        store.delete(&id).unwrap();
    }

    #[test]
    fn migrates_v1_board_state_and_serialized_names() {
        let store = temp_store();
        fs::create_dir_all(store.saves_dir()).unwrap();
        let mut game = serde_json::to_value(Game::new(GameConfig::default()).unwrap()).unwrap();
        let object = game.as_object_mut().unwrap();

        let current_models = object.remove("models").unwrap();
        let current_models = current_models.as_object().unwrap();
        let mut legacy_models = serde_json::Map::new();
        for (old_tile, new_tile) in [
            (1, 1),
            (3, 4),
            (6, 7),
            (7, 8),
            (8, 10),
            (9, 11),
            (11, 13),
            (13, 16),
            (16, 19),
            (17, 20),
            (18, 22),
            (19, 23),
        ] {
            let mut state = current_models[&new_tile.to_string()].clone();
            let state_object = state.as_object_mut().unwrap();
            let tensors = state_object.remove("tensors").unwrap();
            let archived = state_object.remove("archived").unwrap();
            state_object.insert("houses".into(), tensors);
            state_object.insert("mortgaged".into(), archived);
            legacy_models.insert(old_tile.to_string(), state);
        }
        legacy_models["1"]["owner"] = serde_json::json!(0);
        legacy_models["1"]["houses"] = serde_json::json!(2);
        object.insert("assets".into(), legacy_models.into());

        for player in object["players"].as_array_mut().unwrap() {
            let player = player.as_object_mut().unwrap();
            let credits = player.remove("credits").unwrap();
            let cooldown_turns = player.remove("cooldown_turns").unwrap();
            let bypass_tokens = player.remove("bypass_tokens").unwrap();
            player.insert("cash".into(), credits);
            player.insert("jail_turns".into(), cooldown_turns);
            player.insert("get_out_cards".into(), bypass_tokens);
            player.insert("position".into(), serde_json::json!(19));
        }
        for card in object["deck"].as_array_mut().unwrap() {
            if card == "AdvanceHub" {
                *card = serde_json::json!("AdvanceStart");
            } else if card == "AdvanceFlagship" {
                *card = serde_json::json!("AdvanceStation");
            } else if card == "EnterCooldown" {
                *card = serde_json::json!("GoToJail");
            } else if card == "BypassToken" {
                *card = serde_json::json!("GetOutOfJail");
            }
        }
        object.insert(
            "phase".into(),
            serde_json::json!({"OfferPurchase": {"tile": 19}}),
        );
        object.insert("pending_bank_auctions".into(), serde_json::json!([3]));
        object.insert(
            "logs".into(),
            serde_json::json!([
                {"Moved": {"player": 0, "position": 19}},
                {"Built": {"player": 0, "tile": 1, "houses": 2}}
            ]),
        );

        let envelope = serde_json::json!({
            "schema_version": 1,
            "id": "abc",
            "name": "Legacy",
            "updated_at_ms": 1,
            "game": game
        });
        fs::write(
            store.path_for("abc"),
            serde_json::to_vec_pretty(&envelope).unwrap(),
        )
        .unwrap();

        let loaded = store.load("abc").unwrap();
        assert_eq!(loaded.schema_version, SAVE_VERSION);
        assert_eq!(loaded.game.players[0].position, 23);
        assert_eq!(loaded.game.phase, Phase::OfferPurchase { tile: 23 });
        assert_eq!(loaded.game.pending_bank_auctions, vec![4]);
        assert_eq!(loaded.game.models[&1].owner, Some(0));
        assert_eq!(loaded.game.models[&1].tensors, 2);
        assert_eq!(loaded.game.models.len(), 16);
        assert_eq!(
            loaded.game.logs[0],
            GameLog::Moved {
                player: 0,
                position: 23
            }
        );
    }

    #[test]
    fn copies_legacy_data_once_without_removing_it() {
        let base = std::env::temp_dir().join(format!(
            "tensor-data-migration-{}-{}",
            std::process::id(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let legacy = base.join("legacy");
        let current = base.join("current");
        fs::create_dir_all(legacy.join("saves")).unwrap();
        fs::write(legacy.join("config.json"), b"legacy-config").unwrap();
        fs::write(legacy.join("saves").join("abc.json"), b"legacy-save").unwrap();

        migrate_legacy_root(&legacy, &current).unwrap();
        assert_eq!(
            fs::read(current.join("config.json")).unwrap(),
            b"legacy-config"
        );
        assert_eq!(
            fs::read(current.join("saves").join("abc.json")).unwrap(),
            b"legacy-save"
        );
        assert!(legacy.join("config.json").exists());

        fs::write(legacy.join("new.txt"), b"do-not-merge").unwrap();
        migrate_legacy_root(&legacy, &current).unwrap();
        assert!(!current.join("new.txt").exists());
    }
}
