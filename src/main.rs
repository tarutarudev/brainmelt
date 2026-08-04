use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut optimize = true;

    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }

            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: missing output file after '{}'", args[i - 1]);
                    return ExitCode::FAILURE;
                }
                output = Some(args[i].clone());
            }

            "--no-opt" => {
                optimize = false;
            }

            arg if arg.starts_with('-') => {
                eprintln!("error: unknown option '{arg}'");
                return ExitCode::FAILURE;
            }

            arg => {
                if input.is_some() {
                    eprintln!("error: multiple input files are not supported");
                    return ExitCode::FAILURE;
                }
                input = Some(arg.to_string());
            }
        }

        i += 1;
    }

    let input = match input {
        Some(input) => input,
        None => {
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let source = match fs::read_to_string(&input) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("error: cannot read '{input}': {e}");
            return ExitCode::FAILURE;
        }
    };

    match brainmelt::compile_to_native_linux_amd64(&source, optimize) {
        Ok(bytes) => {
            let path = output.unwrap_or_else(|| "a.out".to_string());

            if let Err(e) = fs::write(&path, &bytes) {
                eprintln!("error: cannot write '{path}': {e}");
                return ExitCode::FAILURE;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let permissions = fs::Permissions::from_mode(0o755);
                if let Err(e) = fs::set_permissions(&path, permissions) {
                    eprintln!("warning: failed to set executable bit: {e}");
                }
            }

            ExitCode::SUCCESS
        }

        Err(e) => {
            eprintln!("compile error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    println!("Usage: brainmelt <input.bf> [-o output] [--no-opt]");
}
