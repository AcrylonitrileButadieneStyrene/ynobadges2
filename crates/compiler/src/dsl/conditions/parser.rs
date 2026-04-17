use crate::format::output::Condition;

use super::token::Token;

type TokenSpan = (Token, std::ops::Range<usize>);

pub struct Parser {
    tokens: Vec<TokenSpan>,
    state: State,
    condition: Condition,
    position: usize,
}

struct State {
    last_delayable: Option<DelayTarget>,
}

enum DelayTarget {
    Switch,
    Variable,
}

#[derive(Debug)]
pub enum Error {
    Expected(&'static str),
}

impl Parser {
    pub fn new(tokens: Vec<TokenSpan>) -> Self {
        Self {
            tokens,
            state: State {
                last_delayable: None,
            },
            condition: Condition::default(),
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
                    if self.condition.trigger.is_none() {
                        self.condition.trigger =
                            Some(crate::format::output::ConditionTrigger::Coords);
                    }
                }
                Token::Y => {
                    let (y1, y2) = self.equals_range()?;
                    self.condition.map_y1 = Some(y1 as _);
                    self.condition.map_y2 = y2.map(|x| x as _);
                    if self.condition.trigger.is_none() {
                        self.condition.trigger =
                            Some(crate::format::output::ConditionTrigger::Coords);
                    }
                }
                Token::Switch => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    let id = *id as u16;

                    self.expect_equals()?;

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

                    self.state.last_delayable = Some(DelayTarget::Switch);
                }
                Token::Variable => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    let id = *id as u16;

                    // todo: more ops
                    self.expect_equals()?;

                    let Some(Token::Number(value)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    let value = *value;

                    if let (Some(var_ids), Some(var_values)) =
                        (&mut self.condition.var_ids, &mut self.condition.var_values)
                    {
                        var_ids.push(id);
                        var_values.push(value);
                    } else if let (Some(existing_id), Some(existing_value)) =
                        (self.condition.var_id, self.condition.var_value)
                    {
                        self.condition.var_ids = Some(vec![existing_id, id]);
                        self.condition.var_values = Some(vec![existing_value, value]);
                        self.condition.var_id = None;
                        self.condition.var_value = None;
                    } else {
                        self.condition.var_id = Some(id);
                        self.condition.var_value = Some(value);
                    }

                    self.state.last_delayable = Some(DelayTarget::Variable);
                }
                Token::Event => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    self.condition.value = Some(id.to_string());
                    self.condition.trigger =
                        Some(crate::format::output::ConditionTrigger::EventAction);
                }
                Token::Delayed => match self.state.last_delayable {
                    Some(DelayTarget::Switch) => {
                        self.condition.switch_delay = true;
                    }
                    Some(DelayTarget::Variable) => {
                        self.condition.var_delay = true;
                    }
                    None => return Err(Error::Expected("a switch or variable earlier")),
                },
                _ => return Err(Error::Expected("start of instruction")),
            }
        }

        Ok(())
    }

    fn equals_range(&mut self) -> Result<(i32, Option<i32>), Error> {
        self.expect_equals()?;

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
