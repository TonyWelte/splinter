use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::Result;
use ros2tui::common::app::{App, AppArgs};

#[derive(Debug, Parser)]
#[command(name = "ros2tui")]
#[command(about = "A TUI for ROS2")]
#[command(version, about)]
struct CliArgs {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    #[command(name = "topic")]
    Topic(TopicArgs),
    Node,
}

#[derive(Debug, Args, Clone)]
struct TopicArgs {
    #[command(subcommand)]
    command: TopicCommands,
}

#[derive(Debug, Subcommand, Clone)]
enum TopicCommands {
    #[command(name = "list")]
    List,
    #[command(name = "echo")]
    Echo { name: String },
    #[command(name = "pub")]
    Pub { name: String, message: String },
    #[command(name = "hz")]
    Hz { name: String },
}

pub fn run(app: App) -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    app.run(terminal)?;
    ratatui::restore();

    Ok(())
}

fn main() -> Result<()> {
    // Get file from CLI arguments
    let args = CliArgs::parse();

    // Handle commands
    let app = match args.commands {
        Some(Commands::Topic(topic_args)) => match topic_args.command {
            TopicCommands::List => App::new(AppArgs::TopicList),
            TopicCommands::Echo { name } => App::new(AppArgs::RawMessage(name)),
            TopicCommands::Pub { name, message } => {
                App::new(AppArgs::TopicPublisher(name, message))
            }
            TopicCommands::Hz { name } => App::new(AppArgs::HzPlot(name)),
        },
        Some(Commands::Node) => App::new(AppArgs::NodeList),
        None => App::default(),
    };

    run(app)?;

    Ok(())
}
