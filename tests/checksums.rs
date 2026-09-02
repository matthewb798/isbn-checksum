//! Table-driven coverage of the cases that are easy to get wrong:
//! the ISBN-10 'X' digit, separators, wrong-length input, and the
//! degenerate all-zero code that passes the math but looks suspicious.

use isbn_checksum::{check, check_isbn10, check_isbn13, IsbnError, IsbnKind};

#[test]
fn isbn10_cases() {
    let cases: &[(&str, Result<(), IsbnError>)] = &[
        ("0471958697", Ok(())),
        // check digit is the letter X, standing in for 10
        ("156881111X", Ok(())),
        // lowercase x must also be accepted
        ("156881111x", Ok(())),
        // hyphens and spaces are just formatting, not data
        ("0-471-95869-7", Ok(())),
        ("0 471 95869 7", Ok(())),
        // sum is 0 mod 11, which is a legal (if silly) checksum
        ("0000000000", Ok(())),
        // last digit wrong: 7 -> 0
        (
            "0471958690",
            Err(IsbnError::BadCheckDigit {
                expected: '7',
                found: '0',
            }),
        ),
        // X only means anything as the tenth character
        ("X471958697", Err(IsbnError::InvalidChar('X'))),
        // letter that isn't a valid check digit stand-in at all
        ("047195869A", Err(IsbnError::InvalidChar('A'))),
        ("047195869", Err(IsbnError::WrongLength(9))),
        ("04719586970", Err(IsbnError::WrongLength(11))),
        ("", Err(IsbnError::WrongLength(0))),
    ];

    for (input, expected) in cases {
        assert_eq!(check_isbn10(input), *expected, "input: {input:?}");
    }
}

#[test]
fn isbn13_cases() {
    let cases: &[(&str, Result<(), IsbnError>)] = &[
        ("9780306406157", Ok(())),
        ("978-0-306-40615-7", Ok(())),
        ("978 0 306 40615 7", Ok(())),
        ("0000000000000", Ok(())),
        (
            "9780306406158",
            Err(IsbnError::BadCheckDigit {
                expected: '7',
                found: '8',
            }),
        ),
        ("97803064061", Err(IsbnError::WrongLength(11))),
        ("97803064061579", Err(IsbnError::WrongLength(14))),
        ("978030640615x", Err(IsbnError::InvalidChar('x'))),
    ];

    for (input, expected) in cases {
        assert_eq!(check_isbn13(input), *expected, "input: {input:?}");
    }
}

#[test]
fn check_dispatches_on_length() {
    assert_eq!(check("156881111X"), Ok(IsbnKind::Isbn10));
    assert_eq!(check("978-0-306-40615-7"), Ok(IsbnKind::Isbn13));
    assert_eq!(check("12345"), Err(IsbnError::WrongLength(5)));
}
