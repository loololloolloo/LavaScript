use std::collections::HashMap;
use wasm_encoder::{BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, TypeSection, ValType};
use crate::{lexer, parser::{self, CompareOp, Expr, Op, Stmt}};

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(&tokens)?;

    let mut locals = HashMap::new();
    collect_locals(&program, &mut locals)?;

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
    emit_statements(&mut main, &program, &locals)?;
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

fn collect_locals(statements: &[Stmt], locals: &mut HashMap<String, u32>) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let(name, _) => {
                if locals.contains_key(name) {
                    return Err(format!("variable `{name}` is already declared"));
                }
                let index = locals.len() as u32;
                locals.insert(name.clone(), index);
            }
            Stmt::If { then_body, else_body, .. } => {
                collect_locals(then_body, locals)?;
                collect_locals(else_body, locals)?;
            }
            Stmt::While { body, .. } => collect_locals(body, locals)?,
            Stmt::Print(_) => {}
        }
    }
    Ok(())
}

fn emit_statements(function: &mut Function, statements: &[Stmt], vars: &HashMap<String, u32>) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let(name, expr) => {
                emit_expr(function, expr, vars)?;
                let index = *vars.get(name).unwrap();
                function.instruction(&Instruction::LocalSet(index));
            }
            Stmt::Print(expr) => {
                emit_expr(function, expr, vars)?;
                function.instruction(&Instruction::Call(0));
            }
            Stmt::If { condition, then_body, else_body } => {
                emit_expr(function, condition, vars)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                emit_statements(function, then_body, vars)?;
                if !else_body.is_empty() {
                    function.instruction(&Instruction::Else);
                    emit_statements(function, else_body, vars)?;
                }
                function.instruction(&Instruction::End);
            }
            Stmt::While { condition, body } => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                emit_expr(function, condition, vars)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                emit_statements(function, body, vars)?;
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
    }
    Ok(())
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
        Expr::Compare(left, op, right) => {
            emit_expr(function, left, vars)?;
            emit_expr(function, right, vars)?;
            function.instruction(match op {
                CompareOp::Eq => &Instruction::I32Eq,
                CompareOp::Ne => &Instruction::I32Ne,
                CompareOp::Lt => &Instruction::I32LtS,
                CompareOp::Le => &Instruction::I32LeS,
                CompareOp::Gt => &Instruction::I32GtS,
                CompareOp::Ge => &Instruction::I32GeS,
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
    fn compiles_if_else() {
        validate("let x = 5\nif x > 3 then\nprint x\nelse\nprint 0\nend");
    }

    #[test]
    fn compiles_while() {
        validate("let x = 3\nwhile x > 0\nprint x\nend");
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
