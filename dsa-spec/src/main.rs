use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dsa-spec")]
#[command(about = "Generate code skeletons from DSA specifications")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate code output
    Generate,
    /// Validate a specification
    Validate,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate => {
            println!("Generate command (not yet implemented)");
        }
        Command::Validate => {
            println!("Validate command (not yet implemented)");
        }
    }
}
