//! Check digit validation for ISBN-10, ISBN-13, UPC-A, and EAN-13.
//!
//! ISBN-10 uses a mod-11 checksum where the tenth digit may be the
//! letter `X`, standing in for the value 10 (mod 11 needs eleven
//! possible remainders, and a single decimal digit can't cover them).
//!
//! ISBN-13 dropped that oddity by switching to the same mod-10 scheme
//! used by EAN-13 barcodes generally: alternating weights of 1 and 3.
//! Every ISBN-13 is an EAN-13 with a 978 or 979 "Bookland" prefix, so
//! [`check_isbn13`] and [`check_ean13`] share their math; [`check`] tells
//! them apart by that prefix.
//!
//! UPC-A is the same idea with different weights: alternating 3 and 1,
//! starting from the first digit instead of the second.

/// Which kind of code a caller's input turned out to be, once cleaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeKind {
    Isbn10,
    Isbn13,
    UpcA,
    Ean13,
}

impl std::fmt::Display for BarcodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarcodeKind::Isbn10 => write!(f, "ISBN-10"),
            BarcodeKind::Isbn13 => write!(f, "ISBN-13"),
            BarcodeKind::UpcA => write!(f, "UPC-A"),
            BarcodeKind::Ean13 => write!(f, "EAN-13"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckError {
    /// Length after stripping hyphens and spaces didn't match 10, 12, or 13.
    WrongLength(usize),
    /// A character that isn't a digit (or a trailing X for ISBN-10).
    InvalidChar(char),
    /// Length and characters were fine, but the checksum didn't work out.
    BadCheckDigit { expected: char, found: char },
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::WrongLength(n) => {
                write!(f, "unexpected length: {n} characters after removing separators")
            }
            CheckError::InvalidChar(c) => write!(f, "unexpected character '{c}'"),
            CheckError::BadCheckDigit { expected, found } => {
                write!(f, "check digit should be '{expected}', found '{found}'")
            }
        }
    }
}

impl std::error::Error for CheckError {}

fn clean(input: &str) -> String {
    input.chars().filter(|c| *c != '-' && *c != ' ').collect()
}

/// Validate an ISBN-10 string. Hyphens and spaces are ignored; the
/// final check digit may be `X` or `x`.
pub fn check_isbn10(input: &str) -> Result<(), CheckError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() != 10 {
        return Err(CheckError::WrongLength(chars.len()));
    }

    let mut sum = 0u32;
    for (i, c) in chars.iter().enumerate() {
        let value = if i == 9 && (*c == 'X' || *c == 'x') {
            10
        } else if let Some(d) = c.to_digit(10) {
            d
        } else {
            return Err(CheckError::InvalidChar(*c));
        };
        sum += value * (10 - i as u32);
    }

    if sum % 11 == 0 {
        Ok(())
    } else {
        let expected = isbn10_check_digit(&chars[..9]);
        Err(CheckError::BadCheckDigit {
            expected,
            found: chars[9],
        })
    }
}

/// Validate an ISBN-13 string. Hyphens and spaces are ignored.
pub fn check_isbn13(input: &str) -> Result<(), CheckError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() != 13 {
        return Err(CheckError::WrongLength(chars.len()));
    }

    let mut digits = Vec::with_capacity(13);
    for c in &chars {
        match c.to_digit(10) {
            Some(d) => digits.push(d),
            None => return Err(CheckError::InvalidChar(*c)),
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
        Err(CheckError::BadCheckDigit {
            expected: std::char::from_digit(expected, 10).unwrap(),
            found: chars[12],
        })
    }
}

/// Validate a general EAN-13 barcode. The checksum is identical to
/// ISBN-13's; the two only differ in what the digits mean, not how
/// they're verified.
pub fn check_ean13(input: &str) -> Result<(), CheckError> {
    check_isbn13(input)
}

/// Validate a UPC-A barcode. Hyphens and spaces are ignored. Same
/// mod-10 idea as ISBN-13/EAN-13, but weights alternate 3 and 1
/// starting from the first digit instead of the second.
pub fn check_upca(input: &str) -> Result<(), CheckError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();
    if chars.len() != 12 {
        return Err(CheckError::WrongLength(chars.len()));
    }

    let mut digits = Vec::with_capacity(12);
    for c in &chars {
        match c.to_digit(10) {
            Some(d) => digits.push(d),
            None => return Err(CheckError::InvalidChar(*c)),
        }
    }

    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d * 3 } else { *d })
        .sum();

    if sum % 10 == 0 {
        Ok(())
    } else {
        let expected = upca_check_digit(&digits[..11]);
        Err(CheckError::BadCheckDigit {
            expected: std::char::from_digit(expected, 10).unwrap(),
            found: chars[11],
        })
    }
}

/// Clean the input and dispatch to the right checker based on length:
/// 10 digits for ISBN-10, 12 for UPC-A, 13 for ISBN-13/EAN-13 (told
/// apart by the 978/979 Bookland prefix). Returns which kind was
/// checked so callers can report it.
pub fn check(input: &str) -> Result<BarcodeKind, CheckError> {
    let cleaned = clean(input);
    match cleaned.chars().count() {
        10 => check_isbn10(input).map(|_| BarcodeKind::Isbn10),
        12 => check_upca(input).map(|_| BarcodeKind::UpcA),
        13 => check_isbn13(input).map(|_| {
            if cleaned.starts_with("978") || cleaned.starts_with("979") {
                BarcodeKind::Isbn13
            } else {
                BarcodeKind::Ean13
            }
        }),
        n => Err(CheckError::WrongLength(n)),
    }
}

/// Compute the missing check digit for a partial code and return the
/// completed code along with which kind it turned out to be. `input`
/// is the code *without* its check digit: 9 digits for ISBN-10, 11
/// for UPC-A, or 12 for ISBN-13/EAN-13. Hyphens and spaces are
/// ignored, same as the `check_*` functions.
pub fn compute(input: &str) -> Result<(BarcodeKind, String), CheckError> {
    let cleaned = clean(input);
    let chars: Vec<char> = cleaned.chars().collect();

    match chars.len() {
        9 => {
            for c in &chars {
                if c.to_digit(10).is_none() {
                    return Err(CheckError::InvalidChar(*c));
                }
            }
            let check = isbn10_check_digit(&chars);
            Ok((BarcodeKind::Isbn10, format!("{cleaned}{check}")))
        }
        11 => {
            let digits = to_digits(&chars)?;
            let check = upca_check_digit(&digits);
            Ok((BarcodeKind::UpcA, format!("{cleaned}{check}")))
        }
        12 => {
            let digits = to_digits(&chars)?;
            let check = isbn13_check_digit(&digits);
            let kind = if cleaned.starts_with("978") || cleaned.starts_with("979") {
                BarcodeKind::Isbn13
            } else {
                BarcodeKind::Ean13
            };
            Ok((kind, format!("{cleaned}{check}")))
        }
        n => Err(CheckError::WrongLength(n)),
    }
}

fn to_digits(chars: &[char]) -> Result<Vec<u32>, CheckError> {
    chars
        .iter()
        .map(|c| c.to_digit(10).ok_or(CheckError::InvalidChar(*c)))
        .collect()
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

/// Compute the check digit for the first twelve digits of an ISBN-13
/// or EAN-13.
fn isbn13_check_digit(digits: &[u32]) -> u32 {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d } else { *d * 3 })
        .sum();
    (10 - sum % 10) % 10
}

/// Compute the check digit for the first eleven digits of a UPC-A code.
fn upca_check_digit(digits: &[u32]) -> u32 {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d * 3 } else { *d })
        .sum();
    (10 - sum % 10) % 10
}
