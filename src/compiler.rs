use std::collections::HashMap;
use wasm_encoder::{CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, TypeSection, ValType};
use crate::{lexer, parser::{self, Expr, Op, Stmt}};

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(&tokens)?;

    let mut locals = HashMap::new();
    for stmt in &program {
        if let Stmt::Let(name, _) = stmt {
            if locals.contains_key(name) {
                return Err(format!("variable `{name}` is already declared"));
            }
            let index = locals.len() as u32;
            locals.insert(name.clone(), index);
        }
    }

    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    types.ty().function([], []);

    let mut imports = ImportSection::new();
    imports.import("lavascript", "print_i32", wasm_encoder::EntityType::Function(0));

    let mut functions = FunctionSection::new();
    functions.function(1);

    let local_declarations = if locals.is_empty() {
        Vec::new()
    } else {
        vec![(locals.len() as u32, ValType::I32)]
    };
    let mut main = Function::new(local_declarations);

    for stmt in &program {
        match stmt {
            Stmt::Let(name, expr) => {
                emit_expr(&mut main, expr, &locals)?;
                let index = *locals.get(name).unwrap();
                main.instruction(&Instruction::LocalSet(index));
            }
            Stmt::Print(expr) => {
                emit_expr(&mut main, expr, &locals)?;
                main.instruction(&Instruction::Call(0));
            }
        }
    }

    main.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&main);

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 1);

    module
        .section(&types)
        .section(&imports)
        .section(&functions)
        .section(&exports)
        .section(&code);

    Ok(module.finish())
}

fn emit_expr(function: &mut Function, expr: &Expr, vars: &HashMap<String, u32>) -> Result<(), String> {
    match expr {
        Expr::Number(n) => function.instruction(&Instruction::I32Const(*n)),
        Expr::Variable(name) => {
            let index = vars
                .get(name)
                .ok_or_else(|| format!("undefined variable `{name}`"))?;
            function.instruction(&Instruction::LocalGet(*index));
        }
        Expr::Binary(left, op, right) => {
            emit_expr(function, left, vars)?;
            emit_expr(function, right, vars)?;
            function.instruction(match op {
                Op::Add => &Instruction::I32Add,
                Op::Sub => &Instruction::I32Sub,
                Op::Mul => &Instruction::I32Mul,
                Op::Div => &Instruction::I32DivS,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;

    fn validate(source: &str) {
        let wasm = compile(source).unwrap();
        wasmparser::Validator::new().validate_all(&wasm).unwrap();
    }

    #[test]
    fn parses_and_compiles_math() {
        validate("print 2 + 3 * 4");
    }

    #[test]
    fn compiles_local_variables() {
        validate("let x = 21\nprint x + x");
    }

    #[test]
    fn compiles_multiple_variables() {
        validate("let x = 10\nlet y = x * 3\nprint y - 5");
    }

    #[test]
    fn rejects_undefined_variables() {
        assert!(compile("print missing").is_err());
    }

    #[test]
    fn rejects_duplicate_declarations() {
        assert!(compile("let x = 1\nlet x = 2").is_err());
    }
}
