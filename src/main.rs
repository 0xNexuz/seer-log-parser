use clap::{Parser, Subcommand};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

/// Seer Log Parser CLI
#[derive(Parser)]
#[command(name = "seer-parser")]
#[command(about = "A lightweight CLI for parsing Solana program logs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parses a given text file for Solana errors
    Parse {
        /// The path to the log file to read
        #[arg(value_name = "FILE")]
        file_path: PathBuf,
    },
}

/// Struct to hold our extracted error data
struct ParsedError {
    program_id: String,
    error_code: String,
}

impl ParsedError {
    /// Provide a human-readable translation for common Anchor/Solana errors
    fn translate(&self) -> &str {
        if self.error_code.contains("0x1") {
            "Insufficient Funds"
        } else if self.error_code.contains("0xb") {
            "Blockhash not found"
        } else {
            "Unknown Custom Error - Check Anchor/Program docs"
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file_path } => {
            match fs::read_to_string(file_path) {
                Ok(contents) => {
                    // Regex to capture the standard Solana failure log
                    let re_failed = Regex::new(r"Program (\w+) failed:\s*(.+)").unwrap();
                    let mut errors: Vec<ParsedError> = Vec::new();

                    for line in contents.lines() {
                        if let Some(captures) = re_failed.captures(line) {
                            errors.push(ParsedError {
                                program_id: captures[1].to_string(),
                                error_code: captures[2].to_string(),
                            });
                        }
                    }

                    if errors.is_empty() {
                        println!("{}", "✅ No errors found in the log file.".green());
                    } else {
                        // Print the clean, formatted dashboard
                        for err in errors {
                            println!("\n{}", "🚨 SEER PARSER ALERT".red().bold());
                            println!("Failing Program ID: {}", err.program_id.yellow());
                            println!("Error Code: {}", err.error_code.red().bold());
                            println!("{}", format!("Translation: {}", err.translate()).green());
                            println!("{}", "--------------------------------------------------".bright_black());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} '{}': {}", "🚨 Error reading file at".red().bold(), file_path.display(), e);
                }
            }
        }
    }
}