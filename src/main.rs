use std::env;
use std::io::{self, Read};
use std::process;

mod core;
mod parser;

fn parse_args(args: &[String]) -> Result<core::Config, String> {
    if args.len() < 2 {
        return Err(format!(
            "Usage: {} [--color[=WHEN]] [-r] [-o [-P]] -E <pattern> [file...]",
            args[0]
        ));
    }

    let mut only_matching = false;
    let mut multi_line = false;
    let mut color_mode = "never".to_string();
    let mut recursive = false;
    let mut debug = false;
    let mut pattern = String::new();
    let mut filenames = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--color" => {
                color_mode = "always".to_string();
                i += 1;
            }
            "--color=always" => {
                color_mode = "always".to_string();
                i += 1;
            }
            "--color=auto" => {
                color_mode = "auto".to_string();
                i += 1;
            }
            "--color=never" => {
                color_mode = "never".to_string();
                i += 1;
            }
            "--debug" => {
                debug = true;
                i += 1;
            }
            "-r" => {
                recursive = true;
                i += 1;
            }
            "-o" => {
                only_matching = true;
                i += 1;
            }
            "-P" => {
                if only_matching {
                    multi_line = true;
                }
                i += 1;
            }
            "-E" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} [--color[=WHEN]] [-r] [-o [-P]] -E <pattern> [file...]",
                        args[0]
                    ));
                }
                pattern = args[i + 1].clone();
                i += 2;
                // Collect remaining args as filenames
                while i < args.len() {
                    filenames.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            _ => {
                return Err(format!(
                    "Usage: {} [--color[=WHEN]] [-r] [-o [-P]] -E <pattern> [file...]",
                    args[0]
                ));
            }
        }
    }

    if pattern.is_empty() {
        return Err(format!(
            "Usage: {} [--color[=WHEN]] [-r] [-o [-P]] -E <pattern> [file...]",
            args[0]
        ));
    }

    Ok(core::Config {
        pattern,
        filenames,
        color_mode,
        recursive,
        only_matching,
        multi_line,
        debug,
    })
}

fn process_stdin(config: &core::Config) -> Result<bool, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(core::search_file(&input, config, None))
}

fn process_files(config: &core::Config) -> Result<bool, Box<dyn std::error::Error>> {
    let file_paths = core::collect_files_recursive(&config.filenames, config.recursive);
    let should_prefix = file_paths.len() > 1 || config.recursive;
    let mut matched_any = false;

    for filename in &file_paths {
        let content = std::fs::read_to_string(filename)?;
        let filename_opt = if should_prefix {
            Some(filename.as_str())
        } else {
            None
        };

        if core::search_file(&content, config, filename_opt) {
            matched_any = true;
        }
    }

    Ok(matched_any)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match parse_args(&args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    let matched_any = if config.filenames.is_empty() {
        process_stdin(&config).unwrap_or(false)
    } else {
        process_files(&config).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        })
    };

    if matched_any {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
