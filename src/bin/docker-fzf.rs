use anyhow::Result;
use bollard::query_parameters::{RemoveContainerOptionsBuilder, RemoveImageOptions};
use bollard::Docker;
use clap::{Args, CommandFactory, Parser, Subcommand};
use docker_cli_plugins::docker::{
    find_containers,
    find_images,
    FindContainersOptions,
    FindImagesOptions,
};
use docker_cli_plugins::prompt::prompt_yes_no;
use docker_cli_plugins::{generate_completion, CompletionArgs, Metadata, METADATA_CMD};
use log::{debug, info};

const METADATA: Metadata = Metadata {
    schema_version: "0.1.0",
    vendor: "Mathieu Lemay (github.com/mathieu-lemay)",
    version: "1.0.0",
    short_description: "Fuzzy find resources",
};

#[derive(Parser)]
#[command(name = "docker-fzf", about, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fuzzy find resources
    #[command(name = "fzf", subcommand)]
    FuzzyFind(FzfCommand),

    /// Get the plugin metadata
    #[command(name = METADATA_CMD)]
    Metadata,

    /// Generate shell completion
    Completion(CompletionArgs),
}

#[derive(Subcommand)]
enum FzfCommand {
    /// Remove Container(s)
    #[command(name = "rm")]
    RemoveContainers(RemoveContainersArgs),

    /// Remove Image(s)
    #[command(name = "rmi")]
    RemoveImages(BaseCommandArgs),

    /// Restart Container(s)
    #[command(name = "restart")]
    RestartContainers(BaseCommandArgs),

    /// Kill Container(s)
    #[command(name = "kill")]
    KillContainers(BaseCommandArgs),
}

#[derive(Args)]
struct RemoveContainersArgs {
    /// Initial fzf query
    #[arg()]
    query: Option<String>,

    /// Force the removal of a running container (uses SIGKILL)
    #[arg(short, long)]
    force: bool,

    /// Remove anonymous volumes associated with the container
    #[arg(short, long, default_value = "true")]
    volumes: bool,
}

#[derive(Args)]
struct BaseCommandArgs {
    /// Initial fzf query
    #[arg()]
    query: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    docker_cli_plugins::init_logger();

    let args = Cli::parse();

    match args.command {
        Commands::FuzzyFind(cmd) => match cmd {
            FzfCommand::RemoveContainers(args) => remove_containers(args).await,
            FzfCommand::RemoveImages(args) => remove_images(args).await,
            FzfCommand::RestartContainers(args) => restart_containers(args).await,
            FzfCommand::KillContainers(args) => kill_containers(args).await,
        },
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

async fn remove_containers(args: RemoveContainersArgs) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    let opts = FindContainersOptions {
        query: args.query,
        all: true,
        multi: true,
        pre_select: true,
    };

    let containers = find_containers(&d, opts).await?;

    info!("Selected container: \n  {}", containers.join("\n  "));

    let confirm = prompt_yes_no("Delete container(s)", true);

    if !confirm {
        return Ok(());
    }

    let rm_opt = RemoveContainerOptionsBuilder::new()
        .force(args.force)
        .v(args.volumes)
        .build();

    for c in containers {
        debug!("Removing container: {}", c);
        d.remove_container(&c, Some(rm_opt.clone())).await?;
    }

    Ok(())
}

async fn remove_images(args: BaseCommandArgs) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    let opts = FindImagesOptions {
        query: args.query,
        multi: true,
        pre_select: true,
    };

    let images = find_images(&d, opts).await?;

    info!("Selected images: \n  {}", images.join("\n  "));

    let confirm = prompt_yes_no("Delete images(s)", true);

    if !confirm {
        return Ok(());
    }

    for i in images {
        debug!("Removing image: {}", i);
        d.remove_image(&i, None::<RemoveImageOptions>, None).await?;
    }

    Ok(())
}

async fn restart_containers(args: BaseCommandArgs) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    let opts = FindContainersOptions {
        query: args.query,
        multi: true,
        pre_select: true,
        ..Default::default()
    };

    let containers = find_containers(&d, opts).await?;

    info!("Selected container: \n  {}", containers.join("\n  "));

    let confirm = prompt_yes_no("Restart container(s)", true);

    if !confirm {
        return Ok(());
    }

    for c in containers {
        debug!("Restarting container: {}", c);
        d.restart_container(&c, None).await?;
    }

    Ok(())
}

async fn kill_containers(args: BaseCommandArgs) -> Result<()> {
    let d = Docker::connect_with_defaults()?;

    let opts = FindContainersOptions {
        query: args.query,
        multi: true,
        pre_select: true,
        ..Default::default()
    };

    let containers = find_containers(&d, opts).await?;

    info!("Selected container: \n  {}", containers.join("\n  "));

    let confirm = prompt_yes_no("Kill container(s)", true);

    if !confirm {
        return Ok(());
    }

    for c in containers {
        debug!("Killing container: {}", c);
        d.kill_container(&c, None).await?;
    }

    Ok(())
}
