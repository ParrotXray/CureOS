use alloc::string::String;
use alloc::vec::Vec;
use crate::kprint;

#[derive(Debug)]
pub enum CalcError {
    UnexpectedChar(char),
    UnexpectedEnd,
    DivisionByZero,
    InvalidNumber,
}

pub fn eval_expression(expr: &str) -> Result<f64, CalcError> {
    let mut parser = Parser::new(expr);
    let result = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.pos < parser.chars.len() {
        Err(CalcError::UnexpectedChar(parser.chars[parser.pos]))
    } else {
        Ok(result)
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    _expr: &'a str,
}

impl<'a> Parser<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            chars: expr.chars().collect(),
            pos: 0,
            _expr: expr,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).cloned();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }
    fn parse_number(&mut self) -> Result<f64, CalcError> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(CalcError::InvalidNumber);
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().map_err(|_| CalcError::InvalidNumber)
    }

    fn parse_factor(&mut self) -> Result<f64, CalcError> {
        self.skip_whitespace();
        match self.peek() {
            Some('(') => {
                self.next();
                let val = self.parse_expr()?;
                self.skip_whitespace();
                if self.next() != Some(')') {
                    return Err(CalcError::UnexpectedEnd);
                }
                Ok(val)
            }
            Some('-') => {
                self.next();
                Ok(-self.parse_factor()?)
            }
            _ => self.parse_number(),
        }
    }

    fn parse_term(&mut self) -> Result<f64, CalcError> {
        let mut val = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.next();
                    val *= self.parse_factor()?;
                }
                Some('/') => {
                    self.next();
                    let rhs = self.parse_factor()?;
                    if rhs == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    val /= rhs;
                }
                _ => break,
            }
        }
        Ok(val)
    }

    fn parse_expr(&mut self) -> Result<f64, CalcError> {
        let mut val = self.parse_term()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.next();
                    val += self.parse_term()?;
                }
                Some('-') => {
                    self.next();
                    val -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(val)
    }
}
