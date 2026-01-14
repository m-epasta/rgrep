#[derive(Debug, Clone)]
pub enum Token {
    Digit,                                             // \d
    Word,                                              // \w
    Literal(char),                                     // any literal character
    CharGroup(Vec<char>),                              // [abc]
    NegCharGroup(Vec<char>),                           // [^abc]
    StartAnchor,                                       // ^log
    EndAnchor,                                         // log$
    Quantifier(Box<Token>, Quantifiers),               // pig+ || pig? || pig*
    WildCard,                                          // p.g
    Alternation(Vec<Vec<Token>>),                      // dog|pig
    CaptureGroup(usize, Vec<Token>),                   // (group_number, captured content)
    BackReference(usize),                              // \1, \2, etc.
    ExactRepetition(Box<Token>, usize),                // a{3}
    RangeRepetition(Box<Token>, usize, Option<usize>), // a{2,} or a{2,4}
}

#[derive(Debug, Clone)]
pub enum Quantifiers {
    OneOrMore,  // +
    ZeroOrOne,  // ?
    ZeroOrMore, // *
}
