use std::{fs, path::Path};

const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

pub static mut ITERATION_COUNT: usize = 0;
pub const MAX_ITERATIONS: usize = 2000;

pub fn debug_log(config: Option<&Config>, message: &str) {
    unsafe {
        ITERATION_COUNT += 1;
        if ITERATION_COUNT > MAX_ITERATIONS {
            return;
        }
    }

    if let Some(c) = config {
        if c.debug {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("debug.log")
                .unwrap();
            writeln!(file, "{}", message).unwrap();
        }
    }
}

fn reset_iteration_count() {
    unsafe {
        ITERATION_COUNT = 0;
    }
}

pub fn log_iteration_header(config: Option<&Config>, iteration: usize) {
    if let Some(c) = config {
        if c.debug {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("debug.log")
                .unwrap();
            writeln!(file, "\n=== Iteration {} ===", iteration).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub pattern: String,
    pub filenames: Vec<String>,
    pub color_mode: String,
    pub recursive: bool,
    pub only_matching: bool,
    pub multi_line: bool,
    pub debug: bool,
}

pub fn collect_files_recursive(paths: &[String], recursive: bool) -> Vec<String> {
    let mut files = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            files.push(path_str.clone());
        } else if path.is_dir() {
            if recursive {
                collect_from_dir(path, &mut files);
            } else {
                eprintln!("{}: Is a directory", path_str);
                std::process::exit(1);
            }
        } else {
            eprintln!("{}: No such file or directory", path_str);
            std::process::exit(1);
        }
    }

    files
}

fn collect_from_dir(dir: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                // Skip hidden directories like .git
                if !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
                {
                    collect_from_dir(&path, files);
                }
            }
        }
    }
}

pub fn search_file(content: &str, config: &Config, filename: Option<&str>) -> bool {
    let mut matched_any = false;
    let is_colored = match config.color_mode.as_str() {
        "always" => true,
        "auto" => atty::is(atty::Stream::Stdout),
        _ => false,
    };

    for line in content.lines() {
        let matched = if config.multi_line {
            check_multiples_matching_patterns(line, &config.pattern, is_colored, filename)
        } else if config.only_matching {
            check_only_matching_patterns(line, &config.pattern, is_colored, filename)
        } else {
            check_pattern(line, &config.pattern, is_colored, filename, config)
        };

        if matched {
            matched_any = true;
        }
    }

    matched_any
}

fn match_pattern(input: &str, pattern: &str, config: &Config) -> bool {
    reset_iteration_count();
    debug_log(
        Some(config),
        &format!("Matching pattern '{}' against input '{}'", pattern, input),
    );
    let mut tokens = crate::parser::token::tokenize(pattern);
    let mut group_counter = 1;
    crate::parser::token::assign_group_numbers(&mut tokens, &mut group_counter);
    debug_log(
        Some(config),
        &format!("Tokens after group assignment: {:?}", tokens),
    );
    let result = crate::parser::token::match_tokens(input, &tokens, config);
    debug_log(Some(config), &format!("Match result: {}", result));
    result
}

fn highlight_matches_in_line(input_line: &str, pattern: &str) -> String {
    let mut tokens = crate::parser::token::tokenize(pattern);
    let mut group_counter = 1;
    crate::parser::token::assign_group_numbers(&mut tokens, &mut group_counter);
    let input_chars: Vec<char> = input_line.chars().collect();
    let has_start = matches!(
        tokens.first(),
        Some(crate::parser::token::Token::StartAnchor)
    );
    let has_end = matches!(tokens.last(), Some(crate::parser::token::Token::EndAnchor));
    let mut pos = 0;
    let mut result = String::new();
    while pos < input_chars.len() {
        if has_start && pos != 0 {
            break;
        }
        let mut dummy_captures = Vec::new();
        if let Some(match_len) = crate::parser::token::matches_from_range(
            &input_chars,
            &tokens,
            pos,
            None,
            &mut dummy_captures,
        ) {
            let match_end = pos + match_len;
            if has_end && match_end != input_chars.len() {
                // No match, add the current char and continue
                result.push(input_chars[pos]);
                pos += 1;
                continue;
            }
            // Add the matched part with color
            let start_byte = char_to_byte(input_line, pos);
            let end_byte = char_to_byte(input_line, match_end);
            result.push_str(&format!(
                "{}{}{}",
                RED,
                &input_line[start_byte..end_byte],
                RESET
            ));
            pos = match_end;
        } else {
            // No match, add the current char
            result.push(input_chars[pos]);
            pos += 1;
        }
    }
    // Add remaining characters if any
    if pos < input_chars.len() {
        let start_byte = char_to_byte(input_line, pos);
        result.push_str(&input_line[start_byte..]);
    }
    result
}

pub fn check_pattern(
    input_line: &str,
    pattern: &str,
    is_colored: bool,
    filename: Option<&str>,
    config: &Config,
) -> bool {
    if match_pattern(&input_line, &pattern, config) {
        let output = if is_colored {
            highlight_matches_in_line(input_line, pattern)
        } else {
            input_line.to_string()
        };

        match filename {
            Some(fname) => println!("{}:{}", fname, output),
            None => println!("{}", output),
        }
        true
    } else {
        false
    }
}

fn char_to_byte(s: &str, char_index: usize) -> usize {
    s.chars().take(char_index).map(|c| c.len_utf8()).sum()
}

pub fn check_only_matching_patterns(
    input_line: &str,
    pattern: &str,
    is_colored: bool,
    filename: Option<&str>,
) -> bool {
    let mut tokens = crate::parser::token::tokenize(pattern);
    let mut group_counter = 1;
    crate::parser::token::assign_group_numbers(&mut tokens, &mut group_counter);
    let input_chars: Vec<char> = input_line.chars().collect();
    let has_start = matches!(
        tokens.first(),
        Some(crate::parser::token::Token::StartAnchor)
    );
    let has_end = matches!(tokens.last(), Some(crate::parser::token::Token::EndAnchor));
    let mut pos = 0;
    let mut found = false;
    while pos < input_chars.len() {
        if has_start && pos != 0 {
            break;
        }
        let mut dummy_captures = Vec::new();
        if let Some(match_len) = crate::parser::token::matches_from_range(
            &input_chars,
            &tokens,
            pos,
            None,
            &mut dummy_captures,
        ) {
            let match_end = pos + match_len;
            if has_end && match_end != input_chars.len() {
                pos += 1;
                continue;
            }
            let start_byte = char_to_byte(input_line, pos);
            let end_byte = char_to_byte(input_line, match_end);
            let output = if is_colored {
                format!("{}{}{}", RED, &input_line[start_byte..end_byte], RESET)
            } else {
                input_line[start_byte..end_byte].to_string()
            };

            match filename {
                Some(fname) => println!("{}:{}", fname, output),
                None => println!("{}", output),
            }
            found = true;
            pos = match_end;
        } else {
            pos += 1;
        }
    }
    found
}

pub fn check_multiples_matching_patterns(
    line: &str,
    pattern: &str,
    is_colored: bool,
    filename: Option<&str>,
) -> bool {
    let mut tokens = crate::parser::token::tokenize(pattern);
    let mut group_counter = 1;
    crate::parser::token::assign_group_numbers(&mut tokens, &mut group_counter);
    let input_chars: Vec<char> = line.chars().collect();
    let has_start = matches!(
        tokens.first(),
        Some(crate::parser::token::Token::StartAnchor)
    );
    let has_end = matches!(tokens.last(), Some(crate::parser::token::Token::EndAnchor));
    let mut pos = 0;
    let mut found = false;
    while pos < input_chars.len() {
        if has_start && pos != 0 {
            break;
        }
        let mut dummy_captures = Vec::new();
        if let Some(match_len) = crate::parser::token::matches_from_range(
            &input_chars,
            &tokens,
            pos,
            None,
            &mut dummy_captures,
        ) {
            let match_end = pos + match_len;
            if has_end && match_end != input_chars.len() {
                pos += 1;
                continue;
            }
            let start_byte = char_to_byte(line, pos);
            let end_byte = char_to_byte(line, match_end);
            let output = if is_colored {
                format!("{}{}{}", RED, &line[start_byte..end_byte], RESET)
            } else {
                line[start_byte..end_byte].to_string()
            };

            match filename {
                Some(fname) => println!("{}:{}", fname, output),
                None => println!("{}", output),
            }
            found = true;
            pos = match_end;
        } else {
            pos += 1;
        }
    }
    found
}
