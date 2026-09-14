use std::env;
use std::process::ExitCode;

use isbn_checksum::{check, compute};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let first = args.next();

    let (compute_mode, code) = match first.as_deref() {
        Some("--compute") | Some("-c") => (true, args.next()),
        _ => (false, first),
    };

    let Some(code) = code else {
        eprintln!("usage: isbn-checksum <code>");
        eprintln!("       isbn-checksum --compute <partial-code>");
        eprintln!("       isbn-checksum 978-0-306-40615-7        (ISBN-13)");
        eprintln!("       isbn-checksum 156881111X               (ISBN-10)");
        eprintln!("       isbn-checksum 036000291452             (UPC-A)");
        eprintln!("       isbn-checksum --compute 978030640615   (missing ISBN-13 digit)");
        return ExitCode::FAILURE;
    };

    if compute_mode {
        return match compute(&code) {
            Ok((kind, full)) => {
                println!("{full} ({kind})");
                ExitCode::SUCCESS
            }
            Err(e) => {
                println!("can't compute a check digit for {code}");
                println!("  {e}");
                ExitCode::FAILURE
            }
        };
    }

    match check(&code) {
        Ok(kind) => {
            println!("valid {kind}: {code}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("invalid: {code}");
            println!("  {e}");
            ExitCode::FAILURE
        }
    }
}
