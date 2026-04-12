use logos::Logos as _;

mod parser;
mod token;

#[cfg(test)]
mod tests;

pub fn parse(input: &str) -> crate::format::output::Condition {
    let tokens = token::Token::lexer(input)
        .spanned()
        .map(|(token, span)| token.map(|token| (token, span)))
        .try_collect::<Vec<_>>()
        .unwrap();

    let parser = parser::Parser::new(tokens).eval();

    todo!()
}
