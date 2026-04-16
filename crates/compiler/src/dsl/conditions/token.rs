#[derive(logos::Logos)]
#[logos(skip r"[ \t\n]+")]
pub enum Token {
    #[token("M")]
    Map,

    #[token("X")]
    X,

    #[token("Y")]
    Y,

    #[token("S")]
    Switch,

    #[token("V")]
    Variable,

    #[token("E")]
    Event,

    #[token("=")]
    Equals,

    #[token(":")]
    Colon,

    #[regex("-?[0-9]+", |lex| lex.slice().parse::<i32>().unwrap())]
    Number(i32),

    #[token("ON")]
    On,

    #[token("OFF")]
    Off,

    #[token("DELAYED")]
    Delayed,
}
