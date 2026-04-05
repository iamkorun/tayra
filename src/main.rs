mod commits;
mod git_ops;
mod output;
mod version;

use clap::Parser;
use std::process;

/// Smart semver bumper — reads your git commits, suggests the next version.
#[derive(Parser, Debug)]
#[command(name = "tayra", version, about)]
struct Cli {
    /// Create a git tag with the suggested version
    #[arg(long)]
    tag: bool,

    /// Machine-readable output (just the version string)
    #[arg(long)]
    ci: bool,

    /// Dry run — analyze and suggest without side effects (default behavior)
    #[arg(long)]
    dry_run: bool,

    /// Tag prefix to use (default: auto-detect from existing tags)
    #[arg(long)]
    prefix: Option<String>,

    /// Path to the git repository (default: current directory)
    #[arg(long, default_value = ".")]
    path: String,
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let result = git_ops::analyze(&cli.path)?;

    let prefix = git_ops::detect_prefix(result.current_version.as_ref(), cli.prefix.as_deref());

    if cli.ci {
        println!("{}", output::format_ci(&result, prefix));
    } else {
        print!("{}", output::format_full(&result, prefix));
    }

    if cli.tag && !cli.dry_run {
        let suggested = output::compute_suggested(&result);
        let tag_name = format!("{prefix}{suggested}");
        git_ops::create_tag(&cli.path, &tag_name)?;
        if !cli.ci {
            println!("\nTag '{tag_name}' created at HEAD.");
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
