// Re-export everything from the new modular structure to maintain backward compatibility

pub use crate::parser::types::Token;
pub use crate::parser::tokenize::tokenize;
pub use crate::parser::groups::assign_group_numbers;
pub use crate::parser::matcher::{matches_from_range, match_tokens};
