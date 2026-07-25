use crate::game::{Game, Language};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SAVE_VERSION: u32 = 1;

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
        let project = ProjectDirs::from("org", "monopoly-cli", "monopoly_cli")
            .ok_or_else(|| SaveError::Io(io::Error::other("user data directory is unavailable")))?;
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
        let envelope: SaveEnvelope = serde_json::from_slice(&fs::read(path)?)?;
        if envelope.schema_version != SAVE_VERSION {
            return Err(SaveError::UnsupportedVersion(envelope.schema_version));
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
                    error: (envelope.schema_version != SAVE_VERSION)
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
        saves.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));
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
    use crate::game::{Action, GameConfig};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_store() -> SaveStore {
        let path = std::env::temp_dir().join(format!(
            "monopoly-cli-test-{}-{}-{}",
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
}
