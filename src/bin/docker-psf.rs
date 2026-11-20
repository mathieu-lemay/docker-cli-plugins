use anyhow::Result;
use bollard::plugin::ContainerSummary;
use bollard::{query_parameters, Docker};
use clap::{Parser, Subcommand};
use docker_cli_plugins::{Metadata, METADATA_CMD};
use tabled::settings::Style;
use tabled::{Table, Tabled};

const METADATA: Metadata = Metadata {
    schema_version: "0.1.0",
    vendor: "Mathieu Lemay (github.com/mathieu-lemay)",
    version: "1.0.0",
    short_description: "A better `docker ps`",
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
    /// List containers
    Psf {
        /// Show all containers (default shows just running)
        #[arg(short, long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Metadata => {
            println!("{}", METADATA);
            Ok(())
        }
        Commands::Psf { all } => list_containers(all).await,
    }
}

#[derive(Tabled)]
#[tabled(rename_all = "UPPERCASE")]
struct Container {
    names: String,
    command: String,
    status: String,
    image: String,
}

impl Container {
    fn from(c: ContainerSummary) -> Self {
        Self {
            names: Self::format_names(c.names),
            command: truncate(c.command.unwrap_or_default(), 32),
            status: c.status.unwrap_or_default(),
            image: Self::format_image(c.image),
        }
    }

    fn format_names(names: Option<Vec<String>>) -> String {
        let names = match names {
            Some(n) => n,
            None => return String::new(),
        };

        let n = names
            .iter()
            .map(|n| n.trim_start_matches('/'))
            .collect::<Vec<&str>>()
            .join(",");

        truncate(n, 48)
    }

    fn format_image(image: Option<String>) -> String {
        let image = match image {
            Some(i) => i,
            None => return String::new(),
        };

        let image = image.split('/').next_back().unwrap();

        truncate(image.to_string(), 32)
    }
}

async fn list_containers(all: bool) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    let opts = query_parameters::ListContainersOptions {
        all,
        ..Default::default()
    };

    let mut rows: Vec<Container> = Vec::new();

    let containers = d.list_containers(Some(opts)).await?;

    for c in containers {
        rows.push(Container::from(c));
    }

    let mut t = Table::new(rows);
    t.with(Style::empty());

    println!("{}", t);

    Ok(())
}

fn truncate(s: String, n: usize) -> String {
    if s.len() > n {
        return format!("{}...", &s[..n - 3]);
    }

    s
}
