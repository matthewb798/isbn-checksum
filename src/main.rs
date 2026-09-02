use std::env;
use std::process::ExitCode;

use isbn_checksum::check;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(code) = args.next() else {
        eprintln!("usage: isbn-checksum <code>");
        eprintln!("       isbn-checksum 978-0-306-40615-7");
        return ExitCode::FAILURE;
    };

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
