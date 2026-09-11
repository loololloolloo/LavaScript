use std::collections::HashMap;
use wasm_encoder::{BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, TypeSection, ValType};
use crate::{lexer, parser::{self, CompareOp, Expr, FunctionDecl, Op, Program, Stmt}};

type Signatures = HashMap<String, (u32, usize)>;

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(&tokens)?;
    let mut signatures = HashMap::new();
    for (i, function) in program.functions.iter().enumerate() {
        if signatures.insert(function.name.clone(), (1 + i as u32, function.params.len())).is_some() {
            return Err(format!("function `{}` is already declared", function.name));
        }
    }

    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    for function in &program.functions {
        types.ty().function(std::iter::repeat(ValType::I32).take(function.params.len()), [ValType::I32]);
    }
    let main_type = 1 + program.functions.len() as u32;
    types.ty().function([], []);

    let mut imports = ImportSection::new();
    imports.import("lavascript", "print_i32", wasm_encoder::EntityType::Function(0));

    let mut functions = FunctionSection::new();
    for (i, _) in program.functions.iter().enumerate() { functions.function(1 + i as u32); }
    functions.function(main_type);

    let mut code = CodeSection::new();
    for function_decl in &program.functions {
        let locals = collect_locals(&function_decl.body, &function_decl.params)?;
        let declarations = local_declarations(&locals, function_decl.params.len());
        let mut function = Function::new(declarations);
        emit_statements(&mut function, &function_decl.body, &locals, &signatures, true)?;
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        code.function(&function);
    }

    let main_locals = collect_locals(&program.main, &[])?;
    let mut main = Function::new(local_declarations(&main_locals, 0));
    emit_statements(&mut main, &program.main, &main_locals, &signatures, false)?;
    main.instruction(&Instruction::End);
    code.function(&main);

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 1 + program.functions.len() as u32);
    module.section(&types).section(&imports).section(&functions).section(&exports).section(&code);
    Ok(module.finish())
}

fn local_declarations(locals: &HashMap<String, u32>, param_count: usize) -> Vec<(u32, ValType)> {
    let count = locals.len().saturating_sub(param_count);
    if count == 0 { Vec::new() } else { vec![(count as u32, ValType::I32)] }
}

fn collect_locals(statements: &[Stmt], params: &[String]) -> Result<HashMap<String, u32>, String> {
    let mut locals = HashMap::new();
    for (index, name) in params.iter().enumerate() { locals.insert(name.clone(), index as u32); }
    collect_locals_into(statements, &mut locals)
}

fn collect_locals_into(statements: &[Stmt], locals: &mut HashMap<String, u32>) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let(name, _) => {
                if locals.contains_key(name) { return Err(format!("variable `{name}` is already declared")); }
                let index = locals.len() as u32;
                locals.insert(name.clone(), index);
            }
            Stmt::Assign(name, _) => { if !locals.contains_key(name) { return Err(format!("undefined variable `{name}`")); } }
            Stmt::If { then_body, else_body, .. } => { collect_locals_into(then_body, locals)?; collect_locals_into(else_body, locals)?; }
            Stmt::While { body, .. } => collect_locals_into(body, locals)?,
            Stmt::Print(_) | Stmt::Return(_) => {}
        }
    }
    Ok(())
}

fn emit_statements(function: &mut Function, statements: &[Stmt], vars: &HashMap<String, u32>, signatures: &Signatures, in_function: bool) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                emit_expr(function, expr, vars, signatures)?;
                let index = *vars.get(name).ok_or_else(|| format!("undefined variable `{name}`"))?;
                function.instruction(&Instruction::LocalSet(index));
            }
            Stmt::Print(expr) => { emit_expr(function, expr, vars, signatures)?; function.instruction(&Instruction::Call(0)); }
            Stmt::Return(expr) => {
                if !in_function { return Err("`return` is only valid inside a function".into()); }
                emit_expr(function, expr, vars, signatures)?;
                function.instruction(&Instruction::Return);
            }
            Stmt::If { condition, then_body, else_body } => {
                emit_expr(function, condition, vars, signatures)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                emit_statements(function, then_body, vars, signatures, in_function)?;
                if !else_body.is_empty() { function.instruction(&Instruction::Else); emit_statements(function, else_body, vars, signatures, in_function)?; }
                function.instruction(&Instruction::End);
            }
            Stmt::While { condition, body } => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                emit_expr(function, condition, vars, signatures)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                emit_statements(function, body, vars, signatures, in_function)?;
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
        }
    }
    Ok(())
}

fn emit_expr(function: &mut Function, expr: &Expr, vars: &HashMap<String, u32>, signatures: &Signatures) -> Result<(), String> {
    match expr {
        Expr::Number(n) => { function.instruction(&Instruction::I32Const(*n)); }
        Expr::Variable(name) => {
            let index = vars.get(name).ok_or_else(|| format!("undefined variable `{name}`"))?;
            function.instruction(&Instruction::LocalGet(*index));
        }
        Expr::Binary(left, op, right) => {
            emit_expr(function, left, vars, signatures)?; emit_expr(function, right, vars, signatures)?;
            function.instruction(match op { Op::Add=>&Instruction::I32Add, Op::Sub=>&Instruction::I32Sub, Op::Mul=>&Instruction::I32Mul, Op::Div=>&Instruction::I32DivS });
        }
        Expr::Compare(left, op, right) => {
            emit_expr(function, left, vars, signatures)?; emit_expr(function, right, vars, signatures)?;
            function.instruction(match op { CompareOp::Eq=>&Instruction::I32Eq, CompareOp::Ne=>&Instruction::I32Ne, CompareOp::Lt=>&Instruction::I32LtS, CompareOp::Le=>&Instruction::I32LeS, CompareOp::Gt=>&Instruction::I32GtS, CompareOp::Ge=>&Instruction::I32GeS });
        }
        Expr::Call(name, args) => {
            let (index, arity) = signatures.get(name).ok_or_else(|| format!("undefined function `{name}`"))?;
            if args.len() != *arity { return Err(format!("function `{name}` expects {arity} argument(s), got {}", args.len())); }
            for arg in args { emit_expr(function, arg, vars, signatures)?; }
            function.instruction(&Instruction::Call(*index));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;
    fn validate(source: &str) { let wasm = compile(source).unwrap(); wasmparser::Validator::new().validate_all(&wasm).unwrap(); }
    #[test] fn parses_and_compiles_math() { validate("print 2 + 3 * 4"); }
    #[test] fn compiles_local_variables() { validate("let x = 21\nprint x + x"); }
    #[test] fn compiles_multiple_variables() { validate("let x = 10\nlet y = x * 3\nprint y - 5"); }
    #[test] fn compiles_if_else() { validate("let x = 5\nif x > 3 then\nprint x\nelse\nprint 0\nend"); }
    #[test] fn compiles_while() { validate("let x = 3\nwhile x > 0\nprint x\nx = x - 1\nend"); }
    #[test] fn compiles_comparisons() { validate("let x = 5\nif x >= 5 then\nprint 1\nend"); }
    #[test] fn compiles_functions() { validate("function add(a, b)\nreturn a + b\nend\nprint add(2, 3)"); }
    #[test] fn compiles_function_locals() { validate("function square(x)\nlet y = x * x\nreturn y\nend\nprint square(4)"); }
    #[test] fn compiles_forward_calls() { validate("print add(7, 8)\nfunction add(a, b)\nreturn a + b\nend"); }
    #[test] fn compiles_function_control_flow() { validate("function countdown(x)\nwhile x > 0\nx = x - 1\nend\nreturn x\nend\nprint countdown(3)"); }
    #[test] fn rejects_undefined_variables() { assert!(compile("print missing").is_err()); }
    #[test] fn rejects_undefined_assignment() { assert!(compile("missing = 1").is_err()); }
    #[test] fn rejects_duplicate_declarations() { assert!(compile("let x = 1\nlet x = 2").is_err()); }
    #[test] fn rejects_undefined_function() { assert!(compile("print missing(1)").is_err()); }
    #[test] fn rejects_wrong_argument_count() { assert!(compile("function add(a,b)\nreturn a+b\nend\nprint add(1)").is_err()); }
    #[test] fn rejects_duplicate_function() { assert!(compile("function x()\nreturn 1\nend\nfunction x()\nreturn 2\nend").is_err()); }
}
