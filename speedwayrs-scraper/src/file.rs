use std::{
    fs::{self, DirBuilder},
    path::PathBuf,
};

use anyhow::{Context, Result};

pub fn check_folder(path: PathBuf) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }

    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .with_context(|| format!("Cannot create folder {}.", path.display()))?;

    Ok(())
}
