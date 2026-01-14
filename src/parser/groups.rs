use crate::parser::types::Token;

pub fn assign_group_numbers(tokens: &mut [Token], group_counter: &mut usize) {
    for token in tokens.iter_mut() {
        match token {
            Token::CaptureGroup(group_num, inner_tokens) => {
                *group_num = *group_counter;
                *group_counter += 1;
                assign_group_numbers(inner_tokens, group_counter);
            }
            Token::Alternation(branches) => {
                // For alternation, we need to handle the case where different branches
                // might have different numbers of capture groups. This is complex.
                // For now, let's assign numbers based on the first branch and hope
                // all branches have the same structure (which they should in valid regex).
                if let Some(first_branch) = branches.first_mut() {
                    assign_group_numbers(first_branch, group_counter);
                }
            }
            Token::Quantifier(inner, _) => {
                if let Token::CaptureGroup(group_num, inner_tokens) = inner.as_mut() {
                    *group_num = *group_counter;
                    *group_counter += 1;
                    assign_group_numbers(inner_tokens, group_counter);
                } else {
                    assign_group_numbers(std::slice::from_mut(inner.as_mut()), group_counter);
                }
            }
            Token::ExactRepetition(inner, _) | Token::RangeRepetition(inner, _, _) => {
                if let Token::CaptureGroup(group_num, inner_tokens) = inner.as_mut() {
                    *group_num = *group_counter;
                    *group_counter += 1;
                    assign_group_numbers(inner_tokens, group_counter);
                } else {
                    assign_group_numbers(std::slice::from_mut(inner.as_mut()), group_counter);
                }
            }
            _ => {}
        }
    }
}
