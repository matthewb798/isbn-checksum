# isbn-checksum

A command-line tool that answers one question: is this ISBN valid, and
if not, what should the check digit have been?

## Why this exists

Both ISBN formats end in a digit that isn't part of the number, it's a
checksum computed from the rest. That's what catches a typo when
someone hand-enters a code, or a scanner misreads a barcode. Getting
the checksum math wrong is easy in a way that doesn't show up until
someone tests it against a real book:

- ISBN-10 uses mod 11, which has eleven possible remainders. A single
  decimal digit only covers ten of them, so the tenth character can
  legally be the letter `X` (standing in for the value 10).
- ISBN-13 switched to mod 10 with alternating weights of 1 and 3, the
  same scheme used by EAN-13 and UPC-A barcodes. No letters involved,
  but the weighting alternates in the opposite direction from what
  people sometimes expect.
- Real-world ISBNs show up with hyphens, spaces, and inconsistent
  casing on the `X`. A checker that only accepts thirteen bare digits
  rejects most codes as printed on an actual book.

## Usage

```
$ isbn-checksum 978-0-306-40615-7
valid ISBN-13: 978-0-306-40615-7

$ isbn-checksum 156881111X
valid ISBN-10: 156881111X

$ isbn-checksum 0471958690
invalid: 0471958690
  check digit should be '7', found '0'
```

The tool strips hyphens and spaces, then picks ISBN-10 or ISBN-13
based on the resulting length (10 or 13 digits).

## Library

The checksum logic lives in `src/lib.rs` and can be used directly:

```rust
use isbn_checksum::check;

match check("978-0-306-40615-7") {
    Ok(kind) => println!("valid {kind}"),
    Err(e) => println!("invalid: {e}"),
}
```

## Status

ISBN-10 and ISBN-13 are covered, including the `X` check digit and
formatted input. UPC-A and EAN-13 barcodes use a closely related
checksum and are planned but not implemented yet.
