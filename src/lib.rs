pub mod docker;
pub mod error;
pub mod fzf;
pub mod prompt;

use std::fmt::Display;
use std::io;

use clap::{Args, Command};
use clap_complete::{generate, Shell};
use log::LevelFilter;
use serde_json::json;

pub const METADATA_CMD: &str = "docker-cli-plugin-metadata";

pub fn init_logger() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
}

pub struct Metadata {
    pub schema_version: &'static str,
    pub vendor: &'static str,
    pub version: &'static str,
    pub short_description: &'static str,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let j = json!({
            "SchemaVersion": self.schema_version,
            "Vendor": self.vendor,
            "Version": self.version,
            "ShortDescription": self.short_description,
        });

        write!(f, "{}", j)
    }
}

#[derive(Args)]
pub struct CompletionArgs {
    pub shell: Shell,
}

pub fn generate_completion(mut cmd: Command, shell: Shell) {
    let name = cmd.get_name().to_string();

    generate(shell, &mut cmd, name, &mut io::stdout());
}
