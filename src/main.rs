use std::{env, fs, process};

mod compiler;

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();

    match command.as_str() {
        "build" => {
            let input = args.next().unwrap_or_else(|| {
                eprintln!("usage: lavascript build <file.ls> [-o <file.wasm>]");
                process::exit(2);
            });

            let mut output = String::from("out.wasm");
            while let Some(arg) = args.next() {
                if arg == "-o" {
                    output = args.next().unwrap_or_else(|| {
                        eprintln!("missing output path after -o");
                        process::exit(2);
                    });
                }
            }

            let source = fs::read_to_string(&input).unwrap_or_else(|e| {
                eprintln!("could not read {input}: {e}");
                process::exit(1);
            });

            let wasm = compiler::compile(&source).unwrap_or_else(|e| {
                eprintln!("compile error: {e}");
                process::exit(1);
            });

            fs::write(&output, wasm).unwrap_or_else(|e| {
                eprintln!("could not write {output}: {e}");
                process::exit(1);
            });

            println!("compiled {input} -> {output}");
        }
        _ => {
            eprintln!("LavaScript compiler");
            eprintln!("usage: lavascript build <file.ls> [-o <file.wasm>]");
            process::exit(2);
        }
    }
}
