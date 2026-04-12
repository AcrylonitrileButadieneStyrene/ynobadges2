use logos::Logos as _;

mod parser;
mod token;

#[cfg(test)]
mod tests;

pub fn parse(input: &str) -> Option<crate::format::output::Condition> {
    let tokens = token::Token::lexer(input)
        .spanned()
        .map(|(token, span)| token.map(|token| (token, span.clone())).map_err(|()| span))
        .try_collect::<Vec<_>>()
        .inspect_err(|span| {
            ariadne::Report::build(ariadne::ReportKind::Error, span.clone())
                .with_message("unrecognized token")
                .finish()
                .eprint(ariadne::Source::from(input))
                .unwrap();
        })
        .ok()?;

    parser::Parser::new(tokens)
        .eval()
        .inspect_err(|(span, err)| {
            ariadne::Report::build(ariadne::ReportKind::Error, span.clone())
                .with_message(match err {
                    parser::Error::Expected(str) => format!("Expected {str}"),
                })
                .finish()
                .eprint(ariadne::Source::from(input))
                .unwrap();
        })
        .ok()
}
