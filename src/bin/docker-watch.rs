use anyhow::{bail, Result};
use bollard::errors::Error;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::Docker;
use clap::{CommandFactory, Parser, Subcommand};
use docker_cli_plugins::docker::get_container_names;
use docker_cli_plugins::error::{NoContainersError, NoSelectionError};
use docker_cli_plugins::fzf::{fuzzy_search, SearchOptions};
use docker_cli_plugins::{generate_completion, CompletionArgs, Metadata, METADATA_CMD};
use futures_util::stream::TryStreamExt;
use log::{debug, error, info, warn};
use tokio::time;

const METADATA: Metadata = Metadata {
    schema_version: "0.1.0",
    vendor: "Mathieu Lemay (github.com/mathieu-lemay)",
    version: "1.0.0",
    short_description: "Watch for logs for a container",
};

#[derive(Parser)]
#[command(name = "docker-watch", about, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch for logs
    Watch {
        /// Query
        #[arg()]
        query: Option<String>,

        #[arg(short, long)]
        tail: bool,
    },
    /// Get the plugin metadata
    #[command(name = METADATA_CMD)]
    Metadata,
    /// Generate shell completion
    Completion(CompletionArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    docker_cli_plugins::init_logger();

    let args = Cli::parse();

    match args.command {
        Commands::Watch { query, tail } => watch_container_logs(query, tail).await,
        Commands::Metadata => {
            println!("{}", METADATA);
            Ok(())
        }
        Commands::Completion(args) => {
            generate_completion(Cli::command(), args.shell);
            Ok(())
        }
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
    loop {
        let names = match get_container_names(d, false).await {
            Ok(n) => n,
            Err(e) if e.is::<NoContainersError>() => {
                warn!("No running containers, waiting");
                time::sleep(time::Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => {
                bail!(e)
            }
        };

        let opts = SearchOptions {
            query: query.clone(),
            multi: false,
            ..Default::default()
        };

        let mut selected_items = match fuzzy_search(names, opts) {
            Ok(i) => i,
            Err(e) if e.is::<NoSelectionError>() => {
                return Ok(None);
            }
            Err(e) => bail!(e),
        };

        return Ok(selected_items.pop());
    }
}
