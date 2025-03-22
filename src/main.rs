use walkdir::WalkDir;
use std::fs::{self, File};
use serde_json::{json, to_string_pretty};
use std::collections::HashMap;
use std::io::Write;
use rayon::prelude::*;

fn analyze_file(path: &std::path::Path) -> Option<(String, serde_json::Value)> {
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
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!(
            "codebase-analyzer2 v{}: Analyze code projects and summarize for Grok 3\n\
             Usage: codebase-analyzer2 [options] [path] [extensions] [output_file]\n\
             Options:\n\
             - --all: Analyze all files (ignore extensions)\n\
             - path: Directory to analyze (default: .)\n\
             - extensions: Comma-separated file types (default: py,rs,c,cpp,h,json)\n\
             - output_file: JSON output file (default: summary.json)",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

    let mut use_all = false;
    let mut start_idx = 1;
    if args.len() > 1 && args[1] == "--all" {
        use_all = true;
        start_idx = 2;
    }

    let path = args.get(start_idx).unwrap_or(&".".to_string()).clone();
    let default_extensions = vec![
        "py".to_string(), "rs".to_string(), "c".to_string(),
        "cpp".to_string(), "h".to_string(), "json".to_string()
    ];
    let extensions: Vec<String> = if use_all {
        vec![] // No extensions filter with --all
    } else {
        args.get(start_idx + 1)
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or(default_extensions)
    };
    let output_file = args.get(start_idx + (if use_all { 1 } else { 2 }))
        .unwrap_or(&"summary.json".to_string())
        .clone();

    let file_metrics: HashMap<String, serde_json::Value> = WalkDir::new(&path)
        .min_depth(1)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| match entry {
            Ok(e) => Some(e),
            Err(e) => {
                eprintln!("Warning: Skipping entry due to error: {}", e);
                None
            }
        })
        .filter_map(|entry| {
            let path = entry.path();
            if use_all || path.extension().and_then(|s| s.to_str()).map_or(false, |ext| extensions.contains(&ext.to_string())) {
                analyze_file(path)
            } else {
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