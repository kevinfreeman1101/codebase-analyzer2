use walkdir::WalkDir;
use std::fs::{self, File};
use serde_json::{json, to_string_pretty};
use std::collections::HashMap;
use std::io::Write;
use rayon::prelude::*;

fn analyze_file(path: &std::path::Path) -> Option<(String, serde_json::Value)> {
    println!("Analyzing: {}", path.display());
    let metadata = fs::metadata(path).ok()?;
    let file_size = metadata.len();
    let content = fs::read_to_string(path).ok()?;
    let lines = content.lines().count();
    let complexity = content.lines()
        .filter(|l| {
            l.contains("if ") || l.contains("for ") || l.contains("while ") || 
            l.contains("fn ") || l.contains("def ") || l.contains("class ")
        })
        .count();
    let comment_lines = content.lines()
        .filter(|l| l.trim().starts_with("//") || l.trim().starts_with("#") || l.trim().starts_with("/*"))
        .count();
    let doc_coverage = if lines > 0 { comment_lines as f64 / lines as f64 * 100.0 } else { 0.0 };
    let deps: Vec<String> = content.lines()
        .filter_map(|line| {
            if line.trim().starts_with("use ") && line.ends_with(';') {
                Some(line.trim()[4..line.len()-1].to_string())
            } else if line.trim().starts_with("import ") {
                Some(line.trim()[7..].to_string())
            } else if line.trim().starts_with("#include ") {
                Some(line.trim()[9..].to_string())
            } else {
                None
            }
        })
        .collect();
    let assignments: Vec<String> = content.lines()
        .filter_map(|line| {
            if line.contains('=') && !line.trim().starts_with("if") && !line.trim().starts_with("for") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() > 1 { Some(parts[0].trim().to_string()) } else { None }
            } else {
                None
            }
        })
        .collect();
    let hotspots: Vec<String> = content.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if (line.contains("for ") && line.contains(" in ")) ||
               (line.contains("while ") && !line.contains("break")) ||
               line.contains("read_to_string") || line.contains("write") {
                Some(format!("Line {}: {}", i + 1, line.trim()))
            } else {
                None
            }
        })
        .collect();
    Some((
        path.display().to_string(),
        json!({
            "lines": lines,
            "size_bytes": file_size,
            "complexity": complexity,
            "doc_percent": doc_coverage,
            "deps": deps,
            "vars": assignments,
            "hotspots": hotspots
        })
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: codebase-analyzer2 [path] [--all] [extensions] [output_file]");
        return Ok(());
    }

    let mut path = ".".to_string();
    let mut use_all = false;
    let mut extensions: Vec<String> = vec![
        "py".to_string(), "rs".to_string(), "c".to_string(),
        "cpp".to_string(), "h".to_string(), "json".to_string()
    ];
    let mut output_file = "summary.json".to_string();
    let mut i = 1;

    // Parse path
    if !args[i].starts_with('-') {
        path = args[i].clone();
        i += 1;
    }

    // Parse --all
    if i < args.len() && args[i] == "--all" {
        use_all = true;
        extensions = vec![];
        i += 1;
    }

    // Parse extensions if not using --all
    if !use_all && i < args.len() && !args[i].starts_with('-') {
        extensions = args[i].split(',').map(String::from).collect();
        i += 1;
    }

    // Parse output file
    if i < args.len() {
        output_file = args[i].clone();
    }

    println!("Path: {}, Use all: {}, Extensions: {:?}, Output: {}", path, use_all, extensions, output_file);

    let file_metrics: HashMap<String, serde_json::Value> = WalkDir::new(&path)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| match entry {
            Ok(e) => {
                println!("Found entry: {}", e.path().display());
                Some(e)
            }
            Err(e) => {
                eprintln!("Warning: Skipping entry due to error: {}", e);
                None
            }
        })
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                println!("Skipping non-file: {}", path.display());
                return None;
            }
            if use_all || path.extension().and_then(|s| s.to_str()).map_or(false, |ext| extensions.contains(&ext.to_string())) {
                analyze_file(path)
            } else {
                println!("Skipping due to extension: {}", path.display());
                None
            }
        })
        .collect();

    let total_lines: usize = file_metrics.values()
        .map(|v| v["lines"].as_u64().unwrap_or(0) as usize)
        .sum();
    let total_size: u64 = file_metrics.values()
        .map(|v| v["size_bytes"].as_u64().unwrap_or(0))
        .sum();
    let errors: Vec<String> = Vec::new();

    let summary = json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "files_count": file_metrics.len(),
        "lines": total_lines,
        "size": total_size,
        "path": path,
        "exts": if use_all { "all".to_string() } else { extensions.join(",") },
        "errors": errors,
        "files": file_metrics
    });

    let mut file = File::create(&output_file).map_err(|e| {
        eprintln!("Error creating output file {}: {}", output_file, e);
        e
    })?;
    file.write_all(to_string_pretty(&summary)?.as_bytes())?;
    println!("Analysis written to {} ({} files)", output_file, file_metrics.len());
    Ok(())
}