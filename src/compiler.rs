use std::collections::{HashMap, HashSet};
use wasm_encoder::{BlockType, CodeSection, DataSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, MemorySection, Module, TypeSection, ValType};
use crate::{lexer, parser::{self, CompareOp, Expr, LogicalOp, Op, Program, Stmt}};

type Signatures = HashMap<String, (u32, usize)>;

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(&tokens)?;
    let mut signatures = HashMap::new();
    for (i, function) in program.functions.iter().enumerate() {
        if signatures.insert(function.name.clone(), (2 + i as u32, function.params.len())).is_some() { return Err(format!("function `{}` is already declared", function.name)); }
    }
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    types.ty().function([ValType::I32, ValType::I32], []);
    for function in &program.functions { types.ty().function(std::iter::repeat(ValType::I32).take(function.params.len()), [ValType::I32]); }
    let main_type = 2 + program.functions.len() as u32;
    types.ty().function([], []);
    let mut imports = ImportSection::new();
    imports.import("lavascript", "print_i32", wasm_encoder::EntityType::Function(0));
    imports.import("lavascript", "print_string", wasm_encoder::EntityType::Function(1));
    let mut functions = FunctionSection::new();
    for (i, _) in program.functions.iter().enumerate() { functions.function(2 + i as u32); }
    functions.function(main_type);
    let mut code = CodeSection::new();
    let mut data = DataSection::new();
    let mut strings = StringTable::new();
    for function_decl in &program.functions {
        let (locals, consts) = collect_locals(&function_decl.body, &function_decl.params)?;
        let mut function = Function::new(local_declarations(&locals, function_decl.params.len()));
        emit_statements(&mut function, &function_decl.body, &locals, &consts, &signatures, true, &mut strings, 0)?;
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        code.function(&function);
    }
    let (main_locals, main_consts) = collect_locals(&program.main, &[])?;
    let mut main = Function::new(local_declarations(&main_locals, 0));
    emit_statements(&mut main, &program.main, &main_locals, &main_consts, &signatures, false, &mut strings, 0)?;
    main.instruction(&Instruction::End);
    code.function(&main);
    let mut memory = MemorySection::new();
    memory.memory(wasm_encoder::MemoryType { minimum: 1, maximum: None, memory64: false, shared: false, page_size_log2: None });
    for (offset, bytes) in &strings.entries { data.active(*offset, &bytes[..]); }
    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 2 + program.functions.len() as u32);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&types).section(&imports).section(&functions).section(&memory).section(&exports).section(&code).section(&data);
    Ok(module.finish())
}

struct StringTable { entries: Vec<(u32, Vec<u8>)>, next: u32 }
impl StringTable {
    fn new() -> Self { Self { entries: Vec::new(), next: 0 } }
    fn intern(&mut self, value: &str) -> u32 {
        let wanted = value.as_bytes();
        if let Some((offset, _)) = self.entries.iter().find(|(_, bytes)| bytes.as_slice() == wanted) { return *offset; }
        let offset = self.next; let bytes = wanted.to_vec(); self.next += bytes.len() as u32 + 1; self.entries.push((offset, bytes)); offset
    }
}

fn local_declarations(locals: &HashMap<String, u32>, param_count: usize) -> Vec<(u32, ValType)> { let count=locals.len().saturating_sub(param_count); if count==0{Vec::new()}else{vec![(count as u32,ValType::I32)]} }
fn collect_locals(statements:&[Stmt],params:&[String])->Result<(HashMap<String,u32>,HashSet<String>),String>{let mut locals=HashMap::new();let mut consts=HashSet::new();for(index,name)in params.iter().enumerate(){locals.insert(name.clone(),index as u32);}collect_locals_into(statements,&mut locals,&mut consts)?;Ok((locals,consts))}
fn collect_locals_into(statements:&[Stmt],locals:&mut HashMap<String,u32>,consts:&mut HashSet<String>)->Result<(),String>{for stmt in statements{match stmt{
    Stmt::Let(name,_)=>{if locals.contains_key(name){return Err(format!("variable `{name}` is already declared"));}let index=locals.len() as u32;locals.insert(name.clone(),index);}
    Stmt::Const(name,_)=>{if locals.contains_key(name){return Err(format!("variable `{name}` is already declared"));}let index=locals.len() as u32;locals.insert(name.clone(),index);consts.insert(name.clone());}
    Stmt::Assign(name,_)=>{if !locals.contains_key(name){return Err(format!("undefined variable `{name}`"));}if consts.contains(name){return Err(format!("cannot assign to constant `{name}`"));}}
    Stmt::If{then_body,else_body,..}=>{collect_locals_into(then_body,locals,consts)?;collect_locals_into(else_body,locals,consts)?;}
    Stmt::While{body,..}|Stmt::Repeat{body,..}|Stmt::Do{body,..}=>collect_locals_into(body,locals,consts)?,
    Stmt::For{name,body,..}=>{if !locals.contains_key(name){let index=locals.len() as u32;locals.insert(name.clone(),index);}collect_locals_into(body,locals,consts)?;}
    Stmt::Print(_) | Stmt::Return(_) | Stmt::Break | Stmt::Continue=>{}
}}Ok(())}

fn emit_statements(function:&mut Function,statements:&[Stmt],vars:&HashMap<String,u32>,consts:&HashSet<String>,signatures:&Signatures,in_function:bool,strings:&mut StringTable,loop_depth:usize)->Result<(),String>{
    for stmt in statements{match stmt{
        Stmt::Let(name,expr)|Stmt::Const(name,expr)=>{emit_expr(function,expr,vars,signatures,strings)?;let index=*vars.get(name).ok_or_else(||format!("undefined variable `{name}`"))?;function.instruction(&Instruction::LocalSet(index));}
        Stmt::Assign(name,expr)=>{if consts.contains(name){return Err(format!("cannot assign to constant `{name}`"));}emit_expr(function,expr,vars,signatures,strings)?;let index=*vars.get(name).ok_or_else(||format!("undefined variable `{name}`"))?;function.instruction(&Instruction::LocalSet(index));}
        Stmt::Print(expr)=>{match expr{Expr::String(value)=>{let offset=strings.intern(value);let len=value.len() as i32;function.instruction(&Instruction::I32Const(offset as i32));function.instruction(&Instruction::I32Const(len));function.instruction(&Instruction::Call(1));}_=>{emit_expr(function,expr,vars,signatures,strings)?;function.instruction(&Instruction::Call(0));}}}
        Stmt::Return(expr)=>{if !in_function{return Err("`return` is only valid inside a function".into());}emit_expr(function,expr,vars,signatures,strings)?;function.instruction(&Instruction::Return);}
        Stmt::If{condition,then_body,else_body}=>{emit_expr(function,condition,vars,signatures,strings)?;function.instruction(&Instruction::If(BlockType::Empty));emit_statements(function,then_body,vars,consts,signatures,in_function,strings,loop_depth)?;if !else_body.is_empty(){function.instruction(&Instruction::Else);emit_statements(function,else_body,vars,consts,signatures,in_function,strings,loop_depth)?;}function.instruction(&Instruction::End);}
        Stmt::While{condition,body}=>{function.instruction(&Instruction::Block(BlockType::Empty));function.instruction(&Instruction::Loop(BlockType::Empty));emit_expr(function,condition,vars,signatures,strings)?;function.instruction(&Instruction::I32Eqz);function.instruction(&Instruction::BrIf(1));emit_statements(function,body,vars,consts,signatures,in_function,strings,loop_depth+1)?;function.instruction(&Instruction::Br(0));function.instruction(&Instruction::End);function.instruction(&Instruction::End);}
        Stmt::For{name,start,end,step,body}=>{let step_value=match step{Expr::Number(n) if *n!=0=>*n,_=>return Err("`for` step must be a non-zero numeric literal".into())};let index=*vars.get(name).ok_or_else(||format!("undefined variable `{name}`"))?;emit_expr(function,start,vars,signatures,strings)?;function.instruction(&Instruction::LocalSet(index));function.instruction(&Instruction::Block(BlockType::Empty));function.instruction(&Instruction::Loop(BlockType::Empty));function.instruction(&Instruction::LocalGet(index));emit_expr(function,end,vars,signatures,strings)?;if step_value>0{function.instruction(&Instruction::I32GtS);function.instruction(&Instruction::I32Eqz);}else{function.instruction(&Instruction::I32LtS);function.instruction(&Instruction::I32Eqz);}function.instruction(&Instruction::BrIf(1));emit_statements(function,body,vars,consts,signatures,in_function,strings,loop_depth+1)?;function.instruction(&Instruction::LocalGet(index));function.instruction(&Instruction::I32Const(step_value));function.instruction(&Instruction::I32Add);function.instruction(&Instruction::LocalSet(index));function.instruction(&Instruction::Br(0));function.instruction(&Instruction::End);function.instruction(&Instruction::End);}
        Stmt::Repeat{body,condition}=>{function.instruction(&Instruction::Block(BlockType::Empty));function.instruction(&Instruction::Loop(BlockType::Empty));emit_statements(function,body,vars,consts,signatures,in_function,strings,loop_depth+1)?;emit_expr(function,condition,vars,signatures,strings)?;function.instruction(&Instruction::BrIf(1));function.instruction(&Instruction::Br(0));function.instruction(&Instruction::End);function.instruction(&Instruction::End);}
        Stmt::Do{body}=>emit_statements(function,body,vars,consts,signatures,in_function,strings,loop_depth)?,
        Stmt::Break=>{if loop_depth==0{return Err("`break` is only valid inside a loop".into());}function.instruction(&Instruction::Br(loop_depth as u32));}
        Stmt::Continue=>{if loop_depth==0{return Err("`continue` is only valid inside a loop".into());}function.instruction(&Instruction::Br(0));}
    }}Ok(())
}

fn emit_expr(function:&mut Function,expr:&Expr,vars:&HashMap<String,u32>,signatures:&Signatures,strings:&mut StringTable)->Result<(),String>{match expr{
    Expr::Number(n)=>function.instruction(&Instruction::I32Const(*n)),Expr::Bool(value)=>function.instruction(&Instruction::I32Const(if *value{1}else{0})),Expr::String(_)=>return Err("string values can only be used with `print` for now".into()),
    Expr::Variable(name)=>{let index=vars.get(name).ok_or_else(||format!("undefined variable `{name}`"))?;function.instruction(&Instruction::LocalGet(*index));}
    Expr::Binary(left,op,right)=>{emit_expr(function,left,vars,signatures,strings)?;emit_expr(function,right,vars,signatures,strings)?;function.instruction(match op{Op::Add=>&Instruction::I32Add,Op::Sub=>&Instruction::I32Sub,Op::Mul=>&Instruction::I32Mul,Op::Div=>&Instruction::I32DivS,Op::Mod=>&Instruction::I32RemS});}
    Expr::Compare(left,op,right)=>{emit_expr(function,left,vars,signatures,strings)?;emit_expr(function,right,vars,signatures,strings)?;function.instruction(match op{CompareOp::Eq=>&Instruction::I32Eq,CompareOp::Ne=>&Instruction::I32Ne,CompareOp::Lt=>&Instruction::I32LtS,CompareOp::Le=>&Instruction::I32LeS,CompareOp::Gt=>&Instruction::I32GtS,CompareOp::Ge=>&Instruction::I32GeS});}
    Expr::Logical(left,LogicalOp::And,right)=>{emit_expr(function,left,vars,signatures,strings)?;emit_expr(function,right,vars,signatures,strings)?;function.instruction(&Instruction::I32And);}
    Expr::Logical(left,LogicalOp::Or,right)=>{emit_expr(function,left,vars,signatures,strings)?;emit_expr(function,right,vars,signatures,strings)?;function.instruction(&Instruction::I32Or);}
    Expr::Not(value)=>{emit_expr(function,value,vars,signatures,strings)?;function.instruction(&Instruction::I32Eqz);}
    Expr::Call(name,args)=>{let(index,arity)=signatures.get(name).ok_or_else(||format!("undefined function `{name}`"))?;if args.len()!=*arity{return Err(format!("function `{name}` expects {arity} argument(s), got {}",args.len()));}for arg in args{emit_expr(function,arg,vars,signatures,strings)?;}function.instruction(&Instruction::Call(*index));}
}Ok(())}

#[cfg(test)]
mod tests{use super::compile;fn validate(source:&str){let wasm=compile(source).unwrap();wasmparser::Validator::new().validate_all(&wasm).unwrap();}
#[test]fn parses_and_compiles_math(){validate("print 2 + 3 * 4");}
#[test]fn compiles_local_variables(){validate("let x = 21\nprint x + x");}
#[test]fn compiles_if_else(){validate("let x = 5\nif x > 3 then\nprint x\nelse\nprint 0\nend");}
#[test]fn compiles_while(){validate("let x = 3\nwhile x > 0\nprint x\nx = x - 1\nend");}
#[test]fn compiles_functions(){validate("function add(a,b)\nreturn a+b\nend\nprint add(2,3)");}
#[test]fn compiles_multiple_function_calls(){validate("function add(a,b)\nreturn a+b\nend\nfunction twice(x)\nreturn add(x,x)\nend\nprint twice(4)");}
#[test]fn compiles_strings(){validate("print \"Hello, LavaScript!\"");}
#[test]fn compiles_escaped_strings(){validate("print \"line1\\nline2\"");}
#[test]fn compiles_booleans_and_logic(){validate("let a = true\nlet b = false\nprint a and not b\nprint a && b\nprint a or b\nprint a || b");}
#[test]fn compiles_modulo_and_unary(){validate("print -10 % 3");}
#[test]fn compiles_break_and_continue(){validate("let x = 5\nwhile x > 0\nx = x - 1\nif x == 3 then\ncontinue\nend\nif x == 1 then\nbreak\nend\nend");}
#[test]fn compiles_for_loop(){validate("for i = 1, 5\nprint i\nend");}
#[test]fn compiles_negative_for_loop(){validate("for i = 5, 1, -1\nprint i\nend");}
#[test]fn compiles_repeat_and_do(){validate("let x = 0\nrepeat\nx = x + 1\nuntil x >= 3\ndo\nprint x\nend");}
#[test]fn compiles_const(){validate("const x = 42\nprint x");}
#[test]fn rejects_const_assignment(){assert!(compile("const x = 1\nx = 2").is_err());}
#[test]fn rejects_loop_control_outside_loop(){assert!(compile("break").is_err());assert!(compile("continue").is_err());}
#[test]fn rejects_bad_for_step(){assert!(compile("for i = 1, 5, 0\nprint i\nend").is_err());}
#[test]fn rejects_string_expression(){assert!(compile("let x = \"hi\"").is_err());}
#[test]fn rejects_undefined_function(){assert!(compile("print missing(1)").is_err());}
#[test]fn rejects_wrong_argument_count(){assert!(compile("function add(a,b)\nreturn a+b\nend\nprint add(1)").is_err());}}
