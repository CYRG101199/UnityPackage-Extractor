use clap::Parser;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tar::Archive;

#[derive(Parser, Debug)]
#[command(
    name = "unitypackage_extractor",
    about = "extract and restore .unitypackage files"
)]
struct Args {
    filename: String,
}

struct AssetData {
    path: Option<String>,
    asset: Option<PathBuf>,
}

fn extract_package(filename: &str) -> io::Result<()> {
    if !filename.ends_with(".unitypackage") {
        eprintln!("Invalid package");
        return Ok(());
    }

    println!("Opening package");
    let base_dir_name = filename.replace(".unitypackage", "");
    let temp_dir_name = format!("{}_temp", base_dir_name);
    fs::create_dir_all(&temp_dir_name)?;
    println!("Creating folder {}...", base_dir_name);
    fs::create_dir_all(&base_dir_name)?;

    let file = File::open(filename)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    let mut data_dir: HashMap<String, AssetData> = HashMap::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let path_str = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let filename = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();

        if filename == "asset.meta" {
            continue;
        }

        let asset_data = data_dir.entry(path_str.clone()).or_insert(AssetData { path: None, asset: None });

        if filename == "pathname" {
            let temp_file_path = Path::new(&temp_dir_name).join(&path);
            if let Some(parent) = temp_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(&temp_file_path)?;
            let mut file = File::open(&temp_file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            asset_data.path = Some(contents.trim().to_string());
        } else if filename == "asset" {
            let temp_file_path = Path::new(&temp_dir_name).join(&path);
            if let Some(parent) = temp_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(&temp_file_path)?;
            asset_data.asset = Some(temp_file_path);
        }
    }

    for asset in data_dir.values() {
        if let (Some(asset_path), Some(asset_file)) = (&asset.path, &asset.asset) {
            let rel_path = asset_path.trim_start_matches("Assets/");
            let out_path = Path::new(&base_dir_name).join(rel_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            print!("Creating file {}...", out_path.display());
            fs::copy(asset_file, &out_path)?;
            println!(" OK");
        }
    }

    fs::remove_dir_all(&temp_dir_name)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    extract_package(&args.filename)
} 