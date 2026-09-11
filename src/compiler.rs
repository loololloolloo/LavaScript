use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};

const PRINT_TYPE: u32 = 0;
const MAIN_TYPE: u32 = 1;
const PRINT_FUNCTION: u32 = 0;
const MAIN_FUNCTION: u32 = 1;

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    types.ty().function([], []);

    let mut imports = ImportSection::new();
    imports.import(
        "lavascript",
        "print_i32",
        wasm_encoder::EntityType::Function(PRINT_TYPE),
    );

    let mut functions = FunctionSection::new();
    functions.function(MAIN_TYPE);

    let mut main = Function::new([]);
    let mut found_statement = false;

    for statement in source.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(value) = statement.strip_prefix("print ") {
            let value = value.trim().parse::<i32>().map_err(|_| {
                format!("expected an integer after `print`, got `{value}`")
            })?;
            main.instruction(&Instruction::I32Const(value));
            main.instruction(&Instruction::Call(PRINT_FUNCTION));
            found_statement = true;
        } else {
            return Err(format!("unsupported statement: `{statement}`"));
        }
    }

    if !found_statement {
        main.instruction(&Instruction::Nop);
    }
    main.instruction(&Instruction::End);

    let mut code = CodeSection::new();
    code.function(&main);

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, MAIN_FUNCTION);

    module
        .section(&types)
        .section(&imports)
        .section(&functions)
        .section(&exports)
        .section(&code);

    Ok(module.finish())
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn emits_valid_wasm() {
        let wasm = compile("print 42").unwrap();
        wasmparser::Validator::new().validate_all(&wasm).unwrap();
    }
}
