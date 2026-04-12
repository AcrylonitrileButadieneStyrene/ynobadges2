use crate::format::output::Condition;

use super::token::Token;

type TokenSpan = (Token, std::ops::Range<usize>);

pub struct Parser {
    tokens: Vec<TokenSpan>,
    condition: Condition,
    state: State,
    position: usize,
}

enum State {
    None,
}

enum Error {
    Expected(&'static str),
}

impl Parser {
    pub fn new(tokens: Vec<TokenSpan>) -> Self {
        Self {
            tokens,
            condition: Condition::default(),
            state: State::None,
            position: 0,
        }
    }

    pub fn eval(mut self) -> Result<Condition, (std::ops::Range<usize>, Error)> {
        self.eval_inner()
            .map(|()| self.condition)
            .map_err(|err| (self.tokens[self.position].1.clone(), err))
    }

    fn eval_inner(&mut self) -> Result<(), Error> {
        while let Some(token) = self.next() {
            match token {
                Token::Map => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    self.condition.map = Some(*id as u16);
                }
                Token::X => {
                    let (x1, x2) = self.equals_range()?;
                    self.condition.map_x1 = Some(x1 as _);
                    self.condition.map_x2 = x2.map(|x| x as _);
                }
                Token::Y => {
                    let (y1, y2) = self.equals_range()?;
                    self.condition.map_y1 = Some(y1 as _);
                    self.condition.map_y2 = y2.map(|x| x as _);
                }
                Token::Switch => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    let id = *id as u16;

                    self.expect_equals();

                    let value = match self.next() {
                        Some(Token::On) => true,
                        Some(Token::Off) => false,
                        _ => return Err(Error::Expected("boolean")),
                    };

                    if let (Some(switch_ids), Some(switch_values)) = (
                        &mut self.condition.switch_ids,
                        &mut self.condition.switch_values,
                    ) {
                        switch_ids.push(id);
                        switch_values.push(value);
                    } else if let Some(existing) = self.condition.switch_id {
                        self.condition.switch_ids = Some(vec![existing, id]);
                        self.condition.switch_values =
                            Some(vec![self.condition.switch_value, value]);
                        self.condition.switch_id = None;
                        self.condition.switch_value = false;
                    } else {
                        self.condition.switch_id = Some(id);
                        self.condition.switch_value = value;
                    }
                }
                Token::Variable => todo!(),
                Token::Event => todo!(),
                _ => return Err(Error::Expected("start of instruction")),
            }
        }

        Ok(())
    }

    fn equals_range(&mut self) -> Result<(i32, Option<i32>), Error> {
        self.expect_equals();

        let Some(Token::Number(x1)) = self.next() else {
            return Err(Error::Expected("number"));
        };

        let x1 = *x1;

        if matches!(self.peek(), Some(Token::Colon)) {
            self.position += 1;

            let Some(Token::Number(x2)) = self.next() else {
                return Err(Error::Expected("number"));
            };

            Ok((x1, Some(*x2)))
        } else {
            Ok((x1, None))
        }
    }

    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token.map(|(token, _)| token)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|(token, _)| token)
    }

    fn expect_equals(&mut self) -> Result<(), Error> {
        matches!(self.next(), Some(Token::Equals)).ok_or(Error::Expected("equals"))
    }
}
