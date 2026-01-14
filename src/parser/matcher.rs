use crate::parser::types::Token;
use crate::core::Config;

/// Recursive function similar to matches_from, but returns the length of the match if successful
pub fn matches_from_range(
    input: &[char],
    tokens: &[Token],
    input_index: usize,
    config: Option<&Config>,
    captures: &mut Vec<Option<String>>,
) -> Option<usize> {
    // Log iteration header
    unsafe {
        if crate::core::ITERATION_COUNT <= crate::core::MAX_ITERATIONS {
            crate::core::log_iteration_header(config, crate::core::ITERATION_COUNT);
        }
    }

    if tokens.is_empty() {
        crate::core::debug_log(
            config,
            &format!("matches_from_range: empty tokens, returning 0"),
        );
        return Some(0);
    }

    let token = &tokens[0];
    let _full_len = input.len();

    crate::core::debug_log(
        config,
        &format!(
            "matches_from_range: pos={}, token={:?}, remaining_tokens={}",
            input_index,
            token,
            tokens.len()
        ),
    );

    match token {
        Token::StartAnchor => {
            if input_index != 0 {
                crate::core::debug_log(config, &format!("StartAnchor failed: input_index != 0"));
                return None;
            }
            crate::core::debug_log(config, &format!("StartAnchor matched"));
            matches_from_range(input, &tokens[1..], input_index, config, captures)
        }
        Token::EndAnchor => {
            if input_index != input.len() {
                crate::core::debug_log(
                    config,
                    &format!(
                        "EndAnchor failed: input_index={} != input.len()={}",
                        input_index,
                        input.len()
                    ),
                );
                return None;
            }
            crate::core::debug_log(config, &format!("EndAnchor matched"));
            matches_from_range(input, &tokens[1..], input_index, config, captures)
        }
        Token::Quantifier(inner, quant) => match quant {
            crate::parser::types::Quantifiers::OneOrMore => {
                crate::core::debug_log(config, &format!("Quantifier OneOrMore: inner={:?}", inner));
                // Collect all possible match positions
                let mut positions = vec![input_index];
                let mut current_pos = input_index;
                while let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    captures,
                ) {
                    crate::core::debug_log(
                        config,
                        &format!("OneOrMore: matched inner at {}, len={}", current_pos, len),
                    );
                    current_pos += len;
                    positions.push(current_pos);
                }
                crate::core::debug_log(config, &format!("OneOrMore positions: {:?}", positions));

                if positions.len() <= 1 {
                    crate::core::debug_log(config, &format!("OneOrMore failed: no matches"));
                    return None; // need at least 1 match
                }

                // Try with different numbers of matches (longest first - greedy)
                for &pos in positions.iter().rev() {
                    // try longest matches first
                    if pos == input_index {
                        continue;
                    } // skip 0 matches
                    crate::core::debug_log(config, &format!("OneOrMore trying pos={}", pos));
                    let saved_captures = captures.clone();
                    if let Some(rest_len) =
                        matches_from_range(input, &tokens[1..], pos, config, captures)
                    {
                        let total = pos - input_index + rest_len;
                        crate::core::debug_log(
                            config,
                            &format!("OneOrMore success: total_len={}", total),
                        );
                        return Some(total);
                    } else {
                        *captures = saved_captures;
                    }
                }
                crate::core::debug_log(config, &format!("OneOrMore failed: no rest match"));
                None
            }
            crate::parser::types::Quantifiers::ZeroOrMore => {
                crate::core::debug_log(
                    config,
                    &format!("Quantifier ZeroOrMore: inner={:?}", inner),
                );
                // Collect all possible match positions (including 0 matches)
                let mut positions = vec![input_index];
                let mut current_pos = input_index;
                while let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    captures,
                ) {
                    crate::core::debug_log(
                        config,
                        &format!("ZeroOrMore: matched inner at {}, len={}", current_pos, len),
                    );
                    current_pos += len;
                    positions.push(current_pos);
                }
                crate::core::debug_log(config, &format!("ZeroOrMore positions: {:?}", positions));

                // Try with different numbers of matches (longest first - greedy)
                for &pos in positions.iter().rev() {
                    crate::core::debug_log(config, &format!("ZeroOrMore trying pos={}", pos));
                    let saved_captures = captures.clone();
                    if let Some(rest_len) =
                        matches_from_range(input, &tokens[1..], pos, config, captures)
                    {
                        let total = pos - input_index + rest_len;
                        crate::core::debug_log(
                            config,
                            &format!("ZeroOrMore success: total_len={}", total),
                        );
                        return Some(total);
                    } else {
                        *captures = saved_captures;
                    }
                }
                crate::core::debug_log(config, &format!("ZeroOrMore failed: no rest match"));
                None
            }
            crate::parser::types::Quantifiers::ZeroOrOne => {
                crate::core::debug_log(config, &format!("Quantifier ZeroOrOne: inner={:?}", inner));
                // try 1 match
                let saved_captures = captures.clone();
                if let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    input_index,
                    config,
                    captures,
                ) {
                    crate::core::debug_log(
                        config,
                        &format!("ZeroOrOne: trying 1 match, len={}", len),
                    );
                    if let Some(rest_len) =
                        matches_from_range(input, &tokens[1..], input_index + len, config, captures)
                    {
                        let total = len + rest_len;
                        crate::core::debug_log(
                            config,
                            &format!("ZeroOrOne success with 1 match: total_len={}", total),
                        );
                        return Some(total);
                    } else {
                        *captures = saved_captures;
                    }
                } else {
                    *captures = saved_captures;
                }
                // try 0 match
                crate::core::debug_log(config, &format!("ZeroOrOne: trying 0 match"));
                matches_from_range(input, &tokens[1..], input_index, config, captures)
            }
        },
        Token::ExactRepetition(inner, count) => {
            crate::core::debug_log(
                config,
                &format!("ExactRepetition: inner={:?}, count={}", inner, count),
            );
            let mut current_pos = input_index;
            for i in 0..*count {
                crate::core::debug_log(
                    config,
                    &format!(
                        "ExactRepetition: matching instance {} at pos {}",
                        i + 1,
                        current_pos
                    ),
                );
                if let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    captures,
                ) {
                    current_pos += len;
                } else {
                    crate::core::debug_log(
                        config,
                        &format!("ExactRepetition failed: couldn't match instance {}", i + 1),
                    );
                    return None;
                }
            }
            crate::core::debug_log(
                config,
                &format!("ExactRepetition success: matched {} instances", count),
            );
            if let Some(rest_len) =
                matches_from_range(input, &tokens[1..], current_pos, config, captures)
            {
                Some(current_pos - input_index + rest_len)
            } else {
                crate::core::debug_log(
                    config,
                    &format!("ExactRepetition failed: rest didn't match"),
                );
                None
            }
        }
        Token::RangeRepetition(inner, min, max) => {
            crate::core::debug_log(
                config,
                &format!(
                    "RangeRepetition: inner={:?}, min={}, max={:?}",
                    inner, min, max
                ),
            );
            let mut current_pos = input_index;
            let mut count = 0;

            // Match minimum required occurrences
            for i in 0..*min {
                crate::core::debug_log(
                    config,
                    &format!(
                        "RangeRepetition: matching required instance {} at pos {}",
                        i + 1,
                        current_pos
                    ),
                );
                if let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    captures,
                ) {
                    current_pos += len;
                    count += 1;
                } else {
                    crate::core::debug_log(
                        config,
                        &format!(
                            "RangeRepetition failed: couldn't match required instance {}",
                            i + 1
                        ),
                    );
                    return None;
                }
            }

            // For {n,} (unlimited max), greedily match as many as possible
            // For {n,m}, match up to the maximum
            if let Some(max_val) = max {
                // {n,m} - match up to maximum
                while count < *max_val {
                    crate::core::debug_log(
                        config,
                        &format!(
                            "RangeRepetition: trying optional instance {} at pos {}",
                            count + 1,
                            current_pos
                        ),
                    );
                    if let Some(len) = matches_from_range(
                        input,
                        &[inner.as_ref().clone()],
                        current_pos,
                        config,
                        captures,
                    ) {
                        current_pos += len;
                        count += 1;
                    } else {
                        break;
                    }
                }
            } else {
                // {n,} - match as many as possible
                while let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    captures,
                ) {
                    crate::core::debug_log(
                        config,
                        &format!(
                            "RangeRepetition: matched optional instance {} at pos {}",
                            count + 1,
                            current_pos
                        ),
                    );
                    current_pos += len;
                    count += 1;
                }
            }

            crate::core::debug_log(
                config,
                &format!(
                    "RangeRepetition success: matched {} instances (min={}, max={:?})",
                    count, min, max
                ),
            );
            if let Some(rest_len) =
                matches_from_range(input, &tokens[1..], current_pos, config, captures)
            {
                Some(current_pos - input_index + rest_len)
            } else {
                crate::core::debug_log(
                    config,
                    &format!("RangeRepetition failed: rest didn't match"),
                );
                None
            }
        }
        Token::Alternation(branches) => {
            crate::core::debug_log(config, &format!("Alternation: {} branches", branches.len()));
            for (i, branch) in branches.iter().enumerate() {
                crate::core::debug_log(config, &format!("Alternation trying branch {}", i));
                let saved_captures = captures.clone();
                let mut combined = branch.clone();
                combined.extend_from_slice(&tokens[1..]);
                if let Some(len) =
                    matches_from_range(input, &combined, input_index, config, captures)
                {
                    crate::core::debug_log(
                        config,
                        &format!("Alternation success with branch {}: len={}", i, len),
                    );
                    return Some(len);
                } else {
                    *captures = saved_captures;
                }
            }
            crate::core::debug_log(config, &format!("Alternation failed"));
            None
        }
        Token::CaptureGroup(group_num, inner_tokens) => {
            crate::core::debug_log(
                config,
                &format!("CaptureGroup {}: matching inner tokens", group_num),
            );

            // Collect all possible match lengths from inner tokens
            let possible_lengths =
                collect_all_match_lengths(input, inner_tokens, input_index, config, &*captures);
            crate::core::debug_log(
                config,
                &format!(
                    "CaptureGroup {}: possible lengths = {:?}",
                    group_num, possible_lengths
                ),
            );

            // Try lengths from longest to shortest (greedy with backtracking)
            for &len in possible_lengths.iter().rev() {
                let saved_captures = captures.clone();
                // Use group number as capture index (1-based to 0-based)
                let capture_index = group_num - 1;
                if capture_index >= captures.len() {
                    captures.resize(capture_index + 1, None);
                }
                // Capture the matched substring
                let matched_str: String = input[input_index..input_index + len].iter().collect();
                captures[capture_index] = Some(matched_str.clone());
                crate::core::debug_log(
                    config,
                    &format!(
                        "CaptureGroup {} trying: captured '{}' at index {}, len={}",
                        group_num, matched_str, capture_index, len
                    ),
                );

                // Re-run inner tokens with real captures to populate nested groups
                // This ensures nested capture groups are recorded in the captures vector
                let inner_match =
                    matches_from_range(input, inner_tokens, input_index, config, captures);
                if inner_match.is_none() {
                    // Inner match doesn't match, skip
                    *captures = saved_captures;
                    continue;
                }

                if let Some(rest_len) =
                    matches_from_range(input, &tokens[1..], input_index + len, config, captures)
                {
                    crate::core::debug_log(
                        config,
                        &format!(
                            "CaptureGroup {} success: captured '{}', total_len={}",
                            group_num,
                            matched_str,
                            len + rest_len
                        ),
                    );
                    return Some(len + rest_len);
                } else {
                    *captures = saved_captures;
                }
                crate::core::debug_log(
                    config,
                    &format!("CaptureGroup {} backtracking from len={}", group_num, len),
                );
            }

            crate::core::debug_log(
                config,
                &format!("CaptureGroup {} failed: no match found", group_num),
            );
            None
        }
        Token::BackReference(n) => {
            crate::core::debug_log(config, &format!("BackReference: checking capture {}", n));
            if input_index >= input.len() {
                crate::core::debug_log(
                    config,
                    &format!("BackReference failed: input_index >= input.len()"),
                );
                return None;
            }
            if let Some(Some(captured)) = captures.get(n - 1) {
                let captured_chars: Vec<char> = captured.chars().collect();
                crate::core::debug_log(
                    config,
                    &format!(
                        "BackReference: trying to match '{}' at pos {}",
                        captured, input_index
                    ),
                );
                if input[input_index..].starts_with(&captured_chars) {
                    let len = captured_chars.len();
                    crate::core::debug_log(
                        config,
                        &format!("BackReference success: matched '{}' len={}", captured, len),
                    );
                    if let Some(rest_len) =
                        matches_from_range(input, &tokens[1..], input_index + len, config, captures)
                    {
                        Some(len + rest_len)
                    } else {
                        crate::core::debug_log(
                            config,
                            &format!("BackReference failed: rest didn't match"),
                        );
                        None
                    }
                } else {
                    crate::core::debug_log(
                        config,
                        &format!(
                            "BackReference failed: '{}' doesn't match input at pos {}",
                            captured, input_index
                        ),
                    );
                    None
                }
            } else {
                crate::core::debug_log(
                    config,
                    &format!("BackReference failed: capture {} not found", n),
                );
                None
            }
        }
        _ => {
            if input_index >= input.len() {
                crate::core::debug_log(
                    config,
                    &format!("Token {:?} failed: input_index >= input.len()", token),
                );
                return None;
            }
            if !single_matches(&input[input_index..], token) {
                crate::core::debug_log(
                    config,
                    &format!("Token {:?} failed: no match at pos {}", token, input_index),
                );
                return None;
            }
            crate::core::debug_log(
                config,
                &format!("Token {:?} matched at pos {}", token, input_index),
            );
            if let Some(rest_len) =
                matches_from_range(input, &tokens[1..], input_index + 1, config, captures)
            {
                return Some(1 + rest_len);
            } else {
                crate::core::debug_log(
                    config,
                    &format!("Token {:?} failed: rest didn't match", token),
                );
                return None;
            }
        }
    }
}

pub fn match_tokens(input: &str, tokens: &[Token], config: &Config) -> bool {
    let input_chars: Vec<char> = input.chars().collect();

    // Respect ^ anchor
    let start_positions = if matches!(tokens.first(), Some(Token::StartAnchor)) {
        vec![0]
    } else {
        (0..=input_chars.len()).collect()
    };

    for &start_index in &start_positions {
        let mut captures = Vec::new();
        if matches_from_range(
            &input_chars,
            tokens,
            start_index,
            Some(config),
            &mut captures,
        )
        .is_some()
        {
            return true;
        }
    }

    false
}

pub fn single_matches(input: &[char], token: &Token) -> bool {
    if input.is_empty() {
        return false;
    }

    let ch = input[0];
    match token {
        Token::Digit => ch.is_ascii_digit(),
        Token::Word => ch.is_ascii_alphanumeric() || ch == '_',
        Token::Literal(c) => *c == ch,
        Token::CharGroup(set) => set.contains(&ch),
        Token::NegCharGroup(set) => !set.contains(&ch),
        Token::StartAnchor | Token::EndAnchor => true, // handled in matches_from
        Token::WildCard => ch != '\n',
        Token::Quantifier(_, _) => unreachable!("Quantifier handled in matches_from"),
        Token::Alternation(_) => unreachable!("handled in matches_from"),
        Token::CaptureGroup(_, _) => unreachable!("CaptureGroup handled in matches_from"),
        Token::BackReference(_) => unreachable!("BackReference handled in matches_from"),
        Token::ExactRepetition(_, _) => unreachable!("ExactRepetition handled in matches_from"),
        Token::RangeRepetition(_, _, _) => unreachable!("RangeRepetition handled in matches_from"),
    }
}

/// Collect all possible match lengths for a sequence of tokens starting at input_index.
/// This is used for backtracking in capture groups.
#[allow(unused_assignments)]
fn collect_all_match_lengths(
    input: &[char],
    tokens: &[Token],
    input_index: usize,
    config: Option<&Config>,
    captures: &Vec<Option<String>>,
) -> Vec<usize> {
    if tokens.is_empty() {
        return vec![0];
    }

    let token = &tokens[0];
    let mut result = Vec::new();

    match token {
        Token::StartAnchor => {
            if input_index == 0 {
                return collect_all_match_lengths(input, &tokens[1..], input_index, config, captures);
            }
            return vec![];
        }
        Token::EndAnchor => {
            if input_index == input.len() {
                return collect_all_match_lengths(input, &tokens[1..], input_index, config, captures);
            }
            return vec![];
        }
        Token::Quantifier(inner, quant) => {
            // Collect all positions where the quantifier can stop
            let mut positions = vec![input_index]; // For * and ?, include 0 matches
            if matches!(quant, crate::parser::types::Quantifiers::OneOrMore) {
                positions.clear(); // For +, must have at least 1 match
            }

            let mut current_pos = input_index;
            let mut dummy_captures = Vec::new();
            while let Some(len) = matches_from_range(
                input,
                &[inner.as_ref().clone()],
                current_pos,
                config,
                &mut dummy_captures,
            ) {
                current_pos += len;
                positions.push(current_pos);
                if matches!(quant, crate::parser::types::Quantifiers::ZeroOrOne) && positions.len() > 1 {
                    break; // ? matches at most once
                }
            }

            // For each position where quantifier can stop, collect lengths for remaining tokens
            for pos in positions {
                let rest_lengths = collect_all_match_lengths(input, &tokens[1..], pos, config, captures);
                for rest_len in rest_lengths {
                    result.push((pos - input_index) + rest_len);
                }
            }
        }
        Token::Alternation(branches) => {
            for branch in branches {
                let mut combined = branch.clone();
                combined.extend_from_slice(&tokens[1..]);
                let branch_lengths =
                    collect_all_match_lengths(input, &combined, input_index, config, captures);
                result.extend(branch_lengths);
            }
        }
        Token::CaptureGroup(group_num, inner_tokens) => {
            let group_num = *group_num;
            // Collect all possible lengths for inner tokens
            let inner_lengths = collect_all_match_lengths(input, inner_tokens, input_index, config, captures);
            for inner_len in inner_lengths {
                let mut temp_captures = captures.clone();
                let matched_str: String = input[input_index..input_index + inner_len].iter().collect();
                if group_num > temp_captures.len() {
                    temp_captures.resize(group_num, None);
                }
                temp_captures[group_num - 1] = Some(matched_str);
                // Populate nested captures by running dummy inner match
                let mut dummy_captures = temp_captures.clone();
                let _ = matches_from_range(input, inner_tokens, input_index, config, &mut dummy_captures);
                temp_captures = dummy_captures;
                let rest_lengths = collect_all_match_lengths(input, &tokens[1..], input_index + inner_len, config, &temp_captures);
                for rest_len in rest_lengths {
                    result.push(inner_len + rest_len);
                }
            }
        }
        Token::BackReference(n) => {
            let n = *n;
            if let Some(Some(captured)) = captures.get(n - 1) {
                let captured_chars: Vec<char> = captured.chars().collect();
                if input_index + captured_chars.len() <= input.len() && input[input_index..input_index + captured_chars.len()] == captured_chars {
                    let len = captured_chars.len();
                    let rest_lengths = collect_all_match_lengths(input, &tokens[1..], input_index + len, config, captures);
                    for rest_len in rest_lengths {
                        result.push(len + rest_len);
                    }
                }
            }
        }
        Token::ExactRepetition(inner, count) => {
            let mut current_pos = input_index;
            let mut dummy_captures = Vec::new();
            let mut success = true;
            for _ in 0..*count {
                if let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    &mut dummy_captures,
                ) {
                    current_pos += len;
                } else {
                    success = false;
                    break;
                }
            }
            if success {
                let rest_lengths =
                    collect_all_match_lengths(input, &tokens[1..], current_pos, config, captures);
                for rest_len in rest_lengths {
                    result.push((current_pos - input_index) + rest_len);
                }
            }
        }
        Token::RangeRepetition(inner, min, max) => {
            let mut positions = Vec::new();
            let mut current_pos = input_index;
            let mut count = 0;
            let mut dummy_captures = Vec::new();

            // Match minimum required
            let mut success = true;
            for _ in 0..*min {
                if let Some(len) = matches_from_range(
                    input,
                    &[inner.as_ref().clone()],
                    current_pos,
                    config,
                    &mut dummy_captures,
                ) {
                    current_pos += len;
                    count += 1;
                } else {
                    success = false;
                    break;
                }
            }

            if success {
                positions.push(current_pos);
                // Match additional up to max
                if let Some(max_val) = max {
                    while count < *max_val {
                        if let Some(len) = matches_from_range(
                            input,
                            &[inner.as_ref().clone()],
                            current_pos,
                            config,
                            &mut dummy_captures,
                        ) {
                            current_pos += len;
                            count += 1;
                            positions.push(current_pos);
                        } else {
                            break;
                        }
                    }
                } else {
                    while let Some(len) = matches_from_range(
                        input,
                        &[inner.as_ref().clone()],
                        current_pos,
                        config,
                        &mut dummy_captures,
                    ) {
                        current_pos += len;
                        count += 1;
                        positions.push(current_pos);
                    }
                }

                for pos in positions {
                    let rest_lengths = collect_all_match_lengths(input, &tokens[1..], pos, config, captures);
                    for rest_len in rest_lengths {
                        result.push((pos - input_index) + rest_len);
                    }
                }
            }
        }
        _ => {
            // Simple tokens: Literal, Digit, Word, CharGroup, NegCharGroup, WildCard
            if input_index >= input.len() {
                return vec![];
            }
            if single_matches(&input[input_index..], token) {
                let rest_lengths =
                    collect_all_match_lengths(input, &tokens[1..], input_index + 1, config, captures);
                for rest_len in rest_lengths {
                    result.push(1 + rest_len);
                }
            }
        }
    }

    // Remove duplicates and sort
    result.sort_unstable();
    result.dedup();
    result
}
