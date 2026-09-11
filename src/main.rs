use std::{env, fs, process};

mod compiler;
mod lexer;
mod optimizer;
mod parser;

fn usage() -> ! {
    eprintln!("LavaScript compiler");
    eprintln!("usage:");
    eprintln!("  lavascript build <file.ls> [-o <file.wasm>]");
    eprintln!("  lavascript check <file.ls>");
    process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("build") => {
            let input = args.next().unwrap_or_else(|| usage());
            let mut output = String::from("out.wasm");
            while let Some(arg) = args.next() {
                if arg == "-o" { output = args.next().unwrap_or_else(|| { eprintln!("missing output path after -o"); process::exit(2); }); }
                else { eprintln!("unknown build option `{arg}`"); usage(); }
            }
            let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("could not read {input}: {e}"); process::exit(1); });
            let wasm = compiler::compile(&source).unwrap_or_else(|e| { eprintln!("compile error: {e}"); process::exit(1); });
            fs::write(&output, wasm).unwrap_or_else(|e| { eprintln!("could not write {output}: {e}"); process::exit(1); });
            println!("compiled {input} -> {output}");
        }
        Some("check") => {
            let input = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() { eprintln!("too many arguments for `check`"); usage(); }
            let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("could not read {input}: {e}"); process::exit(1); });
            compiler::compile(&source).unwrap_or_else(|e| { eprintln!("check failed: {e}"); process::exit(1); });
            println!("ok: {input}");
        }
        _ => usage(),
    }
}
