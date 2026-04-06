mod commits;
mod git_ops;
mod output;
mod version;

use clap::Parser;
use std::process;

/// Smart semver bumper — reads your git commits, suggests the next version.
///
/// tayra reads your git history since the last semver tag, parses Conventional
/// Commits, and prints the bump level and next version. Zero config.
#[derive(Parser, Debug)]
#[command(name = "tayra", version, about, long_about = None)]
struct Cli {
    /// Create a git tag with the suggested version at HEAD
    #[arg(long)]
    tag: bool,

    /// Machine-readable output — prints only the suggested version string
    #[arg(long)]
    ci: bool,

    /// Alias for --ci: print only the suggested version
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Verbose output — mark breaking commits and show extra context
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Dry run — show what --tag would do without creating the tag
    #[arg(long)]
    dry_run: bool,

    /// Tag prefix to use (default: auto-detect from existing tags, fallback "v")
    #[arg(long)]
    prefix: Option<String>,

    /// Path to the git repository (default: current directory)
    #[arg(long, default_value = ".")]
    path: String,
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let result = git_ops::analyze(&cli.path, cli.prefix.as_deref())?;

    let prefix = git_ops::detect_prefix(result.current_version.as_ref(), cli.prefix.as_deref());

    let machine_readable = cli.ci || cli.quiet;

    if machine_readable {
        println!("{}", output::format_ci(&result, &prefix));
    } else {
        print!("{}", output::format_full(&result, &prefix, cli.verbose));
    }

    if cli.tag {
        let suggested = output::compute_suggested(&result);
        let tag_name = format!("{prefix}{suggested}");

        if cli.dry_run {
            if !machine_readable {
                println!("\nDRY RUN: would create tag '{tag_name}' at HEAD.");
            }
        } else {
            git_ops::create_tag(&cli.path, &tag_name)?;
            if !machine_readable {
                println!("\nTag '{tag_name}' created at HEAD.");
            }
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
