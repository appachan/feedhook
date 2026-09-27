use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};

/// Feed URL to the IDs of the entries already processed.
pub type State = BTreeMap<String, BTreeSet<String>>;

pub fn load(path: &Path) -> Result<State> {
    match fs::read_to_string(path) {
        Ok(text) => {
            serde_json::from_str(&text).with_context(|| format!("invalid state {}", path.display()))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(State::new()),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

/// Writes to a temporary file and renames it, so that a crash never leaves a broken file.
pub fn save(path: &Path, state: &State) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(state)?)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))
}
