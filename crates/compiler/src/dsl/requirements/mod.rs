use logos::Logos as _;

mod token;

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    All,
    Tag(String),
    Tags(Vec<String>),
    TagArray(Vec<Vec<String>>),
}

enum State {
    Default,
    Split,
    Merge,
}

#[must_use]
pub fn parse(input: &str) -> Option<Request> {
    token::Token::lexer(input)
        .spanned()
        .map(|(token, span)| token.map(|token| (token, span.clone())).map_err(|()| span))
        .try_collect::<Vec<_>>()
        .inspect_err(|span| {
            ariadne::Report::build(ariadne::ReportKind::Error, span.clone())
                .with_message("unrecognized token")
                .with_label(
                    ariadne::Label::new(span.clone())
                        .with_message("Does not match any known tokens"),
                )
                .finish()
                .eprint(ariadne::Source::from(input))
                .unwrap();
        })
        .ok()?
        .into_iter()
        .fold(
            (Some(Request::All), State::Default),
            |(acc, state), (token, span)| match acc {
                None => (None, state),
                x if matches!(token, token::Token::And) && matches!(state, State::Default) => {
                    (x, State::Merge)
                }
                x if matches!(token, token::Token::Or) && matches!(state, State::Default) => {
                    (x, State::Split)
                }
                Some(Request::All)
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Default) =>
                {
                    (Some(Request::Tag(id.clone())), State::Default)
                }
                Some(Request::Tag(existing))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Merge) =>
                {
                    (
                        Some(Request::Tags(vec![existing, id.clone()])),
                        State::Default,
                    )
                }
                Some(Request::Tag(existing))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Split) =>
                {
                    (
                        Some(Request::TagArray(vec![vec![existing, id.clone()]])),
                        State::Default,
                    )
                }
                Some(Request::Tags(mut array))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Merge) =>
                {
                    array.push(id.clone());
                    (Some(Request::Tags(array)), State::Default)
                }
                Some(Request::Tags(mut array))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Split) =>
                {
                    let last = array.pop().unwrap();
                    (
                        Some(Request::TagArray(vec![array, vec![last, id.clone()]])),
                        State::Default,
                    )
                }
                Some(Request::TagArray(mut array))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Merge) =>
                {
                    array.push(vec![id.clone()]);
                    (Some(Request::TagArray(array)), State::Default)
                }
                Some(Request::TagArray(mut array))
                    if let token::Token::ID(ref id) = token
                        && matches!(state, State::Split) =>
                {
                    array.last_mut().unwrap().push(id.clone());
                    (Some(Request::TagArray(array)), State::Default)
                }
                _ => {
                    ariadne::Report::build(ariadne::ReportKind::Error, span.clone())
                        .with_message("unrecognized token")
                        .with_label(
                            ariadne::Label::new(span)
                                .with_message("Does not match any known tokens"),
                        )
                        .finish()
                        .eprint(ariadne::Source::from(input))
                        .unwrap();
                    (None, state)
                }
            },
        )
        .0
}

#[cfg(test)]
mod tests {
    #[test]
    fn none() {
        assert_eq!(super::parse(""), Some(super::Request::All))
    }

    #[test]
    fn single() {
        assert_eq!(
            super::parse("test"),
            Some(super::Request::Tag("test".to_string()))
        )
    }

    #[test]
    fn list() {
        assert_eq!(
            super::parse("test1 AND test2"),
            Some(super::Request::Tags(vec![
                "test1".to_string(),
                "test2".to_string()
            ]))
        )
    }

    #[test]
    fn complex_1() {
        assert_eq!(
            super::parse("test1 AND test2 OR test3"),
            Some(super::Request::TagArray(vec![
                vec!["test1".to_string()],
                vec!["test2".to_string(), "test3".to_string()],
            ]))
        )
    }

    #[test]
    fn complex_2() {
        assert_eq!(
            super::parse("test1 OR test2 AND test3"),
            Some(super::Request::TagArray(vec![
                vec!["test1".to_string(), "test2".to_string()],
                vec!["test3".to_string()],
            ]))
        )
    }

    #[test]
    fn complex_3() {
        assert_eq!(
            super::parse("test1 AND test2 OR test3 OR test4 AND test5 "),
            Some(super::Request::TagArray(vec![
                vec!["test1".to_string()],
                vec![
                    "test2".to_string(),
                    "test3".to_string(),
                    "test4".to_string()
                ],
                vec!["test5".to_string()],
            ]))
        )
    }
}
