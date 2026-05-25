use crate::format::output::{Condition, ConditionTrigger};

use super::token::Token;

type TokenSpan = (Token, std::ops::Range<usize>);

pub struct Parser {
    pub tokens: Vec<TokenSpan>,
    pub state: State,
    pub condition: Condition,
    pub position: usize,
}

pub struct State {
    pub last_delayable: Option<DelayTarget>,
    pub has_trigger: bool,
}

pub enum DelayTarget {
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
                has_trigger: false,
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
                    self.expect_equals()?;
                    let (x1, x2) = self.range()?;
                    self.condition.map_x1 = x1 as _;
                    self.condition.map_x2 = x2.unwrap_or_default() as _;
                    if self.condition.trigger.is_none() && !self.state.has_trigger {
                        self.condition.trigger =
                            Some(crate::format::output::ConditionTrigger::Coords);
                    }
                }
                Token::Y => {
                    self.expect_equals()?;
                    let (y1, y2) = self.range()?;
                    self.condition.map_y1 = y1 as _;
                    self.condition.map_y2 = y2.unwrap_or_default() as _;
                    if self.condition.trigger.is_none() && !self.state.has_trigger {
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

                    super::transformers::push_switch(self, id, value);
                }
                Token::Variable => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    let id = *id as u16;

                    let op = match self.next() {
                        Some(Token::Eq) => "=",
                        Some(Token::Ge) => ">=",
                        Some(Token::Le) => "<=",
                        Some(Token::Lt) => "<",
                        Some(Token::Gt) => ">",
                        Some(Token::Ne) => "!=",
                        _ => return Err(Error::Expected("comparison")),
                    }
                    .to_string();

                    let (value1, value2) = if op == "=" {
                        self.range()?
                    } else {
                        let Some(Token::Number(value)) = self.next() else {
                            return Err(Error::Expected("number"));
                        };

                        (*value, None)
                    };

                    if let Some(value2) = value2 {
                        self.condition.var_value = Some(value1);
                        self.condition.var_op = Some(">=<".to_string());
                        self.condition.var_value2 = Some(value2);
                    } else {
                        super::transformers::push_variable(self, id, op, value1);
                    }
                }
                Token::Event => {
                    let Some(Token::Number(id)) = self.next() else {
                        return Err(Error::Expected("number"));
                    };

                    self.condition.value = Some(id.to_string());
                    self.condition.trigger = Some(ConditionTrigger::EventAction);
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
                Token::Indirect => {
                    if matches!(self.condition.trigger, Some(ConditionTrigger::EventAction)) {
                        self.condition.trigger = Some(ConditionTrigger::Event);
                    } else {
                        return Err(Error::Expected("cannot make non-event action indirect"));
                    }
                }
                Token::Picture => {
                    self.expect_equals()?;
                    let Some(Token::String(string)) = self.next() else {
                        return Err(Error::Expected("string"));
                    };

                    // todo: throw an error if the value/trigger were already
                    //       set or overwrite trigger: bounds (above too).
                    self.condition.value = Some(string.clone());
                    self.condition.trigger = Some(ConditionTrigger::Picture);
                }
                _ => return Err(Error::Expected("start of instruction")),
            }
        }

        Ok(())
    }

    fn range(&mut self) -> Result<(i32, Option<i32>), Error> {
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
        matches!(self.next(), Some(Token::Eq)).ok_or(Error::Expected("equals"))
    }
}
