use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Check for catalog usage in workspace packages
  Check {
    /// Path to the workspace root (default: current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,
  },
  /// Fix catalog usage in workspace packages
  Fix {
    /// Path to the workspace root (default: current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,
  },
}

#[derive(Debug, Deserialize)]
struct WorkspaceConfig {
  packages: Vec<String>,
  #[serde(default)]
  catalog: Vec<String>,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Check { path } => {
      let workspace_path = path.unwrap_or_else(|| PathBuf::from("."));
      check_workspace(&workspace_path)?;
    }
    Commands::Fix { path } => {
      let workspace_path = path.unwrap_or_else(|| PathBuf::from("."));
      fix_workspace(&workspace_path)?;
    }
  }

  Ok(())
}

fn check_workspace(workspace_path: &PathBuf) -> Result<()> {
  let workspace_yaml_path = workspace_path.join("pnpm-workspace.yaml");
  let workspace_yaml = std::fs::read_to_string(&workspace_yaml_path)
    .with_context(|| format!("Failed to read {}", workspace_yaml_path.display()))?;

  let config: WorkspaceConfig =
    serde_yaml::from_str(&workspace_yaml).with_context(|| "Failed to parse pnpm-workspace.yaml")?;

  if config.catalog.is_empty() {
    println!("No catalog entries found in pnpm-workspace.yaml");
    return Ok(());
  }

  println!("Found {} catalog entries", config.catalog.len());
  println!("Catalog entries: {:?}", config.catalog);

  // Scan packages for catalog usage
  for package_glob in &config.packages {
    println!("Scanning packages matching: {}", package_glob);
    let glob_pattern = format!("{}/{}/package.json", workspace_path.display(), package_glob);
    let package_paths =
      glob(&glob_pattern).with_context(|| format!("Invalid glob pattern: {}", glob_pattern))?;

    for entry in package_paths {
      match entry {
        Ok(package_path) => {
          check_package(&package_path, &config.catalog)
            .with_context(|| format!("Failed to check {}", package_path.display()))?;
        }
        Err(e) => println!("Warning: Failed to read entry: {}", e),
      }
    }
  }

  Ok(())
}

fn check_package(package_path: &PathBuf, catalog_entries: &[String]) -> Result<()> {
  let package_json = fs::read_to_string(package_path)
    .with_context(|| format!("Failed to read {}", package_path.display()))?;

  let package_data: serde_json::Value = serde_json::from_str(&package_json)
    .with_context(|| format!("Failed to parse {}", package_path.display()))?;

  println!("Checking package: {}", package_path.display());
  if let Some(deps) = package_data.get("dependencies").and_then(|d| d.as_object()) {
    for entry in catalog_entries {
      if let Some(version) = deps.get(entry).and_then(|v| v.as_str()) {
        if !version.starts_with("workspace:") {
          println!(
            "  Found non-workspace dependency {} with version {}",
            entry, version
          );
        }
      }
    }
  }

  Ok(())
}

fn fix_workspace(workspace_path: &PathBuf) -> Result<()> {
  let workspace_yaml_path = workspace_path.join("pnpm-workspace.yaml");
  let workspace_yaml = std::fs::read_to_string(&workspace_yaml_path)
    .with_context(|| format!("Failed to read {}", workspace_yaml_path.display()))?;

  let config: WorkspaceConfig =
    serde_yaml::from_str(&workspace_yaml).with_context(|| "Failed to parse pnpm-workspace.yaml")?;

  // Process each package
  for package_glob in &config.packages {
    println!("Processing packages matching: {}", package_glob);
    let glob_pattern = format!("{}/{}/package.json", workspace_path.display(), package_glob);
    let package_paths =
      glob(&glob_pattern).with_context(|| format!("Invalid glob pattern: {}", glob_pattern))?;

    for entry in package_paths {
      match entry {
        Ok(package_path) => {
          println!("Would fix package: {}", package_path.display());
        }
        Err(e) => println!("Warning: Failed to read entry: {}", e),
      }
    }
  }

  Ok(())
}
