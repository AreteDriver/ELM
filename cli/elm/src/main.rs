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
        /// Launch on Singularity (test server)
        #[arg(long, visible_alias = "sisi")]
        singularity: bool,
        /// Use DirectX 12 instead of DirectX 11
        #[arg(long)]
        dx12: bool,
        /// Send desktop notification when EVE closes
        #[arg(long)]
        notify: bool,
        /// Enable MangoHud performance overlay (FPS, GPU, CPU stats)
        #[arg(long)]
        hud: bool,
        /// MangoHud config (e.g., "fps,gpu_temp,cpu_temp,frametime")
        #[arg(long, default_value = "")]
        hud_config: String,
        /// Additional arguments to pass to EVE
        #[arg(long, num_args = 1..)]
        args: Vec<String>,
    },
    /// Show installed engines, prefixes, and snapshots
    Status,
    /// Check system compatibility and dependencies
    Doctor,
    /// View Wine/Proton and EVE logs
    Logs {
        /// Log type: launcher, wine, all (default: launcher)
        #[arg(long, default_value = "launcher")]
        log_type: String,
        /// Number of lines to show (default: 50)
        #[arg(long, short = 'n', default_value = "50")]
        lines: usize,
        /// List available log files without showing content
        #[arg(long)]
        list: bool,
        /// Profile name (default: "default")
        #[arg(long, default_value = "default")]
        profile: String,
    },
    /// Check for engine updates
    Update {
        /// Actually download and install the update
        #[arg(long)]
        install: bool,
        /// Skip automatic prefix backup before updating
        #[arg(long)]
        no_backup: bool,
        /// Send desktop notification if update available
        #[arg(long)]
        notify: bool,
    },
    /// Clean up old engines and download cache
    Clean {
        /// Only show what would be removed (dry run)
        #[arg(long)]
        dry_run: bool,
        /// Remove all downloaded archives
        #[arg(long)]
        downloads: bool,
        /// Remove old engine versions (keep latest)
        #[arg(long)]
        engines: bool,
        /// Remove everything (downloads + old engines)
        #[arg(long)]
        all: bool,
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
    /// Manage EVE profiles (multiple accounts)
    Profile {
        #[command(subcommand)]
        cmd: ProfileCmd,
    },
    /// Manage configuration files
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
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

#[derive(Subcommand)]
enum ProfileCmd {
    /// List all profiles
    List,
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
    },
    /// Delete a profile (removes prefix and local data)
    Delete {
        /// Profile name
        name: String,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Show profile details
    Info {
        /// Profile name
        name: String,
    },
    /// Clone an existing profile to a new one
    Clone {
        /// Source profile name
        source: String,
        /// New profile name
        target: String,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Initialize config files with defaults
    Init {
        /// Overwrite existing configs
        #[arg(long)]
        force: bool,
    },
    /// Show current config file paths
    Show,
    /// Edit config in default editor
    Edit,
    /// Apply a settings preset (performance, quality, balanced)
    Preset {
        /// Preset name: performance, quality, balanced
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Run { profile, singularity, dx12, notify, hud, hud_config, args: extra_args } => {
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
                .unwrap_or_else(|| PathBuf::from("drive_c/users/steamuser/AppData/Local/eve-online/eve-online.exe"));

            // Build launch arguments
            let mut launch_args: Vec<String> = Vec::new();

            // Server selection (--singularity overrides config)
            if singularity {
                launch_args.push("/server:singularity".to_string());
            } else {
                // Use config default or tranquility
                let default_server = manifest.as_ref()
                    .and_then(|m| m.launch.entrypoints.first())
                    .and_then(|e| e.args.as_ref())
                    .and_then(|args| args.iter().find(|a| a.starts_with("/server:")))
                    .cloned()
                    .unwrap_or_else(|| "/server:tranquility".to_string());
                launch_args.push(default_server);
            }

            // DX12 mode
            if dx12 {
                launch_args.push("/triPlatform:dx12".to_string());
            }

            // Add any extra user-provided arguments
            launch_args.extend(extra_args);

            let mut env_vars: HashMap<String, String> = manifest.as_ref()
                .and_then(|m| m.env.as_ref())
                .and_then(|e| e.base.clone())
                .unwrap_or_else(|| [
                    ("DXVK_ASYNC", "1"),
                    ("PROTON_NO_ESYNC", "1"),
                    ("PROTON_NO_FSYNC", "1"),
                ].into_iter().map(|(k,v)| (k.to_string(), v.to_string())).collect());

            // Enable VKD3D for DX12
            if dx12 {
                env_vars.insert("VKD3D_FEATURE_LEVEL".to_string(), "12_1".to_string());
            }

            // Enable MangoHud overlay
            if hud {
                env_vars.insert("MANGOHUD".to_string(), "1".to_string());
                if !hud_config.is_empty() {
                    env_vars.insert("MANGOHUD_CONFIG".to_string(), hud_config.clone());
                } else {
                    // Default config: FPS, frametime, GPU/CPU stats
                    env_vars.insert("MANGOHUD_CONFIG".to_string(),
                        "fps,frametime,gpu_stats,gpu_temp,cpu_stats,cpu_temp,ram,vram".to_string());
                }
            }

            let engine_dist = engines_dir.join(&engine_id).join("dist");
            let prefix_dir = prefixes_dir.join(format!("eve-{}", profile));

            // Find the actual proton subdirectory (e.g., GE-Proton10-27)
            let proton_root = if engine_dist.exists() {
                std::fs::read_dir(&engine_dist)?
                    .filter_map(|e| e.ok())
                    .find(|e| e.path().join("proton").exists())
                    .map(|e| e.path())
                    .ok_or_else(|| anyhow::anyhow!("No proton found in {}", engine_dist.display()))?
            } else {
                return Err(anyhow::anyhow!("Engine not installed. Run: elm update --install"));
            };

            // 1. Ensure engine is installed
            if !proton_root.join("proton").exists() {
                println!("Engine not found. Run: elm update --install");
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

            // Show launch info
            let server = if singularity { "Singularity (test)" } else { "Tranquility" };
            let dx_mode = if dx12 { "DirectX 12" } else { "DirectX 11" };
            let hud_status = if hud { ", HUD: On" } else { "" };
            println!("✓ Server: {}, Mode: {}{}", server, dx_mode, hud_status);

            if !launch_args.is_empty() {
                println!("✓ Args: {}", launch_args.join(" "));
            }

            println!("Launching EVE Online...");
            let start_time = std::time::Instant::now();

            let spec = elm_core::runtime::launch::LaunchSpec {
                proton_root,
                prefix_dir,
                exe_path_in_prefix: exe_rel,
                args: launch_args,
                env: env_vars,
            };
            let result = elm_core::runtime::launch::launch(spec).await;

            // Send notification when EVE closes
            if notify {
                let duration = start_time.elapsed();
                let hours = duration.as_secs() / 3600;
                let minutes = (duration.as_secs() % 3600) / 60;

                let time_str = if hours > 0 {
                    format!("{}h {}m", hours, minutes)
                } else if minutes > 0 {
                    format!("{}m", minutes)
                } else {
                    "< 1m".to_string()
                };

                let (title, body, icon) = match &result {
                    Ok(_) => (
                        "EVE Online Closed",
                        format!("Session ended after {}", time_str),
                        "eve-online"
                    ),
                    Err(e) => (
                        "EVE Online Error",
                        format!("Crashed after {}: {}", time_str, e),
                        "dialog-error"
                    ),
                };

                let _ = std::process::Command::new("notify-send")
                    .args([
                        "--app-name=ELM",
                        &format!("--icon={}", icon),
                        title,
                        &body,
                    ])
                    .spawn();
            }

            result?;
        }
        Commands::Status => {
            let home = std::env::var("HOME").unwrap_or_default();
            let data_dir = PathBuf::from(format!("{home}/.local/share/elm"));
            let config_dir = PathBuf::from(format!("{home}/.config/elm"));

            println!("ELM Status");
            println!("==========\n");

            // Engines
            println!("Engines:");
            let engines_dir = data_dir.join("engines");
            if engines_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&engines_dir) {
                    let mut found = false;
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let marker = entry.path().join("installed.json");
                            let status = if marker.exists() { "✓" } else { "○" };
                            println!("  {} {}", status, entry.file_name().to_string_lossy());
                            found = true;
                        }
                    }
                    if !found {
                        println!("  (none)");
                    }
                }
            } else {
                println!("  (none)");
            }

            // Prefixes
            println!("\nPrefixes:");
            let prefixes_dir = data_dir.join("prefixes");
            if prefixes_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&prefixes_dir) {
                    let mut found = false;
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let drive_c = entry.path().join("pfx/drive_c");
                            let status = if drive_c.exists() { "✓" } else { "○" };
                            // Get size
                            let size = dir_size(&entry.path()).unwrap_or(0);
                            println!("  {} {} ({:.1} GB)", status, entry.file_name().to_string_lossy(), size as f64 / 1_073_741_824.0);
                            found = true;
                        }
                    }
                    if !found {
                        println!("  (none)");
                    }
                }
            } else {
                println!("  (none)");
            }

            // Snapshots
            println!("\nSnapshots:");
            let snapshots_dir = data_dir.join("snapshots");
            if snapshots_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                    let mut found = false;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "zst").unwrap_or(false) {
                            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            println!("  {} ({:.1} GB)", path.file_name().unwrap().to_string_lossy(), size as f64 / 1_073_741_824.0);
                            found = true;
                        }
                    }
                    if !found {
                        println!("  (none)");
                    }
                }
            } else {
                println!("  (none)");
            }

            // Config
            println!("\nConfig:");
            let manifest_path = config_dir.join("manifests/eve-online.json");
            if manifest_path.exists() {
                println!("  ✓ {}", manifest_path.display());
            } else {
                println!("  (no custom config)");
            }

            println!("\nPaths:");
            println!("  Data:   {}", data_dir.display());
            println!("  Config: {}", config_dir.display());
        }
        Commands::Doctor => {
            println!("ELM Doctor");
            println!("==========\n");

            let mut issues = 0;

            // Check Vulkan
            print!("Vulkan: ");
            let vulkan_ok = std::process::Command::new("vulkaninfo")
                .arg("--summary")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if vulkan_ok {
                println!("✓ available");
            } else {
                println!("✗ not found (install vulkan-tools)");
                issues += 1;
            }

            // Check GPU
            print!("GPU:    ");
            let gpu_info = std::process::Command::new("sh")
                .args(["-c", "lspci | grep -i vga | head -1"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            if !gpu_info.is_empty() {
                let gpu = gpu_info.split(':').last().unwrap_or(&gpu_info).trim();
                println!("✓ {}", gpu);
            } else {
                println!("? unknown");
            }

            // Check driver
            print!("Driver: ");
            let driver = if std::path::Path::new("/proc/driver/nvidia/version").exists() {
                let ver = std::fs::read_to_string("/proc/driver/nvidia/version")
                    .unwrap_or_default()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                format!("NVIDIA {}", ver.split_whitespace().nth(7).unwrap_or(""))
            } else {
                std::process::Command::new("sh")
                    .args(["-c", "glxinfo 2>/dev/null | grep 'OpenGL version' | head -1"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            };
            if driver.contains("unknown") {
                println!("? {}", driver);
            } else {
                println!("✓ {}", driver.chars().take(50).collect::<String>());
            }

            // Check Steam
            print!("Steam:  ");
            let home = std::env::var("HOME").unwrap_or_default();
            let steam_path = format!("{home}/.steam/steam");
            if std::path::Path::new(&steam_path).exists() {
                println!("✓ {}", steam_path);
            } else {
                println!("✗ not found at {}", steam_path);
                issues += 1;
            }

            // Check Python3
            print!("Python: ");
            let python_ver = std::process::Command::new("python3")
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            if !python_ver.is_empty() {
                println!("✓ {}", python_ver.trim());
            } else {
                println!("✗ python3 not found");
                issues += 1;
            }

            // Check MangoHud (optional)
            print!("MangoHud: ");
            let mangohud_ok = std::process::Command::new("mangohud")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if mangohud_ok {
                println!("✓ available (use --hud to enable)");
            } else {
                println!("○ not installed (optional, for FPS overlay)");
            }

            // Check libraries
            println!("\nLibraries:");
            let libs = [
                ("libvulkan", "vulkan-icd-loader"),
                ("libGL", "mesa"),
                ("libX11", "libx11"),
            ];
            for (lib, pkg) in libs {
                print!("  {}: ", lib);
                let found = std::process::Command::new("ldconfig")
                    .args(["-p"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.contains(lib))
                    .unwrap_or(false);
                if found {
                    println!("✓");
                } else {
                    println!("✗ (install {})", pkg);
                    issues += 1;
                }
            }

            // Check disk space
            println!("\nDisk:");
            let data_dir = PathBuf::from(format!("{home}/.local/share/elm"));
            if let Ok(output) = std::process::Command::new("df")
                .args(["-h", data_dir.to_str().unwrap_or("/home")])
                .output()
            {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    if let Some(line) = s.lines().nth(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            println!("  Available: {} (on {})", parts[3], parts[0]);
                        }
                    }
                }
            }

            // Summary
            println!("\n----------");
            if issues == 0 {
                println!("✓ System ready for EVE Online");
            } else {
                println!("✗ {} issue(s) found", issues);
            }
        }
        Commands::Logs { log_type, lines, list, profile } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let prefix_dir = PathBuf::from(format!("{home}/.local/share/elm/prefixes/eve-{profile}"));
            let logs_dir = prefix_dir.join("pfx/drive_c/users/steamuser/AppData/Roaming/EVE Online/logs");

            // Collect all log files
            let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

            // EVE Launcher logs
            if logs_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&logs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "log").unwrap_or(false) {
                            if let Ok(meta) = path.metadata() {
                                if let Ok(modified) = meta.modified() {
                                    log_files.push((path, modified));
                                }
                            }
                        }
                    }
                }
            }

            // Squirrel/installer logs
            let squirrel_log = prefix_dir.join("pfx/drive_c/users/steamuser/AppData/Local/eve-online/Squirrel-Update.log");
            if squirrel_log.exists() {
                if let Ok(meta) = squirrel_log.metadata() {
                    if let Ok(modified) = meta.modified() {
                        log_files.push((squirrel_log, modified));
                    }
                }
            }

            // Proton fixes log
            let proton_log = PathBuf::from("/tmp/protonfixes_test.log");
            if proton_log.exists() {
                if let Ok(meta) = proton_log.metadata() {
                    if let Ok(modified) = meta.modified() {
                        log_files.push((proton_log, modified));
                    }
                }
            }

            // Sort by modification time (newest first)
            log_files.sort_by(|a, b| b.1.cmp(&a.1));

            if list {
                println!("Available log files:\n");
                for (path, time) in &log_files {
                    let age = time.elapsed().map(|d| {
                        if d.as_secs() < 60 {
                            format!("{}s ago", d.as_secs())
                        } else if d.as_secs() < 3600 {
                            format!("{}m ago", d.as_secs() / 60)
                        } else if d.as_secs() < 86400 {
                            format!("{}h ago", d.as_secs() / 3600)
                        } else {
                            format!("{}d ago", d.as_secs() / 86400)
                        }
                    }).unwrap_or_else(|_| "?".to_string());

                    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    println!("  {} ({}, {})", path.display(), age, format_size(size));
                }
                return Ok(());
            }

            // Filter by log type
            let filtered: Vec<_> = match log_type.as_str() {
                "launcher" => log_files.into_iter().filter(|(p, _)| {
                    p.file_name().map(|n| n.to_string_lossy().contains("eve-online-launcher")).unwrap_or(false)
                }).collect(),
                "wine" | "proton" => log_files.into_iter().filter(|(p, _)| {
                    p.to_string_lossy().contains("proton") || p.to_string_lossy().contains("wine")
                }).collect(),
                "squirrel" | "installer" => log_files.into_iter().filter(|(p, _)| {
                    p.to_string_lossy().contains("Squirrel")
                }).collect(),
                "all" => log_files,
                _ => {
                    println!("Unknown log type: {}. Use: launcher, wine, squirrel, all", log_type);
                    return Ok(());
                }
            };

            if filtered.is_empty() {
                println!("No {} logs found for profile '{}'", log_type, profile);
                println!("\nRun 'elm logs --list' to see all available logs");
                return Ok(());
            }

            // Show most recent log
            let (log_path, _) = &filtered[0];
            println!("=== {} ===\n", log_path.display());

            if let Ok(content) = std::fs::read_to_string(log_path) {
                let all_lines: Vec<&str> = content.lines().collect();
                let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
                for line in &all_lines[start..] {
                    println!("{}", line);
                }
                println!("\n=== Showing last {} of {} lines ===",
                    std::cmp::min(lines, all_lines.len()), all_lines.len());
            } else {
                println!("(could not read log file)");
            }
        }
        Commands::Update { install, no_backup, notify } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let data_dir = PathBuf::from(format!("{home}/.local/share/elm"));
            let engines_dir = data_dir.join("engines");
            let downloads_dir = data_dir.join("downloads");
            let prefixes_dir = data_dir.join("prefixes");
            let snapshots_dir = data_dir.join("snapshots");

            println!("Checking for engine updates...\n");

            // Get installed version
            let installed: Option<String> = std::fs::read_dir(&engines_dir)
                .ok()
                .and_then(|entries| {
                    entries
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .filter(|e| e.path().join("installed.json").exists())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .next()
                });

            println!("Installed: {}", installed.as_deref().unwrap_or("(none)"));

            // Fetch latest from GitHub API
            print!("Latest:    ");
            let latest = std::process::Command::new("curl")
                .args(["-s", "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases/latest"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| {
                    // Parse tag_name from JSON
                    s.lines()
                        .find(|l| l.contains("\"tag_name\""))
                        .and_then(|l| l.split('"').nth(3))
                        .map(|s| s.to_string())
                });

            let latest_tag = match &latest {
                Some(tag) => {
                    println!("{}", tag);
                    tag.clone()
                }
                None => {
                    println!("(failed to fetch)");
                    return Err(anyhow::anyhow!("Could not fetch latest release from GitHub"));
                }
            };

            // Compare versions
            let installed_normalized = installed.as_ref()
                .map(|s| s.to_lowercase().replace("-", "").replace("_", ""));
            let latest_normalized = latest_tag.to_lowercase().replace("-", "").replace("_", "");

            let needs_update = installed_normalized.as_ref()
                .map(|i| i != &latest_normalized)
                .unwrap_or(true);

            if !needs_update {
                println!("\n✓ Engine is up to date");
                return Ok(());
            }

            println!("\n⬆ Update available!");

            // Send notification if requested
            if notify {
                let _ = std::process::Command::new("notify-send")
                    .args([
                        "--app-name=ELM",
                        "--icon=software-update-available",
                        "EVE Engine Update Available",
                        &format!("{} → {}", installed.as_deref().unwrap_or("none"), latest_tag),
                    ])
                    .spawn();
            }

            if !install {
                println!("\nRun 'elm update --install' to download and install");
                return Ok(());
            }

            // Auto-backup existing prefixes before update
            if !no_backup && prefixes_dir.exists() {
                let prefixes: Vec<_> = std::fs::read_dir(&prefixes_dir)?
                    .flatten()
                    .filter(|e| e.path().is_dir() && e.path().join("pfx/drive_c").exists())
                    .collect();

                if !prefixes.is_empty() {
                    println!("\nBacking up {} prefix(es) before update...", prefixes.len());
                    std::fs::create_dir_all(&snapshots_dir)?;

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    for entry in &prefixes {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let snapshot_name = format!("{}-pre-update-{}", name, timestamp);
                        let prefix_path = entry.path();

                        print!("  {} ... ", name);
                        std::io::Write::flush(&mut std::io::stdout())?;

                        match elm_core::rollback::snapshot::snapshot_prefix(
                            &prefix_path.join("pfx"),
                            &snapshots_dir,
                            &snapshot_name
                        ) {
                            Ok(out) => {
                                let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
                                println!("✓ ({:.1} GB)", size as f64 / 1_073_741_824.0);
                            }
                            Err(e) => {
                                println!("✗ ({})", e);
                                println!("\nWarning: Backup failed, but continuing with update.");
                                println!("You may want to manually backup your prefix before proceeding.");
                            }
                        }
                    }
                    println!();
                }
            }

            // Download and install
            println!("Downloading {}...", latest_tag);

            let download_url = format!(
                "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/{}/{}.tar.gz",
                latest_tag, latest_tag
            );

            let archive_path = downloads_dir.join(format!("{}.tar.gz", latest_tag));
            std::fs::create_dir_all(&downloads_dir)?;

            // Download with curl (shows progress)
            let status = std::process::Command::new("curl")
                .args(["-L", "-o", archive_path.to_str().unwrap(), &download_url, "--progress-bar"])
                .status()?;

            if !status.success() {
                return Err(anyhow::anyhow!("Download failed"));
            }

            // Extract
            println!("Extracting...");
            let engine_id = latest_tag.to_lowercase();
            let engine_dir = engines_dir.join(&engine_id);
            let dist_dir = engine_dir.join("dist");
            std::fs::create_dir_all(&dist_dir)?;

            let status = std::process::Command::new("tar")
                .args(["-xzf", archive_path.to_str().unwrap(), "-C", dist_dir.to_str().unwrap()])
                .status()?;

            if !status.success() {
                return Err(anyhow::anyhow!("Extraction failed"));
            }

            // Write marker
            let marker = serde_json::json!({
                "engine_id": engine_id,
                "version": latest_tag
            });
            std::fs::write(engine_dir.join("installed.json"), serde_json::to_vec_pretty(&marker)?)?;

            println!("\n✓ Installed {} to {}", latest_tag, engine_dir.display());
            println!("\nNote: Update ~/.config/elm/manifests/eve-online.json to use the new engine");
        }
        Commands::Clean { dry_run, downloads, engines, all } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let data_dir = PathBuf::from(format!("{home}/.local/share/elm"));
            let downloads_dir = data_dir.join("downloads");
            let engines_dir = data_dir.join("engines");

            let clean_downloads = downloads || all;
            let clean_engines = engines || all;

            if !clean_downloads && !clean_engines {
                println!("ELM Clean");
                println!("=========\n");
                println!("Specify what to clean:");
                println!("  --downloads  Remove downloaded archives");
                println!("  --engines    Remove old engine versions (keep latest)");
                println!("  --all        Remove both");
                println!("  --dry-run    Show what would be removed");
                return Ok(());
            }

            let mut total_freed: u64 = 0;

            // Clean downloads
            if clean_downloads && downloads_dir.exists() {
                println!("Downloads:");
                let mut download_size: u64 = 0;
                let mut files_to_remove = Vec::new();

                for entry in std::fs::read_dir(&downloads_dir)?.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        download_size += size;
                        files_to_remove.push(path);
                    }
                }

                if files_to_remove.is_empty() {
                    println!("  (no files to clean)");
                } else {
                    for path in &files_to_remove {
                        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                        println!("  {} {}", if dry_run { "○" } else { "✗" }, path.file_name().unwrap().to_string_lossy());
                        if !dry_run {
                            std::fs::remove_file(path)?;
                        }
                        total_freed += size;
                    }
                    println!("  {} ({:.1} MB)",
                        if dry_run { "Would free" } else { "Freed" },
                        download_size as f64 / 1_048_576.0);
                }
                println!();
            }

            // Clean old engines (keep latest)
            if clean_engines && engines_dir.exists() {
                println!("Engines:");
                let mut engine_entries: Vec<_> = std::fs::read_dir(&engines_dir)?
                    .flatten()
                    .filter(|e| e.path().is_dir() && e.path().join("installed.json").exists())
                    .collect();

                if engine_entries.len() <= 1 {
                    println!("  (only one engine installed, nothing to clean)");
                } else {
                    // Sort by name (version) descending to keep latest
                    engine_entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                    // Keep the first (latest), remove the rest
                    let latest = &engine_entries[0];
                    println!("  ✓ Keeping: {}", latest.file_name().to_string_lossy());

                    for entry in &engine_entries[1..] {
                        let path = entry.path();
                        let size = dir_size(&path).unwrap_or(0);
                        println!("  {} {} ({:.1} GB)",
                            if dry_run { "○" } else { "✗" },
                            entry.file_name().to_string_lossy(),
                            size as f64 / 1_073_741_824.0);

                        if !dry_run {
                            std::fs::remove_dir_all(&path)?;
                        }
                        total_freed += size;
                    }
                }
                println!();
            }

            // Summary
            println!("----------");
            if dry_run {
                println!("Dry run: would free {:.2} GB", total_freed as f64 / 1_073_741_824.0);
                println!("\nRun without --dry-run to actually clean");
            } else {
                println!("Freed {:.2} GB", total_freed as f64 / 1_073_741_824.0);
            }
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
        Commands::Profile { cmd } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let prefixes_dir = PathBuf::from(format!("{home}/.local/share/elm/prefixes"));
            let snapshots_dir = PathBuf::from(format!("{home}/.local/share/elm/snapshots"));

            match cmd {
                ProfileCmd::List => {
                    println!("EVE Profiles");
                    println!("============\n");

                    if !prefixes_dir.exists() {
                        println!("No profiles found. Create one with: elm profile create <name>");
                        return Ok(());
                    }

                    let mut profiles: Vec<_> = std::fs::read_dir(&prefixes_dir)?
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .filter(|e| e.file_name().to_string_lossy().starts_with("eve-"))
                        .collect();

                    if profiles.is_empty() {
                        println!("No profiles found. Create one with: elm profile create <name>");
                        return Ok(());
                    }

                    profiles.sort_by_key(|e| e.file_name());

                    for entry in profiles {
                        let name = entry.file_name().to_string_lossy().replace("eve-", "");
                        let path = entry.path();
                        let has_eve = path.join("pfx/drive_c/CCP/EVE").exists();
                        let size = dir_size(&path).unwrap_or(0);

                        let status = if has_eve { "✓" } else { "○" };
                        println!("  {} {} ({:.1} GB)", status, name, size as f64 / 1_073_741_824.0);
                    }

                    println!("\n✓ = EVE installed, ○ = prefix only");
                    println!("\nUsage: elm run --profile <name>");
                }
                ProfileCmd::Create { name } => {
                    let prefix_dir = prefixes_dir.join(format!("eve-{}", name));

                    if prefix_dir.exists() {
                        println!("Profile '{}' already exists at {}", name, prefix_dir.display());
                        return Ok(());
                    }

                    // Find engine
                    let engines_dir = PathBuf::from(format!("{home}/.local/share/elm/engines"));
                    let engine_dir = std::fs::read_dir(&engines_dir)?
                        .flatten()
                        .find(|e| e.path().join("installed.json").exists())
                        .map(|e| e.path());

                    let proton_root = match engine_dir {
                        Some(dir) => {
                            let dist = dir.join("dist");
                            std::fs::read_dir(&dist)?
                                .flatten()
                                .find(|e| e.path().join("proton").exists())
                                .map(|e| e.path())
                                .ok_or_else(|| anyhow::anyhow!("No proton found in engine"))?
                        }
                        None => {
                            println!("No engine installed. Run: elm update --install");
                            return Ok(());
                        }
                    };

                    println!("Creating profile '{}'...", name);
                    elm_core::prefix::ensure_prefix_initialized(&prefix_dir, &proton_root).await?;
                    println!("✓ Profile '{}' created at {}", name, prefix_dir.display());
                    println!("\nTo install EVE: elm run --profile {}", name);
                }
                ProfileCmd::Delete { name, yes } => {
                    let prefix_dir = prefixes_dir.join(format!("eve-{}", name));

                    if !prefix_dir.exists() {
                        println!("Profile '{}' not found", name);
                        return Ok(());
                    }

                    let size = dir_size(&prefix_dir).unwrap_or(0);
                    let size_gb = size as f64 / 1_073_741_824.0;

                    if !yes {
                        println!("Delete profile '{}'? ({:.1} GB)", name, size_gb);
                        println!("This will permanently remove: {}", prefix_dir.display());
                        print!("\nType 'yes' to confirm: ");
                        use std::io::Write;
                        std::io::stdout().flush()?;

                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        if input.trim() != "yes" {
                            println!("Cancelled");
                            return Ok(());
                        }
                    }

                    println!("Deleting profile '{}'...", name);
                    std::fs::remove_dir_all(&prefix_dir)?;

                    // Also remove any snapshots for this profile
                    if snapshots_dir.exists() {
                        for entry in std::fs::read_dir(&snapshots_dir)?.flatten() {
                            let fname = entry.file_name().to_string_lossy().to_string();
                            if fname.starts_with(&format!("{}-", name)) || fname.starts_with(&format!("eve-{}-", name)) {
                                std::fs::remove_file(entry.path())?;
                                println!("  Removed snapshot: {}", fname);
                            }
                        }
                    }

                    println!("✓ Profile '{}' deleted ({:.1} GB freed)", name, size_gb);
                }
                ProfileCmd::Info { name } => {
                    let prefix_dir = prefixes_dir.join(format!("eve-{}", name));

                    if !prefix_dir.exists() {
                        println!("Profile '{}' not found", name);
                        return Ok(());
                    }

                    println!("Profile: {}", name);
                    println!("=========={}", "=".repeat(name.len()));
                    println!();

                    // Size
                    let size = dir_size(&prefix_dir).unwrap_or(0);
                    println!("Size:     {:.2} GB", size as f64 / 1_073_741_824.0);
                    println!("Path:     {}", prefix_dir.display());

                    // EVE status
                    let eve_path = prefix_dir.join("pfx/drive_c/CCP/EVE");
                    if eve_path.exists() {
                        println!("EVE:      ✓ installed");

                        // Check for game client
                        let client_path = eve_path.join("tq/bin64/exefile.exe");
                        if client_path.exists() {
                            println!("Client:   ✓ downloaded");
                        } else {
                            println!("Client:   ○ not downloaded (run game once)");
                        }
                    } else {
                        println!("EVE:      ○ not installed");
                    }

                    // Snapshots
                    println!();
                    println!("Snapshots:");
                    let mut found_snapshots = false;
                    if snapshots_dir.exists() {
                        for entry in std::fs::read_dir(&snapshots_dir)?.flatten() {
                            let fname = entry.file_name().to_string_lossy().to_string();
                            if fname.contains(&name) && fname.ends_with(".tar.zst") {
                                let size = std::fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
                                println!("  {} ({:.1} GB)", fname, size as f64 / 1_073_741_824.0);
                                found_snapshots = true;
                            }
                        }
                    }
                    if !found_snapshots {
                        println!("  (none)");
                    }

                    println!();
                    println!("Commands:");
                    println!("  elm run --profile {}           # Launch EVE", name);
                    println!("  elm snapshot --prefix {} \\", prefix_dir.display());
                    println!("    --snapshots {} --name {}-backup", snapshots_dir.display(), name);
                }
                ProfileCmd::Clone { source, target } => {
                    let source_dir = prefixes_dir.join(format!("eve-{}", source));
                    let target_dir = prefixes_dir.join(format!("eve-{}", target));

                    if !source_dir.exists() {
                        println!("Source profile '{}' not found", source);
                        return Ok(());
                    }

                    if target_dir.exists() {
                        println!("Target profile '{}' already exists", target);
                        return Ok(());
                    }

                    let source_size = dir_size(&source_dir).unwrap_or(0);
                    println!("Cloning profile '{}' to '{}'...", source, target);
                    println!("Size: {:.2} GB", source_size as f64 / 1_073_741_824.0);
                    println!();

                    // Use cp -a for preserving symlinks and attributes
                    let status = std::process::Command::new("cp")
                        .args(["-a", source_dir.to_str().unwrap(), target_dir.to_str().unwrap()])
                        .status()?;

                    if !status.success() {
                        return Err(anyhow::anyhow!("Failed to clone profile"));
                    }

                    println!("✓ Profile '{}' cloned to '{}'", source, target);
                    println!("\nLaunch with: elm run --profile {}", target);
                }
            }
        }
        Commands::Config { cmd } => {
            let home = std::env::var("HOME").unwrap_or_default();
            let config_dir = PathBuf::from(format!("{home}/.config/elm"));
            let manifests_dir = config_dir.join("manifests");

            match cmd {
                ConfigCmd::Init { force } => {
                    println!("Initializing ELM configuration...\n");

                    // Create directories
                    std::fs::create_dir_all(&manifests_dir)?;

                    // Default manifest
                    let manifest_path = manifests_dir.join("eve-online.json");
                    if manifest_path.exists() && !force {
                        println!("  ○ {} (exists, use --force to overwrite)", manifest_path.display());
                    } else {
                        let default_manifest = r#"{
  "schema": "elm.manifest.v1",
  "id": "eve-online",
  "display_name": "EVE Online",
  "installer": {
    "type": "launcher",
    "source": {
      "url": "https://binaries.eveonline.com/EveLauncher-2180591.exe"
    },
    "install_dir": "CCP/EVE"
  },
  "engine": {
    "ref": "ge-proton10-27"
  },
  "runtime": {
    "wineprefix_layout": "per-profile",
    "dx": {
      "preferred": "dx11",
      "allow_dx12": true
    },
    "components": {
      "dxvk": { "enabled": true },
      "vkd3d": { "enabled": true }
    }
  },
  "env": {
    "base": {
      "DXVK_ASYNC": "1",
      "PROTON_NO_ESYNC": "0",
      "PROTON_NO_FSYNC": "0",
      "PROTON_ENABLE_NVAPI": "1",
      "VKD3D_FEATURE_LEVEL": "12_1",
      "WINE_FULLSCREEN_FSR": "1"
    }
  },
  "launch": {
    "entrypoints": [
      {
        "name": "EVE Launcher",
        "type": "exe",
        "path": "drive_c/users/steamuser/AppData/Local/eve-online/eve-online.exe"
      }
    ]
  }
}
"#;
                        std::fs::write(&manifest_path, default_manifest)?;
                        println!("  ✓ Created {}", manifest_path.display());
                    }

                    println!("\nConfiguration initialized!");
                    println!("\nEdit your config:");
                    println!("  elm config edit");
                    println!("\nOr manually edit:");
                    println!("  {}", manifest_path.display());
                }
                ConfigCmd::Show => {
                    println!("ELM Configuration");
                    println!("=================\n");

                    println!("Config directory: {}", config_dir.display());
                    println!();

                    println!("Files:");
                    let manifest_path = manifests_dir.join("eve-online.json");
                    if manifest_path.exists() {
                        let size = std::fs::metadata(&manifest_path).map(|m| m.len()).unwrap_or(0);
                        println!("  ✓ {} ({} bytes)", manifest_path.display(), size);
                    } else {
                        println!("  ○ {} (not found)", manifest_path.display());
                        println!("\n  Run 'elm config init' to create default configs");
                    }

                    // Show current settings if config exists
                    if manifest_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                println!();
                                println!("Current settings:");
                                if let Some(engine) = json.get("engine").and_then(|e| e.get("ref")) {
                                    println!("  Engine: {}", engine.as_str().unwrap_or("?"));
                                }
                                if let Some(env) = json.get("env").and_then(|e| e.get("base")) {
                                    if let Some(obj) = env.as_object() {
                                        println!("  Environment variables: {}", obj.len());
                                        for (k, v) in obj.iter().take(5) {
                                            println!("    {}={}", k, v.as_str().unwrap_or("?"));
                                        }
                                        if obj.len() > 5 {
                                            println!("    ... and {} more", obj.len() - 5);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ConfigCmd::Edit => {
                    let manifest_path = manifests_dir.join("eve-online.json");

                    if !manifest_path.exists() {
                        println!("Config file not found. Creating with defaults...");
                        // Trigger init
                        std::fs::create_dir_all(&manifests_dir)?;
                        let default_manifest = r#"{
  "schema": "elm.manifest.v1",
  "id": "eve-online",
  "display_name": "EVE Online",
  "engine": { "ref": "ge-proton10-27" },
  "env": {
    "base": {
      "DXVK_ASYNC": "1",
      "PROTON_NO_ESYNC": "0",
      "PROTON_NO_FSYNC": "0"
    }
  }
}
"#;
                        std::fs::write(&manifest_path, default_manifest)?;
                    }

                    // Open in editor
                    let editor = std::env::var("EDITOR")
                        .or_else(|_| std::env::var("VISUAL"))
                        .unwrap_or_else(|_| "nano".to_string());

                    println!("Opening {} in {}...", manifest_path.display(), editor);

                    let status = std::process::Command::new(&editor)
                        .arg(&manifest_path)
                        .status()?;

                    if status.success() {
                        println!("Config saved.");
                    } else {
                        println!("Editor exited with error");
                    }
                }
                ConfigCmd::Preset { name } => {
                    let manifest_path = manifests_dir.join("eve-online.json");

                    // Define presets
                    let presets: std::collections::HashMap<&str, serde_json::Value> = [
                        ("performance", serde_json::json!({
                            "DXVK_ASYNC": "1",
                            "PROTON_NO_ESYNC": "0",
                            "PROTON_NO_FSYNC": "0",
                            "DXVK_HUD": "",
                            "WINE_FULLSCREEN_FSR": "1",
                            "WINE_FULLSCREEN_FSR_STRENGTH": "2",
                            "PROTON_ENABLE_NVAPI": "1",
                            "VKD3D_FEATURE_LEVEL": "12_1",
                            "__GL_SHADER_DISK_CACHE": "1",
                            "__GL_SHADER_DISK_CACHE_SKIP_CLEANUP": "1"
                        })),
                        ("quality", serde_json::json!({
                            "DXVK_ASYNC": "0",
                            "PROTON_NO_ESYNC": "0",
                            "PROTON_NO_FSYNC": "0",
                            "DXVK_HUD": "",
                            "WINE_FULLSCREEN_FSR": "0",
                            "PROTON_ENABLE_NVAPI": "1",
                            "VKD3D_FEATURE_LEVEL": "12_1",
                            "__GL_SHADER_DISK_CACHE": "1"
                        })),
                        ("balanced", serde_json::json!({
                            "DXVK_ASYNC": "1",
                            "PROTON_NO_ESYNC": "0",
                            "PROTON_NO_FSYNC": "0",
                            "DXVK_HUD": "fps",
                            "WINE_FULLSCREEN_FSR": "1",
                            "PROTON_ENABLE_NVAPI": "1",
                            "VKD3D_FEATURE_LEVEL": "12_1"
                        })),
                        ("debug", serde_json::json!({
                            "DXVK_ASYNC": "1",
                            "PROTON_NO_ESYNC": "0",
                            "PROTON_NO_FSYNC": "0",
                            "DXVK_HUD": "fps,frametimes,gpuload,devinfo",
                            "DXVK_LOG_LEVEL": "info",
                            "PROTON_LOG": "1",
                            "WINEDEBUG": "warn+all"
                        }))
                    ].into_iter().collect();

                    let preset_name = name.to_lowercase();
                    let preset = match presets.get(preset_name.as_str()) {
                        Some(p) => p,
                        None => {
                            println!("Unknown preset: {}", name);
                            println!("\nAvailable presets:");
                            println!("  performance  - Maximum FPS, FSR upscaling, minimal HUD");
                            println!("  quality      - Native resolution, no async shaders");
                            println!("  balanced     - Good performance with FPS counter");
                            println!("  debug        - Verbose logging for troubleshooting");
                            return Ok(());
                        }
                    };

                    // Load existing config or create new
                    let mut config: serde_json::Value = if manifest_path.exists() {
                        let content = std::fs::read_to_string(&manifest_path)?;
                        serde_json::from_str(&content)?
                    } else {
                        std::fs::create_dir_all(&manifests_dir)?;
                        serde_json::json!({
                            "schema": "elm.manifest.v1",
                            "id": "eve-online",
                            "display_name": "EVE Online",
                            "engine": { "ref": "ge-proton10-27" },
                            "env": { "base": {} }
                        })
                    };

                    // Update env.base with preset values
                    if let Some(env) = config.get_mut("env") {
                        if let Some(base) = env.get_mut("base") {
                            if let Some(base_obj) = base.as_object_mut() {
                                if let Some(preset_obj) = preset.as_object() {
                                    for (k, v) in preset_obj {
                                        base_obj.insert(k.clone(), v.clone());
                                    }
                                }
                            }
                        }
                    }

                    // Write updated config
                    let pretty = serde_json::to_string_pretty(&config)?;
                    std::fs::write(&manifest_path, pretty)?;

                    println!("Applied '{}' preset to {}", preset_name, manifest_path.display());
                    println!("\nSettings:");
                    if let Some(preset_obj) = preset.as_object() {
                        for (k, v) in preset_obj {
                            println!("  {}={}", k, v.as_str().unwrap_or("?"));
                        }
                    }
                }
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

fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                size += dir_size(&entry.path()).unwrap_or(0);
            } else {
                size += meta.len();
            }
        }
    }
    Ok(size)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
