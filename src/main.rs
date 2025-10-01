use std::collections::{BTreeMap, HashSet};
use std::fs::{self};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use toml;
use calcbits::{create_progress_bar, save_to_db};

const BPM_VERSION: &str = "0.1.2";

#[derive(Debug, Deserialize)]
struct PackageVersion {
    path: String,
    binaries: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>, // dependencies field
}

#[derive(Debug, Deserialize)]
struct Repo {
    #[serde(flatten)]
    packages: BTreeMap<String, BTreeMap<String, PackageVersion>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstalledPackage {
    repo: String,
    version: String,
    binaries: Vec<String>,
}

type InstalledDb = BTreeMap<String, InstalledPackage>;

fn load_installed(db_path: &Path) -> InstalledDb {
    if db_path.exists() {
        let content = fs::read_to_string(db_path).unwrap();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        InstalledDb::new()
    }
}

fn save_installed(db_path: &Path, db: &InstalledDb) {
    let content = serde_json::to_string_pretty(db).unwrap();
    fs::write(db_path, content).unwrap();
}

fn parse_pkg_arg(arg: &str) -> (String, String, Option<String>) {
    let parts: Vec<&str> = arg.split(':').collect();
    let repo = parts[0].to_string();
    let package = parts[1].to_string();
    let version = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };
    (repo, package, version)
}

// --- Recursive installer with dependencies + cycle detection ---
fn install_package(
    bpm_store: &Path,
    repo_name: &str,
    package: &str,
    version: Option<&str>,
) {
    let mut visited = HashSet::new();
    install_recursive(bpm_store, repo_name, package, version, &mut visited);
}

fn install_recursive(
    bpm_store: &Path,
    repo_name: &str,
    package: &str,
    version: Option<&str>,
    visited: &mut HashSet<String>,
) {
    let repo_path = bpm_store.join(repo_name).join("packages.mri");
    let toml_content = fs::read_to_string(&repo_path).expect("Failed to read .mri file");
    let repo: Repo = toml::from_str(&toml_content).expect("Failed to parse TOML");

    if let Some(versions) = repo.packages.get(package) {
        let ver = version.unwrap_or_else(|| versions.keys().max().unwrap());
        if let Some(pkg) = versions.get(ver) {
            // --- Detect cycle ---
            let key = format!("{}:{}", package, ver);
            if visited.contains(&key) {
                println!("⚠ Circular dependency detected at {}", key);
                return;
            }
            visited.insert(key.clone());

            // --- 1. Install dependencies first ---
            for dep in &pkg.dependencies {
                let (dep_pkg, dep_ver) = if dep.contains(':') {
                    let parts: Vec<&str> = dep.split(':').collect();
                    (parts[0], Some(parts[1]))
                } else {
                    (dep.as_str(), None)
                };

                let db = load_installed(&bpm_store.join("installed.json"));
                if !db.contains_key(dep_pkg) {
                    println!("Installing dependency {}...", dep_pkg);
                    install_recursive(bpm_store, repo_name, dep_pkg, dep_ver, visited);
                } else {
                    println!("Dependency {} already installed.", dep_pkg);
                }
            }

            // --- 2. Install main package ---
            let bins_dir = bpm_store.join("bins");
            fs::create_dir_all(&bins_dir).unwrap();

            let installing_message = format!("Installing {}", package);
            let pb = create_progress_bar(pkg.binaries.len() as u64, &installing_message);
            let mut installed_bins = Vec::new();

            for bin in &pkg.binaries {
                let src = Path::new(&pkg.path).join(bin);
                let filename = Path::new(bin).file_name().unwrap();
                let dest = bins_dir.join(filename);

                fs::copy(&src, &dest).unwrap_or_else(|_| {
                    println!("Simulated copy {} -> {}", src.display(), dest.display());
                    0
                });

                // Save binary to DB
                let db_file = bpm_store.join("packages.db").to_string_lossy().to_string();
                let _ = save_to_db(&db_file, &filename.to_string_lossy(), &fs::read(&dest).unwrap(), false);

                installed_bins.push(dest.to_string_lossy().to_string());
                pb.inc(1);
            }
            pb.finish_with_message(format!("Installed {} successfully!", package));

            let mut db = load_installed(&bpm_store.join("installed.json"));
            db.insert(package.to_string(), InstalledPackage {
                repo: repo_name.to_string(),
                version: ver.to_string(),
                binaries: installed_bins,
            });
            save_installed(&bpm_store.join("installed.json"), &db);

            visited.remove(&key); // cleanup after install
        } else {
            println!("Version {} not found for package {}", ver, package);
        }
    } else {
        println!("Package {} not found in repo {}", package, repo_name);
    }
}

// Remove package
fn remove_package(bpm_store: &Path, package: &str) {
    let mut db = load_installed(&bpm_store.join("installed.json"));
    if let Some(pkg) = db.remove(package) {
        for bin in &pkg.binaries { let _ = fs::remove_file(bin); }
        save_installed(&bpm_store.join("installed.json"), &db);
        println!("Removed package {}", package);
    } else { println!("Package {} is not installed.", package); }
}

// Update package
fn update_package(bpm_store: &Path, repo_name: &str, package: &str) {
    remove_package(bpm_store, package);
    install_package(bpm_store, repo_name, package, None);
}

// List installed packages
fn list_installed(bpm_store: &Path) {
    let db = load_installed(&bpm_store.join("installed.json"));
    if db.is_empty() { println!("No packages installed."); }
    else {
        println!("Installed packages:");
        for (name, pkg) in db {
            println!("{} ({}): {:?}", name, pkg.version, pkg.binaries);
        }
    }
}

// CLI entry
fn main() {
    let bpm_store = PathBuf::from("C:/Users/User/Bpm-Store");
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: bpm /i|/r|/u|/l <repo:package[:version]> | /v, /h = help");
        return;
    }

    match args[1].as_str() {
        "/i" => {
            if args.len() < 3 { println!("Usage: bpm /i <repo:package[:version]>"); return; }
            let (repo, package, version) = parse_pkg_arg(&args[2]);
            install_package(&bpm_store, &repo, &package, version.as_deref());
        }
        "/r" => {
            if args.len() < 3 { println!("Usage: bpm /r <package>"); return; }
            remove_package(&bpm_store, &args[2]);
        }
        "/u" => {
            if args.len() < 3 { println!("Usage: bpm /u <repo:package>"); return; }
            let (repo, package, _) = parse_pkg_arg(&args[2]);
            update_package(&bpm_store, &repo, &package);
        }
        "/l" => list_installed(&bpm_store),
        "/v" => println!("bpm ver: {}", BPM_VERSION),
        "/h" => {
            println!("Blur Package Manager | Help Menu\n/i = install\n /r = remove\n  /u = update\n   /l = list installed packages\n    /v = shows version\n     /h = shows this menu");
        }
        _ => println!("Unknown command. Use /i, /r, /u, /l"),
    }
}
