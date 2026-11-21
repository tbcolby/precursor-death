//! Birth/survival rules for Life-like automata and the `B3/S23` parser.

use std::fmt;

/// A Life-like rule: bit `n` of `birth` / `survive` says what happens with
/// `n` live neighbours (0..=8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    birth: u16,
    survive: u16,
}

/// Why a rule string was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    /// Must start with `B`.
    MissingBirth,
    /// Needs a `/` between the birth and survival parts.
    MissingSlash,
    /// The part after `/` must start with `S`.
    MissingSurvival,
    /// A character other than a digit 0-8 where digits are expected.
    BadChar(char),
    /// A digit that is not a possible neighbour count (9 is a digit but never a valid count).
    DigitTooLarge(char),
    /// The same count listed twice.
    Duplicate(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty rule"),
            ParseError::MissingBirth => write!(f, "rule must start with B"),
            ParseError::MissingSlash => write!(f, "missing / between B and S"),
            ParseError::MissingSurvival => write!(f, "survival part must start with S"),
            ParseError::BadChar(c) => write!(f, "unexpected '{c}'"),
            ParseError::DigitTooLarge(c) => write!(f, "{c} is more than 8 neighbours"),
            ParseError::Duplicate(c) => write!(f, "{c} listed twice"),
        }
    }
}

impl Rule {
    /// Build from neighbour-count lists; counts above 8 are an error rather than ignored.
    pub fn new(birth: &[u8], survive: &[u8]) -> Result<Rule, ParseError> {
        let mask = |counts: &[u8]| -> Result<u16, ParseError> {
            let mut m = 0u16;
            for &n in counts {
                if n > 8 {
                    return Err(ParseError::DigitTooLarge((b'0' + n.min(9)) as char));
                }
                m |= 1 << n;
            }
            Ok(m)
        };
        Ok(Rule { birth: mask(birth)?, survive: mask(survive)? })
    }

    /// Parse `B3/S23` notation: `B`, digits 0-8, `/`, `S`, digits 0-8. Case
    /// insensitive; surrounding whitespace is ignored; either digit list may
    /// be empty (`B2/S` is Seeds). Rejects anything else, including a
    /// repeated digit.
    pub fn parse(text: &str) -> Result<Rule, ParseError> {
        let t = text.trim();
        if t.is_empty() {
            return Err(ParseError::Empty);
        }
        let mut chars = t.chars();
        if !matches!(chars.next(), Some('B' | 'b')) {
            return Err(ParseError::MissingBirth);
        }
        let rest = chars.as_str();
        let slash = rest.find('/').ok_or(ParseError::MissingSlash)?;
        let (b, s) = (&rest[..slash], &rest[slash + 1..]);
        let mut sc = s.chars();
        if !matches!(sc.next(), Some('S' | 's')) {
            return Err(ParseError::MissingSurvival);
        }
        Ok(Rule { birth: digits(b)?, survive: digits(sc.as_str())? })
    }

    pub fn born(&self, neighbours: u8) -> bool { neighbours <= 8 && self.birth >> neighbours & 1 == 1 }

    pub fn survives(&self, neighbours: u8) -> bool { neighbours <= 8 && self.survive >> neighbours & 1 == 1 }

    /// The next state of a cell with `neighbours` live neighbours.
    pub fn next(&self, alive: bool, neighbours: u8) -> bool {
        if alive {
            self.survives(neighbours)
        } else {
            self.born(neighbours)
        }
    }

    pub fn toggle_birth(&mut self, n: u8) {
        if n <= 8 {
            self.birth ^= 1 << n;
        }
    }

    pub fn toggle_survive(&mut self, n: u8) {
        if n <= 8 {
            self.survive ^= 1 << n;
        }
    }

    /// Canonical notation, digits ascending (`B3/S23`).
    pub fn notation(&self) -> String {
        let list = |m: u16| (0..=8u8).filter(|n| m >> n & 1 == 1).map(|n| (b'0' + n) as char).collect::<String>();
        format!("B{}/S{}", list(self.birth), list(self.survive))
    }
}

fn digits(s: &str) -> Result<u16, ParseError> {
    let mut m = 0u16;
    for c in s.chars() {
        let n = match c {
            '0'..='8' => c as u16 - '0' as u16,
            '9' => return Err(ParseError::DigitTooLarge(c)),
            _ => return Err(ParseError::BadChar(c)),
        };
        if m >> n & 1 == 1 {
            return Err(ParseError::Duplicate(c));
        }
        m |= 1 << n;
    }
    Ok(m)
}
