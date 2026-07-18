use lazin_error::LazinResult;

use crate::common;
use std::path::Path;

pub fn list_workspaces(config_directory: Option<&Path>) -> LazinResult<()> {
    let config = common::parse_config(config_directory)?;

    let workspaces = config.workspaces;
    if workspaces.is_empty() {
        lazin_logger::warn!("No workspaces configured");
        return Ok(());
    }

    lazin_logger::info!("Configured workspaces:");
    for (idx, workspace) in workspaces.iter().enumerate() {
        let enumerator = idx + 1;
        lazin_logger::print!("{}. {}", enumerator, workspace.name.as_ref())
    }

    Ok(())
}
