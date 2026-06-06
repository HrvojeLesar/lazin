use crate::{common, error::LazinResult};
use std::path::Path;

pub fn list_workspaces(config_directory: Option<&Path>) -> LazinResult<()> {
    let config = common::parse_config(config_directory)?;

    let mut workspaces = config.workspaces();
    if workspaces.is_empty() {
        lazin_logger::warn!("No workspaces configured");
        return Ok(());
    }

    workspaces.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));

    lazin_logger::info!("Configured workspaces:");
    for (idx, (workspace_name, _)) in workspaces.iter().enumerate() {
        lazin_logger::print!("{}. {}", idx + 1, workspace_name.str())
    }

    Ok(())
}
