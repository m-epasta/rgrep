use crate::parser::types::{Token, Quantifiers};

pub fn tokenize(pattern: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        let mut token = match ch {
            '\\' => match chars.next() {
                Some('d') => Token::Digit,
                Some('w') => Token::Word,
                Some(c) if c.is_ascii_digit() => {
                    // Handle backreferences \1, \2, etc.
                    let mut num_str = c.to_string();
                    // Allow multi-digit backreferences
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    match num_str.parse::<usize>() {
                        Ok(n) if n > 0 => Token::BackReference(n),
                        _ => Token::Literal(c), // fallback for invalid backref
                    }
                }
                Some(c) => Token::Literal(c),
                None => continue,
            },
            '.' => Token::WildCard,
            '^' => Token::StartAnchor,
            '$' => Token::EndAnchor,

            '[' => {
                let mut group = Vec::new();
                let mut neg = false;

                if chars.peek() == Some(&'^') {
                    neg = true;
                    chars.next();
                }

                while let Some(c) = chars.next() {
                    if c == ']' {
                        break;
                    }
                    group.push(c);
                }

                if neg {
                    Token::NegCharGroup(group)
                } else {
                    Token::CharGroup(group)
                }
            }
            '(' => {
                let mut depth = 1;
                let mut content = String::new();

                while let Some(c) = chars.next() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    content.push(c);
                }

                // Check if this contains alternation
                if content.contains('|') {
                    let parts = split_top_level(&content, '|');
                    let branches = parts.into_iter().map(|p| tokenize(&p)).collect();
                    Token::CaptureGroup(0, vec![Token::Alternation(branches)])
                } else {
                    // Simple capturing group
                    let inner_tokens = tokenize(&content);
                    Token::CaptureGroup(0, inner_tokens)
                }
            }
            other => Token::Literal(other),
        };

        // Check for quantifier after token creation
        if let Some(&next) = chars.peek() {
            match next {
                '+' => {
                    chars.next();
                    token = Token::Quantifier(Box::new(token), Quantifiers::OneOrMore);
                }
                '?' => {
                    chars.next();
                    token = Token::Quantifier(Box::new(token), Quantifiers::ZeroOrOne);
                }
                '*' => {
                    chars.next();
                    token = Token::Quantifier(Box::new(token), Quantifiers::ZeroOrMore);
                }
                '{' => {
                    chars.next(); // consume '{'
                    let mut num_str = String::new();
                    let mut min: Option<usize> = None;
                    let mut max: Option<usize> = None;
                    let mut parsing_max = false;

                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        } else if c == ',' {
                            if let Ok(parsed_min) = num_str.parse::<usize>() {
                                min = Some(parsed_min);
                                num_str.clear();
                                parsing_max = true;
                            } else {
                                // Invalid, treat as literal
                                token = Token::Literal('{');
                                break;
                            }
                        } else if c.is_ascii_digit() {
                            num_str.push(c);
                        } else {
                            // Invalid syntax
                            token = Token::Literal('{');
                            break;
                        }
                    }

                    // If we have a comma, it's {min,} or {min,max}
                    if parsing_max {
                        if num_str.is_empty() {
                            // {n,} case
                            max = None;
                        } else if let Ok(parsed_max) = num_str.parse::<usize>() {
                            max = Some(parsed_max);
                        } else {
                            token = Token::Literal('{');
                        }
                        if let Some(min_val) = min {
                            token = Token::RangeRepetition(Box::new(token), min_val, max);
                        } else {
                            token = Token::Literal('{');
                        }
                    } else {
                        // No comma, it's {n} exact case
                        if let Ok(count) = num_str.parse::<usize>() {
                            token = Token::ExactRepetition(Box::new(token), count);
                        } else {
                            token = Token::Literal('{');
                        }
                    }
                }
                _ => {}
            }
        }

        tokens.push(token);
    }

    tokens
}

fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }

        if c == sep && depth == 0 {
            parts.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }

    parts.push(current);
    parts
}
