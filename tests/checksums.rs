//! Table-driven coverage of the cases that are easy to get wrong:
//! the ISBN-10 'X' digit, separators, wrong-length input, and the
//! degenerate all-zero code that passes the math but looks suspicious.

use isbn_checksum::{
    check, check_ean13, check_isbn10, check_isbn13, check_upca, compute, BarcodeKind, CheckError,
};

#[test]
fn isbn10_cases() {
    let cases: &[(&str, Result<(), CheckError>)] = &[
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
            Err(CheckError::BadCheckDigit {
                expected: '7',
                found: '0',
            }),
        ),
        // X only means anything as the tenth character
        ("X471958697", Err(CheckError::InvalidChar('X'))),
        // letter that isn't a valid check digit stand-in at all
        ("047195869A", Err(CheckError::InvalidChar('A'))),
        ("047195869", Err(CheckError::WrongLength(9))),
        ("04719586970", Err(CheckError::WrongLength(11))),
        ("", Err(CheckError::WrongLength(0))),
    ];

    for (input, expected) in cases {
        assert_eq!(check_isbn10(input), *expected, "input: {input:?}");
    }
}

#[test]
fn isbn13_cases() {
    let cases: &[(&str, Result<(), CheckError>)] = &[
        ("9780306406157", Ok(())),
        ("978-0-306-40615-7", Ok(())),
        ("978 0 306 40615 7", Ok(())),
        ("0000000000000", Ok(())),
        (
            "9780306406158",
            Err(CheckError::BadCheckDigit {
                expected: '7',
                found: '8',
            }),
        ),
        ("97803064061", Err(CheckError::WrongLength(11))),
        ("97803064061579", Err(CheckError::WrongLength(14))),
        ("978030640615x", Err(CheckError::InvalidChar('x'))),
    ];

    for (input, expected) in cases {
        assert_eq!(check_isbn13(input), *expected, "input: {input:?}");
    }
}

#[test]
fn upca_cases() {
    let cases: &[(&str, Result<(), CheckError>)] = &[
        // Wrigley's gum, the textbook UPC-A example
        ("036000291452", Ok(())),
        ("036-00029145-2", Ok(())),
        ("000000000000", Ok(())),
        (
            "036000291450",
            Err(CheckError::BadCheckDigit {
                expected: '2',
                found: '0',
            }),
        ),
        ("03600029145", Err(CheckError::WrongLength(11))),
        ("0360002914520", Err(CheckError::WrongLength(13))),
        ("03600029145X", Err(CheckError::InvalidChar('X'))),
    ];

    for (input, expected) in cases {
        assert_eq!(check_upca(input), *expected, "input: {input:?}");
    }
}

#[test]
fn ean13_matches_isbn13_math() {
    // A non-Bookland EAN-13 (no 978/979 prefix) still validates with
    // the same mod-10 checksum as an ISBN-13.
    assert_eq!(check_ean13("4006381333931"), Ok(()));
    assert_eq!(
        check_ean13("4006381333932"),
        Err(CheckError::BadCheckDigit {
            expected: '1',
            found: '2',
        })
    );
}

#[test]
fn compute_dispatches_on_length() {
    let cases: &[(&str, Result<(BarcodeKind, &str), CheckError>)] = &[
        // ISBN-10 body, 9 digits
        ("047195869", Ok((BarcodeKind::Isbn10, "0471958697"))),
        // ISBN-10 whose check digit works out to the letter X
        ("156881111", Ok((BarcodeKind::Isbn10, "156881111X"))),
        // UPC-A body, 11 digits, with formatting
        ("03600029145", Ok((BarcodeKind::UpcA, "036000291452"))),
        ("036-0002914-5", Ok((BarcodeKind::UpcA, "036000291452"))),
        // ISBN-13 body, 12 digits, Bookland prefix
        ("978030640615", Ok((BarcodeKind::Isbn13, "9780306406157"))),
        // 12-digit body without a Bookland prefix is a plain EAN-13
        ("400638133393", Ok((BarcodeKind::Ean13, "4006381333931"))),
        ("04719586A", Err(CheckError::InvalidChar('A'))),
        ("04719586", Err(CheckError::WrongLength(8))),
        ("0471958697", Err(CheckError::WrongLength(10))),
    ];

    for (input, expected) in cases {
        let actual = compute(input);
        match expected {
            Ok((kind, full)) => assert_eq!(actual, Ok((*kind, full.to_string())), "input: {input:?}"),
            Err(e) => assert_eq!(actual, Err(*e), "input: {input:?}"),
        }
    }
}

#[test]
fn check_dispatches_on_length() {
    assert_eq!(check("156881111X"), Ok(BarcodeKind::Isbn10));
    assert_eq!(check("978-0-306-40615-7"), Ok(BarcodeKind::Isbn13));
    assert_eq!(check("036000291452"), Ok(BarcodeKind::UpcA));
    assert_eq!(check("4006381333931"), Ok(BarcodeKind::Ean13));
    assert_eq!(check("12345"), Err(CheckError::WrongLength(5)));
}
