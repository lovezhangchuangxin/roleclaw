use crate::domain::{EventLogEntry, GlobalGameData, SaveMeta, SaveSnapshot, WorldCard};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub saves_dir: PathBuf,
    pub world_cards_dir: PathBuf,
}

impl AppPaths {
    pub fn from_app(app: &AppHandle) -> Result<Self, String> {
        let base = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to get app data dir: {e}"))?;
        let root = base.join("game-data");
        let saves_dir = root.join("saves");
        let world_cards_dir = root.join("world-cards");
        fs::create_dir_all(&saves_dir).map_err(|e| format!("failed to ensure saves dir: {e}"))?;
        fs::create_dir_all(&world_cards_dir)
            .map_err(|e| format!("failed to ensure world-cards dir: {e}"))?;
        Ok(Self {
            saves_dir,
            world_cards_dir,
        })
    }

    pub fn save_dir(&self, save_id: &str) -> PathBuf {
        self.saves_dir.join(save_id)
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn now_id(prefix: &str) -> String {
    let ms = Utc::now().timestamp_millis();
    format!("{prefix}_{ms}")
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let serialized =
        serde_json::to_string_pretty(data).map_err(|e| format!("failed to serialize json: {e}"))?;
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| format!("invalid parent for {}", path.display()))?,
    )
    .map_err(|e| format!("failed to create parent dirs: {e}"))?;
    fs::write(&tmp, serialized).map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("failed to move tmp file to {}: {e}", path.display()))
}

pub fn append_ndjson<T: Serialize>(path: &Path, row: &T) -> Result<(), String> {
    let payload = serde_json::to_string(row).map_err(|e| format!("failed to serialize row: {e}"))?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    writeln!(file, "{payload}").map_err(|e| format!("failed to append row: {e}"))
}

pub fn load_snapshot(paths: &AppPaths, save_id: &str) -> Result<SaveSnapshot, String> {
    read_json(&paths.save_dir(save_id).join("snapshot.json"))
}

pub fn load_meta(paths: &AppPaths, save_id: &str) -> Result<SaveMeta, String> {
    read_json(&paths.save_dir(save_id).join("meta.json"))
}

pub fn write_snapshot(paths: &AppPaths, snapshot: &SaveSnapshot) -> Result<(), String> {
    write_json(&paths.save_dir(&snapshot.save_id).join("snapshot.json"), snapshot)
}

pub fn write_meta(paths: &AppPaths, meta: &SaveMeta) -> Result<(), String> {
    write_json(&paths.save_dir(&meta.id).join("meta.json"), meta)
}

pub fn collect_recent_logs(
    paths: &AppPaths,
    save_id: &str,
    max_count: usize,
) -> Result<Vec<EventLogEntry>, String> {
    let path = paths.save_dir(save_id).join("events.ndjson");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let mut rows: Vec<EventLogEntry> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<EventLogEntry>(line).map_err(|e| format!("bad ndjson row: {e}")))
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() > max_count {
        rows = rows.split_off(rows.len() - max_count);
    }
    Ok(rows)
}

pub fn list_world_cards(paths: &AppPaths) -> Result<Vec<WorldCard>, String> {
    let mut out = Vec::new();
    for entry in
        fs::read_dir(&paths.world_cards_dir).map_err(|e| format!("failed to read world-cards dir: {e}"))?
    {
        let p = entry.map_err(|e| format!("failed to read entry: {e}"))?.path();
        if p.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        out.push(read_json::<WorldCard>(&p)?);
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn list_saves(paths: &AppPaths) -> Result<Vec<SaveMeta>, String> {
    let mut saves = Vec::new();
    for entry in fs::read_dir(&paths.saves_dir).map_err(|e| format!("failed to read saves dir: {e}"))? {
        let entry = entry.map_err(|e| format!("failed to read save entry: {e}"))?;
        let path = entry.path().join("meta.json");
        if !path.exists() {
            continue;
        }
        saves.push(read_json::<SaveMeta>(&path)?);
    }
    saves.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(saves)
}

pub fn global_data_path(paths: &AppPaths) -> PathBuf {
    paths.saves_dir.parent().unwrap_or(&paths.saves_dir).join("global-data.json")
}

pub fn load_global_data(paths: &AppPaths) -> Result<GlobalGameData, String> {
    read_json(&global_data_path(paths))
}

pub fn write_global_data(paths: &AppPaths, data: &GlobalGameData) -> Result<(), String> {
    write_json(&global_data_path(paths), data)
}
