use wasm_encoder::{CodeSection, Function, FunctionSection, Instruction, Module, TypeSection, ValType};

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut functions = FunctionSection::new();
    let mut code = CodeSection::new();

    let mut body = Function::new([]);
    let mut has_statement = false;

    for statement in source.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(value) = statement.strip_prefix("print ") {
            let number: i32 = value
                .trim()
                .parse()
                .map_err(|_| format!("expected an integer after print, got `{value}`"))?;
            body.instruction(&Instruction::I32Const(number));
            has_statement = true;
        } else {
            return Err(format!("unsupported statement: `{statement}`"));
        }
    }

    if !has_statement {
        body.instruction(&Instruction::Nop);
    }

    body.instruction(&Instruction::End);

    let type_index = types.function([], []);
    functions.function(type_index);
    code.function(&body);

    module.section(&types);
    module.section(&functions);
    module.section(&code);
    module.section(&wasm_encoder::ExportSection::new());

    let _ = ValType::I32;
    Ok(module.finish())
}
