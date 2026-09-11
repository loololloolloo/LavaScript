use std::{env, fs, process};

mod checker;
mod compiler;
mod formatter;
mod lexer;
mod optimizer;
mod parser;
mod stdlib;
mod value;

fn usage() -> ! {
    eprintln!("LavaScript compiler");
    eprintln!("usage:");
    eprintln!("  lavascript build <file.ls> [-o <file.wasm>]");
    eprintln!("  lavascript check <file.ls>");
    eprintln!("  lavascript fmt <file.ls> [-w]");
    process::exit(2);
}

fn load(input: &str) -> (String, parser::Program) {
    let source = fs::read_to_string(input).unwrap_or_else(|e| { eprintln!("could not read {input}: {e}"); process::exit(1); });
    let tokens = lexer::lex(&source).unwrap_or_else(|e| { eprintln!("lex error: {e}"); process::exit(1); });
    let program = parser::parse(&tokens).unwrap_or_else(|e| { eprintln!("parse error: {e}"); process::exit(1); });
    checker::check(&program).unwrap_or_else(|e| { eprintln!("type error: {e}"); process::exit(1); });
    (source, program)
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
            let (source, _) = load(&input);
            let wasm = compiler::compile(&source).unwrap_or_else(|e| { eprintln!("compile error: {e}"); process::exit(1); });
            fs::write(&output, wasm).unwrap_or_else(|e| { eprintln!("could not write {output}: {e}"); process::exit(1); });
            println!("compiled {input} -> {output}");
        }
        Some("check") => {
            let input = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() { eprintln!("too many arguments for `check`"); usage(); }
            load(&input);
            println!("ok: {input}");
        }
        Some("fmt") => {
            let input = args.next().unwrap_or_else(|| usage());
            let write = args.next().as_deref() == Some("-w");
            if !write && args.next().is_some() { usage(); }
            let (source, program) = load(&input);
            let formatted = formatter::format_program(&program);
            if write { fs::write(&input, formatted).unwrap_or_else(|e| { eprintln!("could not write {input}: {e}"); process::exit(1); }); }
            else { print!("{formatted}"); }
            let _ = source;
        }
        _ => usage(),
    }
}
