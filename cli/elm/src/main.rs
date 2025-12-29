use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name="elm", version, about="EVE Linux Manager (prototype CLI)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        #[arg(long)]
        schemas: PathBuf,
        #[arg(long)]
        channel: Option<PathBuf>,
        #[arg(long)]
        engine: Option<PathBuf>,
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        profile: Option<PathBuf>,
    },
    Engine {
        #[command(subcommand)]
        cmd: EngineCmd,
    },
    Prefix {
        #[command(subcommand)]
        cmd: PrefixCmd,
    },
    Install {
        #[command(subcommand)]
        cmd: InstallCmd,
    },
    Launch {
        #[arg(long)]
        proton_root: PathBuf,
        #[arg(long)]
        prefix: PathBuf,
        #[arg(long)]
        exe_rel: PathBuf,
        #[arg(last=true)]
        args: Vec<String>,
    },
    Snapshot {
        #[arg(long)]
        prefix: PathBuf,
        #[arg(long)]
        snapshots: PathBuf,
        #[arg(long)]
        name: String,
    },
    Rollback {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        prefix: PathBuf,
    },
}

#[derive(Subcommand)]
enum EngineCmd {
    Install {
        #[arg(long)]
        schemas: PathBuf,
        #[arg(long)]
        engine: PathBuf,
        #[arg(long)]
        engines_dir: PathBuf,
        #[arg(long)]
        downloads_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum PrefixCmd {
    Init {
        #[arg(long)]
        proton_root: PathBuf,
        #[arg(long)]
        prefix: PathBuf,
    },
}

#[derive(Subcommand)]
enum InstallCmd {
    /// Install EVE Online launcher into prefix
    Eve {
        #[arg(long)]
        proton_root: PathBuf,
        #[arg(long)]
        prefix: PathBuf,
        #[arg(long, default_value = "~/.local/share/elm/downloads")]
        downloads_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Validate { schemas, channel, engine, manifest, profile } => {
            if let Some(p) = channel {
                let _ = elm_core::config::load::load_channel(&p, &schemas)?;
                println!("OK: channel {}", p.display());
            }
            if let Some(p) = engine {
                let _ = elm_core::config::load::load_engine(&p, &schemas)?;
                println!("OK: engine {}", p.display());
            }
            if let Some(p) = manifest {
                let _ = elm_core::config::load::load_manifest(&p, &schemas)?;
                println!("OK: manifest {}", p.display());
            }
            if let Some(p) = profile {
                let _ = elm_core::config::load::load_profile(&p, &schemas)?;
                println!("OK: profile {}", p.display());
            }
        }
        Commands::Engine { cmd } => match cmd {
            EngineCmd::Install { schemas, engine, engines_dir, downloads_dir } => {
                let e = elm_core::config::load::load_engine(&engine, &schemas)?;
                let dist = elm_core::engine::install::ensure_engine_installed(&e, &engines_dir, &downloads_dir)?;
                println!("Installed engine dist at: {}", dist.display());
            }
        },
        Commands::Prefix { cmd } => match cmd {
            PrefixCmd::Init { proton_root, prefix } => {
                elm_core::prefix::ensure_prefix_initialized(&prefix, &proton_root).await?;
                println!("Prefix ready: {}", prefix.display());
            }
        },
        Commands::Install { cmd } => match cmd {
            InstallCmd::Eve { proton_root, prefix, downloads_dir } => {
                // Expand ~ in downloads_dir
                let downloads = if downloads_dir.starts_with("~") {
                    let home = std::env::var("HOME").unwrap_or_default();
                    PathBuf::from(downloads_dir.to_string_lossy().replacen("~", &home, 1))
                } else {
                    downloads_dir
                };
                let result = elm_core::installer::install_eve_launcher(&prefix, &proton_root, &downloads).await?;
                println!("EVE installation complete: {}", result.display());
            }
        },
        Commands::Launch { proton_root, prefix, exe_rel, args } => {
            let spec = elm_core::runtime::launch::LaunchSpec {
                proton_root,
                prefix_dir: prefix,
                exe_path_in_prefix: exe_rel,
                args,
                env: HashMap::new(),
            };
            elm_core::runtime::launch::launch(spec).await?;
        }
        Commands::Snapshot { prefix, snapshots, name } => {
            let out = elm_core::rollback::snapshot::snapshot_prefix(&prefix, &snapshots, &name)?;
            println!("Snapshot created: {}", out.display());
        }
        Commands::Rollback { snapshot, prefix } => {
            elm_core::rollback::restore::restore_prefix(&snapshot, &prefix)?;
            println!("Prefix restored: {}", prefix.display());
        }
    }

    Ok(())
}
