# isbn-checksum

A command-line tool that answers one question: is this code valid, and
if not, what should the check digit have been? Covers ISBN-10,
ISBN-13, EAN-13, and UPC-A.

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

Every ISBN-13 is also a valid EAN-13 (with a 978 or 979 "Bookland"
prefix), and UPC-A barcodes use the same mod-10 idea with different
weights. Since all of these show up on real products next to a
scanner that might have misread them, the tool handles all four.

## Usage

```
$ isbn-checksum 978-0-306-40615-7
valid ISBN-13: 978-0-306-40615-7

$ isbn-checksum 156881111X
valid ISBN-10: 156881111X

$ isbn-checksum 0471958690
invalid: 0471958690
  check digit should be '7', found '0'

$ isbn-checksum 036000291452
valid UPC-A: 036000291452
```

The tool strips hyphens and spaces, then picks a format based on the
resulting length: 10 digits for ISBN-10, 12 for UPC-A, 13 for
ISBN-13 or EAN-13 (told apart by the 978/979 prefix).

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

ISBN-10, ISBN-13, EAN-13, and UPC-A are all covered, including the
ISBN-10 `X` check digit and formatted input. Not yet implemented:
computing a missing check digit, batch checking from stdin, and
property tests over randomly generated valid codes.
