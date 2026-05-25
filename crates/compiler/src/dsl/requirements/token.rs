#[derive(logos::Logos)]
#[logos(skip r"[ \t\n]+")]
pub enum Token {
    #[token("AND")]
    And,

    #[token("OR")]
    Or,

    #[token("COUNT=")]
    Count,

    #[regex("[a-z0-9_]+", |lex| lex.slice().to_string())]
    ID(String),

    #[regex("[0-9]+", |lex| lex.slice().parse::<u8>().unwrap(), priority = 3)]
    Number(u8),
}
