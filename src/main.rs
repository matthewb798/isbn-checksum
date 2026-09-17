use std::env;
use std::io::{self, BufRead, IsTerminal};
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
        if compute_mode {
            eprintln!("usage: isbn-checksum --compute <partial-code>");
            return ExitCode::FAILURE;
        }
        return check_stdin();
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

/// Batch mode: one code per line on stdin, one result per line on
/// stdout. Only reached when no code was given on the command line, so
/// an interactive terminal (nothing piped in) gets the usage message
/// instead of hanging on a read.
fn check_stdin() -> ExitCode {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        eprintln!("usage: isbn-checksum <code>");
        eprintln!("       isbn-checksum --compute <partial-code>");
        eprintln!("       isbn-checksum 978-0-306-40615-7        (ISBN-13)");
        eprintln!("       isbn-checksum 156881111X               (ISBN-10)");
        eprintln!("       isbn-checksum 036000291452             (UPC-A)");
        eprintln!("       isbn-checksum --compute 978030640615   (missing ISBN-13 digit)");
        eprintln!("       <codes, one per line> | isbn-checksum   (batch mode)");
        return ExitCode::FAILURE;
    }

    let mut all_valid = true;
    for line in stdin.lock().lines() {
        let line = line.expect("failed to read line from stdin");
        let code = line.trim();
        if code.is_empty() {
            continue;
        }
        match check(code) {
            Ok(kind) => println!("valid {kind}: {code}"),
            Err(e) => {
                println!("invalid: {code}");
                println!("  {e}");
                all_valid = false;
            }
        }
    }

    if all_valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
