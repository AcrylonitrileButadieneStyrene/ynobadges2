#[derive(logos::Logos)]
#[logos(skip r"[ \t\n]+")]
enum Token {
    #[token("ALL")]
    All,

    #[token("AND")]
    And,

    #[token("OR")]
    Or,

    #[token("(")]
    GroupStart,

    #[token(")")]
    GroupEnd,

    #[regex("[a-zA-Z_]+")]
    ID,
}
