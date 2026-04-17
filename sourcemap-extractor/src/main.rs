use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize)]
struct SourceMap {
    sources: Vec<String>,
    #[serde(rename = "sourcesContent")]
    sources_content: Option<Vec<Option<String>>>,
}

fn sanitize_path(source_path: &str, index: usize) -> String {
    let mut rel_path = source_path.to_string();
    
    if let Some(idx) = rel_path.find("node_modules/") {
        rel_path = rel_path[idx..].to_string();
    }
    
    rel_path = rel_path
        .replace("webpack:///", "")
        .replace("webpack://", "");
        
    if let Some(idx) = rel_path.find('?') {
        rel_path = rel_path[..idx].to_string();
    }
    
    if rel_path.starts_with("../") || rel_path.starts_with("/../") {
        rel_path = rel_path.trim_start_matches('/').trim_start_matches("../").to_string();
    }

    if rel_path.is_empty() || rel_path == "webpack/bootstrap" {
        rel_path = format!("__webpack__/source_{}.js", index);
    }

    rel_path = rel_path.trim_start_matches('/').to_string();
    rel_path = rel_path.replace("../", "_dotdot_/");
    
    rel_path
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <path-to-map-file> <output-directory>", args[0]);
        std::process::exit(1);
    }
    
    let map_file = std::path::Path::new(&args[1]);
    let out_dir = std::path::Path::new(&args[2]);
    
    println!("Reading source map from {}...", map_file.display());
    let map_content = fs::read_to_string(map_file)?;
    
    println!("Parsing JSON...");
    let raw_map: SourceMap = serde_json::from_str(&map_content)?;
    
    println!("Sources count: {}", raw_map.sources.len());
    
    let mut written = 0;
    let mut skipped = 0;
    
    if let Some(contents) = raw_map.sources_content {
        for (i, source_path) in raw_map.sources.iter().enumerate() {
            if i < contents.len() {
                if let Some(content) = &contents[i] {
                    let rel_path = sanitize_path(source_path, i);
                    let full_path = out_dir.join(&rel_path);
                    
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    fs::write(&full_path, content)?;
                    written += 1;
                    
                    if written % 500 == 0 {
                        println!("Written {} files...", written);
                    }
                } else {
                    skipped += 1;
                }
            } else {
                skipped += 1;
            }
        }
    } else {
        println!("No sourcesContent found in the map file!");
    }
    
    println!("Done. Written: {}, Skipped (no content): {}", written, skipped);
    
    Ok(())
}
