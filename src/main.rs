use clap::{Parser, Subcommand};
use colored::Colorize;
use regex::Regex;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "seer-parser")]
#[command(about = "God-Tier Solana Log Parser with CPI Trees and Groq AI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Parse {
        #[arg(value_name = "FILE")]
        file_path: PathBuf,
    },
}

#[derive(Debug, Clone)]
struct ParsedError {
    program_id: String,
    error_code: String,
}

// The Groq AI API Call
async fn get_ai_summary(error: &ParsedError) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = match env::var("GROQ_API_KEY") {
        Ok(key) => key,
        Err(_) => return Ok("⚠️ GROQ_API_KEY not found in environment. Skipping AI summary.".yellow().to_string()),
    };

    let client = Client::new();
    let prompt = format!(
        "You are an elite Solana smart contract auditor. A transaction failed in program '{}' with error '{}'. Explain what this means and how to fix it in exactly 2 short sentences. Be direct, technical, and concise.",
        error.program_id, error.error_code
    );

    let res = client.post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": "llama-3.1-8b-instant", // Updated to the latest Groq model
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.2
        }))
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    
    //Check if the expected AI response exists
    if let Some(summary) = json["choices"][0]["message"]["content"].as_str() {
        Ok(summary.to_string())
    } else {
        //If it fails, print the error groq is sending back
        Ok(format!("⚠️ Groq API Response: {}", json["error"]["message"].as_str().unwrap_or("Unknown error").red()))
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file_path } => {
            match fs::read_to_string(file_path) {
                Ok(contents) => {
                    let re_invoke = Regex::new(r"Program (\w+) invoke \[(\d+)\]").unwrap();
                    let re_failed = Regex::new(r"Program (\w+) failed:\s*(.+)").unwrap();
                    
                    let mut first_error: Option<ParsedError> = None;

                    println!("\n{}", "🌳 CPI EXECUTION TREE".bright_blue().bold());
                    println!("{}", "==================================================".bright_black());

                    // Parse Tree & Find Errors
                    for line in contents.lines() {
                        if let Some(caps) = re_invoke.captures(line) {
                            let prog_id = &caps[1];
                            let depth: usize = caps[2].parse().unwrap_or(1);
                            
                            let indent = "  ".repeat(depth.saturating_sub(1));
                            let branch = if depth == 1 { "▶" } else { "└──" };
                            
                            println!("{}{} {}", indent, branch.bright_black(), prog_id.cyan());
                        }

                        if let Some(caps) = re_failed.captures(line) {
                            let prog_id = caps[1].to_string();
                            let err_code = caps[2].to_string();
                            
                            println!("    {} {} failed: {}", "❌".red(), prog_id.yellow(), err_code.red());

                            if first_error.is_none() {
                                first_error = Some(ParsedError {
                                    program_id: prog_id,
                                    error_code: err_code,
                                });
                            }
                        }
                    }

                    println!("{}", "==================================================\n".bright_black());

                    // Trigger Groq AI for the first captured error
                    if let Some(err) = first_error {
                        println!("{}", "🚨 SEER DIAGNOSTICS ALERT".red().bold());
                        println!("Target: {}", err.program_id.yellow());
                        println!("Code:   {}", err.error_code.red().bold());
                        
                        println!("\n{}", "🧠 GROQ AI ANALYSIS:".bright_magenta().bold());
                        println!("{}", "Fetching sub-second inference...".bright_black());
                        
                        match get_ai_summary(&err).await {
                            Ok(summary) => println!("{}", summary.green()),
                            Err(e) => println!("{} {}", "AI Error:".red(), e),
                        }
                        println!("\n{}", "--------------------------------------------------".bright_black());
                    } else {
                        println!("{}", "✅ Transaction successful. No errors detected.".green());
                    }
                }
                Err(e) => eprintln!("{} '{}': {}", "🚨 Error reading file".red(), file_path.display(), e),
            }
        }
    }
}