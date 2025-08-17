use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

use ros2tui::verbs::{
    // mcap::{run as run_mcap, McapVerbArgs},
    ros2::run as run_ros2,
};

#[derive(Debug, Parser)]
#[command(name = "ros2tui")]
#[command(about = "A TUI for ROS2")]
#[command(version, about)]
struct CliArgs {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    // #[command(name = "mcap")]
    // Mcap(McapVerbArgs),
    #[command(name = "ros2")]
    ROS2,
}

fn main() -> Result<()> {
    // Get file from CLI arguments
    let args = CliArgs::parse();

    match args.commands {
        // Commands::Mcap(mcap_args) => run_mcap(mcap_args)?,
        Commands::ROS2 => run_ros2()?,
    }

    Ok(())
}
