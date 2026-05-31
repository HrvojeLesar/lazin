use clap::crate_version;

pub(super) fn version() {
    lazin_logger::print!("Lazin version: {}", crate_version!())
}
