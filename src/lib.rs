//! Check digit validation for ISBN-10 and ISBN-13.
//!
//! ISBN-10 uses a mod-11 checksum where the tenth digit may be the
//! letter `X`, standing in for the value 10 (mod 11 needs eleven
//! possible remainders, and a single decimal digit can't cover them).
//!
//! ISBN-13 dropped that oddity by switching to the same mod-10 scheme
//! used by EAN-13 and UPC-A barcodes, alternating weights of 1 and 3.

/// Which kind of code a caller's input turned out to be, once cleaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsbnKind {
    Isbn10,
    Isbn13,
}

impl std::fmt::Display for IsbnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsbnKind::Isbn10 => write!(f, "ISBN-10"),
            IsbnKind::Isbn13 => write!(f, "ISBN-13"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsbnError {
    /// Length after stripping hyphens and spaces didn't match 10 or 13.
    WrongLength(usize),
    /// A character that isn't a digit (or a trailing X for ISBN-10).
    InvalidChar(char),
    /// Length and characters were fine, but the checksum didn't work out.
    BadCheckDigit { expected: char, found: char },
}

impl std::fmt::Display for IsbnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsbnError::WrongLength(n) => {
                write!(f, "expected 10 or 13 digits, got {n}")
            }
            IsbnError::InvalidChar(c) => write!(f, "unexpected character '{c}'"),
            IsbnError::BadCheckDigit { expected, found } => {
                write!(f, "check digit should be '{expected}', found '{found}'")
            }
        }
    }
}

impl std::error::Error for IsbnError {}

fn clean(input: &str) -> String {
    input.chars().filter(|c| *c != '-' && *c != ' ').collect()
}

/// Validate an ISBN-10 string. Hyphens and spaces are ignored; the
/// final check digit may be `X` or `x`.
pub fn check_isbn10(input: &str) -> Result<(), IsbnError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() != 10 {
        return Err(IsbnError::WrongLength(chars.len()));
    }

    let mut sum = 0u32;
    for (i, c) in chars.iter().enumerate() {
        let value = if i == 9 && (*c == 'X' || *c == 'x') {
            10
        } else if let Some(d) = c.to_digit(10) {
            d
        } else {
            return Err(IsbnError::InvalidChar(*c));
        };
        sum += value * (10 - i as u32);
    }

    if sum % 11 == 0 {
        Ok(())
    } else {
        let expected = isbn10_check_digit(&chars[..9]);
        Err(IsbnError::BadCheckDigit {
            expected,
            found: chars[9],
        })
    }
}

/// Validate an ISBN-13 string. Hyphens and spaces are ignored.
pub fn check_isbn13(input: &str) -> Result<(), IsbnError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() != 13 {
        return Err(IsbnError::WrongLength(chars.len()));
    }

    let mut digits = Vec::with_capacity(13);
    for c in &chars {
        match c.to_digit(10) {
            Some(d) => digits.push(d),
            None => return Err(IsbnError::InvalidChar(*c)),
        }
    }

    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d } else { *d * 3 })
        .sum();

    if sum % 10 == 0 {
        Ok(())
    } else {
        let expected = isbn13_check_digit(&digits[..12]);
        Err(IsbnError::BadCheckDigit {
            expected: std::char::from_digit(expected, 10).unwrap(),
            found: chars[12],
        })
    }
}

/// Clean the input and dispatch to the ISBN-10 or ISBN-13 checker based
/// on length. Returns which kind was checked so callers can report it.
pub fn check(input: &str) -> Result<IsbnKind, IsbnError> {
    match clean(input).chars().count() {
        10 => check_isbn10(input).map(|_| IsbnKind::Isbn10),
        13 => check_isbn13(input).map(|_| IsbnKind::Isbn13),
        n => Err(IsbnError::WrongLength(n)),
    }
}

/// Compute the check digit for the first nine digits of an ISBN-10.
/// `digits` must contain exactly nine ASCII digit characters.
fn isbn10_check_digit(digits: &[char]) -> char {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, c)| c.to_digit(10).unwrap() * (10 - i as u32))
        .sum();
    let check = (11 - sum % 11) % 11;
    if check == 10 {
        'X'
    } else {
        std::char::from_digit(check, 10).unwrap()
    }
}

/// Compute the check digit for the first twelve digits of an ISBN-13.
fn isbn13_check_digit(digits: &[u32]) -> u32 {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d } else { *d * 3 })
        .sum();
    (10 - sum % 10) % 10
}
