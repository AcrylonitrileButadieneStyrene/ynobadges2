use logos::Logos as _;

mod parser;
mod token;

#[cfg(test)]
mod tests;

#[must_use]
pub fn parse(source: &str, input: &str) -> Option<crate::format::output::Condition> {
    let tokens = token::Token::lexer(input)
        .spanned()
        .map(|(token, span)| token.map(|token| (token, span.clone())).map_err(|()| span))
        .try_collect::<Vec<_>>()
        .inspect_err(|span| {
            ariadne::Report::build(ariadne::ReportKind::Error, (source, span.clone()))
                .with_message("unrecognized token")
                .with_label(
                    ariadne::Label::new((source, span.clone()))
                        .with_message("Does not match any known tokens"),
                )
                .finish()
                .eprint((source, ariadne::Source::from(input)))
                .unwrap();
        })
        .ok()?;

    parser::Parser::new(tokens)
        .eval()
        .inspect_err(|(span, err)| {
            ariadne::Report::build(ariadne::ReportKind::Error, span.clone())
                .with_label(ariadne::Label::new(span.clone()).with_message(match err {
                    parser::Error::Expected(str) => format!("Expected {str}"),
                }))
                .with_message(match err {
                    parser::Error::Expected(_) => format!("Syntax error"),
                })
                .finish()
                .eprint(ariadne::Source::from(input))
                .unwrap();
        })
        .ok()
}
