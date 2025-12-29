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
    /// Launch EVE Online (auto-setup engine, prefix, and game)
    Run {
        /// Profile name (default: "default")
        #[arg(long, default_value = "default")]
        profile: String,
    },
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
        Commands::Run { profile } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let data_dir = PathBuf::from(format!("{home}/.local/share/elm"));
            let config_dir = std::env::var("ELM_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(format!("{home}/.config/elm")));
            let engines_dir = data_dir.join("engines");
            let prefixes_dir = data_dir.join("prefixes");
            let downloads_dir = data_dir.join("downloads");

            // Try to load manifest from config dir, fallback to bundled
            let manifest_path = config_dir.join("manifests/eve-online.json");
            let manifest: Option<elm_core::config::models::ManifestV1> = if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                Some(serde_json::from_str(&content)?)
            } else {
                None
            };

            // Get config from manifest or use defaults
            let engine_id = manifest.as_ref()
                .map(|m| m.engine.engine_ref.clone())
                .unwrap_or_else(|| "ge-proton-10-26".to_string());

            let exe_rel = manifest.as_ref()
                .and_then(|m| m.launch.entrypoints.first())
                .and_then(|e| e.path.clone())
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("drive_c/CCP/EVE/tq/bin64/exefile.exe"));

            let launch_args: Vec<String> = manifest.as_ref()
                .and_then(|m| m.launch.entrypoints.first())
                .and_then(|e| e.args.clone())
                .unwrap_or_else(|| vec!["/server:tranquility".to_string()]);

            let env_vars: HashMap<String, String> = manifest.as_ref()
                .and_then(|m| m.env.as_ref())
                .and_then(|e| e.base.clone())
                .unwrap_or_else(|| [
                    ("DXVK_ASYNC", "1"),
                    ("PROTON_NO_ESYNC", "1"),
                    ("PROTON_NO_FSYNC", "1"),
                ].into_iter().map(|(k,v)| (k.to_string(), v.to_string())).collect());

            let proton_root = engines_dir.join(&engine_id).join(format!("dist/GE-Proton10-26"));
            let prefix_dir = prefixes_dir.join(format!("eve-{}", profile));

            // 1. Ensure engine is installed
            if !proton_root.join("proton").exists() {
                println!("Engine not found. Run: elm engine install ...");
                return Err(anyhow::anyhow!("Engine not installed at {}", proton_root.display()));
            }
            println!("✓ Engine: {}", engine_id);

            // 2. Ensure prefix is initialized
            if !prefix_dir.join("pfx/drive_c").exists() {
                println!("Initializing prefix...");
                elm_core::prefix::ensure_prefix_initialized(&prefix_dir, &proton_root).await?;
            }
            println!("✓ Prefix: eve-{}", profile);

            // 3. Ensure EVE is installed
            let eve_exe = prefix_dir.join("pfx").join(&exe_rel);
            if !eve_exe.exists() {
                println!("Installing EVE Online...");
                elm_core::installer::install_eve_launcher(&prefix_dir, &proton_root, &downloads_dir).await?;
            }
            println!("✓ EVE ready");

            // 4. Launch with env from manifest
            if manifest.is_some() {
                println!("✓ Config loaded from {}", manifest_path.display());
            }
            println!("Launching EVE Online...");
            let spec = elm_core::runtime::launch::LaunchSpec {
                proton_root,
                prefix_dir,
                exe_path_in_prefix: exe_rel,
                args: launch_args,
                env: env_vars,
            };
            elm_core::runtime::launch::launch(spec).await?;
        }
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
