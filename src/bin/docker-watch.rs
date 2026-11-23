use anyhow::{bail, Result};
use bollard::errors::Error;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::Docker;
use clap::{Parser, Subcommand};
use docker_cli_plugins::{Metadata, METADATA_CMD};
use futures_util::stream::TryStreamExt;
use log::{debug, error, info, warn};
use skim::options::SkimOptionsBuilder;
use skim::Skim;
use tokio::time;

const METADATA: Metadata = Metadata {
    schema_version: "0.1.0",
    vendor: "Mathieu Lemay (github.com/mathieu-lemay)",
    version: "1.0.0",
    short_description: "Watch for logs for a container",
};

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the plugin metadata
    #[command(name = METADATA_CMD)]
    Metadata,
    /// Watch for logs
    Watch {
        /// Query
        #[arg()]
        query: Option<String>,

        #[arg(short, long)]
        tail: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    docker_cli_plugins::init_logger();

    let args = Cli::parse();

    match args.command {
        Commands::Metadata => {
            println!("{}", METADATA);
            Ok(())
        }
        Commands::Watch { query, tail } => watch_container_logs(query, tail).await,
    }
}

async fn watch_container_logs(query: Option<String>, tail: bool) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    // Select a container from the running ones, or fallback to using the query.
    let container_name = match select_container(&d, query.clone()).await?.or(query) {
        Some(c) => c,
        None => {
            warn!("No container selected");
            return Ok(());
        }
    };

    info!("Selected container: {}", container_name);

    loop {
        let options = LogsOptionsBuilder::new()
            .stdout(true)
            .stderr(true)
            .follow(true)
            .tail(if tail { "0" } else { "all" })
            .build();
        let mut logs = d.logs(&container_name, Some(options));

        loop {
            let next = match logs.try_next().await {
                Ok(l) => l,
                // Container not found
                Err(Error::DockerResponseServerError {
                    status_code: 404,
                    message: m,
                }) => {
                    error!("Error getting logs from container: {m}");
                    time::sleep(time::Duration::from_secs(1)).await;
                    break;
                }
                // Conflict (the container is shutting down or stopped)
                Err(Error::DockerResponseServerError {
                    status_code: 409,
                    message: m,
                }) => {
                    error!("Error getting logs from container: {m}");
                    time::sleep(time::Duration::from_secs(1)).await;
                    break;
                }
                Err(e) => {
                    bail!(e);
                }
            };

            if let Some(l) = next {
                print!("{l}");
            } else {
                debug!("No more logs from container: {container_name}");
                break;
            }
        }
    }
}

async fn select_container(d: &Docker, query: Option<String>) -> Result<Option<String>> {
    let q = query.unwrap_or_default();

    loop {
        let containers = d.list_containers(None).await?;
        if containers.is_empty() {
            debug!("No running containers, waiting");
            time::sleep(time::Duration::from_secs(1)).await;
            continue;
        }

        let names = containers.iter().fold(Vec::new(), |mut v, i| {
            if let Some(names) = &i.names {
                for n in names {
                    v.push(n.trim_start_matches("/").to_string());
                }
            };

            v
        });

        let options = SkimOptionsBuilder::default()
            .height("50%")
            .select_1(true)
            // .exit_0(true)
            .reverse(true)
            .query(&q)
            .popup("center,75%,75%")
            .popup_name("docker watch")
            .build()?;

        let result = Skim::run_items(options, names).map(|out| out.selected_items);

        let selected_items = match result {
            Ok(i) => i,
            Err(e) => bail!(e),
        };

        if let Some(i) = selected_items.first() {
            return Ok(Some(i.item.text().to_string()));
        }

        debug!("No matching containers, waiting");
        time::sleep(time::Duration::from_secs(1)).await;
    }
}
