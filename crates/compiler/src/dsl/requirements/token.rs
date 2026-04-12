#[derive(logos::Logos)]
#[logos(skip r"[ \t\n]+")]
pub enum Token {
    #[token("AND")]
    And,

    #[token("OR")]
    Or,

    #[regex("[a-z0-9_]+", |lex| lex.slice().to_string())]
    ID(String),
}
