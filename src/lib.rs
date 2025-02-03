#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct WorkspaceConfig {
  packages: Vec<String>,
  #[serde(default)]
  catalog: Vec<String>,
}

#[napi]
pub fn check(workspace_path: String) -> Result<bool> {
  let path = PathBuf::from(workspace_path);
  let workspace_yaml_path = path.join("pnpm-workspace.yaml");

  let workspace_yaml = std::fs::read_to_string(&workspace_yaml_path).map_err(|e| {
    Error::from_reason(format!(
      "Failed to read {}: {}",
      workspace_yaml_path.display(),
      e
    ))
  })?;

  let config: WorkspaceConfig = serde_yaml::from_str(&workspace_yaml)
    .map_err(|e| Error::from_reason(format!("Failed to parse pnpm-workspace.yaml: {}", e)))?;

  // Scan packages for catalog usage
  for package_glob in &config.packages {
    let package_paths = glob::glob(&format!("{}/{}/package.json", path.display(), package_glob))
      .map_err(|e| Error::from_reason(format!("Invalid glob pattern {}: {}", package_glob, e)))?;

    for entry in package_paths {
      match entry {
        Ok(package_path) => {
          if let Err(e) = check_package(&package_path, &config.catalog) {
            eprintln!("Warning: Failed to check {}: {}", package_path.display(), e);
          }
        }
        Err(e) => eprintln!("Warning: Failed to read entry: {}", e),
      }
    }
  }

  Ok(!config.catalog.is_empty())
}

fn check_package(package_path: &PathBuf, catalog_entries: &[String]) -> Result<()> {
  let package_json = fs::read_to_string(package_path)
    .map_err(|e| Error::from_reason(format!("Failed to read {}: {}", package_path.display(), e)))?;

  let package_data: serde_json::Value = serde_json::from_str(&package_json).map_err(|e| {
    Error::from_reason(format!("Failed to parse {}: {}", package_path.display(), e))
  })?;

  if let Some(deps) = package_data.get("dependencies").and_then(|d| d.as_object()) {
    for entry in catalog_entries {
      if let Some(version) = deps.get(entry).and_then(|v| v.as_str()) {
        if !version.starts_with("workspace:") {
          println!("  Found non-workspace dependency {} with version {}", entry, version);
        }
      }
    }
  }

  Ok(())
}

#[napi]
pub fn fix(workspace_path: String) -> Result<()> {
  let path = PathBuf::from(&workspace_path);
  let workspace_yaml_path = path.join("pnpm-workspace.yaml");

  let workspace_yaml = std::fs::read_to_string(&workspace_yaml_path).map_err(|e| {
    Error::from_reason(format!(
      "Failed to read {}: {}",
      workspace_yaml_path.display(),
      e
    ))
  })?;

  let config: WorkspaceConfig = serde_yaml::from_str(&workspace_yaml)
    .map_err(|e| Error::from_reason(format!("Failed to parse pnpm-workspace.yaml: {}", e)))?;

  // Process each package
  for package_glob in &config.packages {
    println!("Processing packages matching: {}", package_glob);
  }

  Ok(())
}
