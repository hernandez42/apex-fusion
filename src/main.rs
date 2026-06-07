//! APEX Fusion CLI

use apex_fusion::{fuse_compress, FusionConfig};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { print_usage(); return; }

    match args[1].as_str() {
        "compress" => {
            if args.len() < 4 { eprintln!("Usage: apex-fusion compress <tool_type> <content>"); return; }
            let tool_type = &args[2];
            let content = &args[3];
            let session_id = args.iter().position(|a| a == "--session")
                .map(|i| args[i+1].clone())
                .unwrap_or_else(|| "default".to_string());
            let config = FusionConfig::default();
            let result = fuse_compress(content, tool_type, &session_id, None, &config);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        "file" => {
            if args.len() < 3 { eprintln!("Usage: apex-fusion file <tool_type> <file_path>"); return; }
            let tool_type = &args[2];
            let file_path = &args[3];
            let content = fs::read_to_string(file_path).expect("Failed to read file");
            let config = FusionConfig::default();
            let result = fuse_compress(&content, tool_type, "cli", None, &config);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        "benchmark" => run_benchmark(),
        "stats" => {
            if args.len() < 3 { eprintln!("Usage: apex-fusion stats <input>"); return; }
            let input = &args[2];
            println!("Input length: {} chars", input.len());
            println!("Estimated tokens: ~{}", input.len() / 4);
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("APEX Fusion - RTK + CBM + Headroom + Caveman");
    println!("Usage:");
    println!("  apex-fusion compress <tool_type> <content>    Compress content");
    println!("  apex-fusion file <tool_type> <file_path>     Compress file");
    println!("  apex-fusion benchmark                        Run benchmark");
    println!("  apex-fusion stats <input>                   Show stats");
    println!("Tool types: Read, Bash, Grep, Glob, WebFetch, WebSearch");
}

fn run_benchmark() {
    println!("=== APEX Fusion Benchmark ===\n");
    let long_bash_output = "output line 0\n".to_string() + &(0..499).map(|i| format!("output line {}\n", i + 1)).collect::<String>();
    let test_cases = vec![
        ("Bash (repetitive)", "line1\nline1\nline1\nline1\nline2\nline2\nline3\nline4\nline5"),
        ("Bash (long output)", long_bash_output.as_str()),
        ("Read (code)", "fn main() {\n    println!(\"Hello\");\n    let x = 1;\n    let y = 2;\n    println!(\"{}\", x + y);\n}"),
        ("JSON (repeating)", r#"[{"id": 1, "name": "test"}, {"id": 1, "name": "test"}, {"id": 1, "name": "test"}, {"id": 2, "name": "other"}]"#),
        ("JSON (nested)", r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}], "count": 2}"#),
        ("Text (verbose)", "The system is designed to be highly efficient and is intended to optimize the performance of database operations. In order to achieve this goal, the system uses advanced indexing strategies and caching mechanisms. Therefore, we can expect significant improvements in query speed."),
        ("Text (technical)", "The database handles approximately 1 million requests per second. Located in San Francisco, California. The system uses O(log n) binary search algorithms for efficient data retrieval."),
    ];
    let config = FusionConfig::default();
    let mut total_original = 0;
    let mut total_compressed = 0;
    for (name, content) in test_cases {
        let tool_type = if name.contains("Bash") { "Bash" } else if name.contains("Read") { "Read" } else if name.contains("JSON") { "Bash" } else { "Read" };
        let result = fuse_compress(content, tool_type, "benchmark", None, &config);
        println!("{}: {} → {} chars ({:.1}% saved)", name, result.original_len, result.compressed_len, result.savings_pct);
        println!("  Layers: {}", result.layers_applied.join(" → "));
        total_original += result.original_len;
        total_compressed += result.compressed_len;
    }
    let overall = (total_original - total_compressed) as f64 / total_original as f64 * 100.0;
    println!("\n=== Overall: {} → {} chars ({:.1}% saved) ===", total_original, total_compressed, overall);
}
