use std::collections::HashMap;
use wasm_encoder::{CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, TypeSection, ValType};
use crate::{lexer, parser::{self, Expr, Op, Stmt}};

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(&tokens)?;
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    types.ty().function([], []);
    let mut imports = ImportSection::new();
    imports.import("lavascript", "print_i32", wasm_encoder::EntityType::Function(0));
    let mut functions = FunctionSection::new();
    functions.function(1);
    let mut main = Function::new([]);
    let mut vars = HashMap::new();
    for stmt in program {
        match stmt {
            Stmt::Let(name, expr) => { emit_expr(&mut main, &expr, &vars)?; vars.insert(name, ()); main.instruction(&Instruction::Drop); }
            Stmt::Print(expr) => { emit_expr(&mut main, &expr, &vars)?; main.instruction(&Instruction::Call(0)); }
        }
    }
    main.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&main);
    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 1);
    module.section(&types).section(&imports).section(&functions).section(&exports).section(&code);
    Ok(module.finish())
}

fn emit_expr(function: &mut Function, expr: &Expr, vars: &HashMap<String, ()>) -> Result<(), String> {
    match expr {
        Expr::Number(n) => function.instruction(&Instruction::I32Const(*n)),
        Expr::Variable(name) => return Err(format!("variable `{name}` is not loadable yet")),
        Expr::Binary(left, op, right) => {
            emit_expr(function, left, vars)?;
            emit_expr(function, right, vars)?;
            function.instruction(match op { Op::Add => &Instruction::I32Add, Op::Sub => &Instruction::I32Sub, Op::Mul => &Instruction::I32Mul, Op::Div => &Instruction::I32DivS });
        }
    }
    let _ = vars;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;
    #[test]
    fn parses_and_compiles_math() {
        let wasm = compile("print 2 + 3 * 4").unwrap();
        wasmparser::Validator::new().validate_all(&wasm).unwrap();
    }
}
