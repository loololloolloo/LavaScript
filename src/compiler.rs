use std::collections::{HashMap, HashSet};
use wasm_encoder::{BlockType, CodeSection, ConstExpr, DataSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, MemorySection, Module, TypeSection, ValType};
use crate::{checker, lexer, optimizer, parser::{self, CompareOp, Expr, LogicalOp, Op, Program, Stmt}};

const TEMP_LOCALS: u32 = 32;
const I_UNARY: u32 = 0;
const I_BINARY: u32 = 1;
const I_VOID_UNARY: u32 = 2;
const I_VOID_BINARY: u32 = 3;
const I_VOID: u32 = 4;

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let t = lexer::lex(source)?;
    let p = parser::parse(&t)?;
    checker::check(&p)?;
    compile_program(&optimizer::optimize(p))
}

fn compile_program(p: &Program) -> Result<Vec<u8>, String> {
    let mut m = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I32]);
    types.ty().function([ValType::I32, ValType::I32], [ValType::I32]);
    types.ty().function([ValType::I32], []);
    types.ty().function([ValType::I32, ValType::I32], []);
    types.ty().function([], []);

    let mut im = ImportSection::new();
    im.import("lavascript", "print_value", wasm_encoder::EntityType::Function(2));
    im.import("lavascript", "make_string", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "string_concat", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "string_len", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "array_new", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "array_get", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "array_set", wasm_encoder::EntityType::Function(3));
    im.import("lavascript", "array_push", wasm_encoder::EntityType::Function(3));
    im.import("lavascript", "array_pop", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "table_new", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "table_get", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "table_set", wasm_encoder::EntityType::Function(3));
    im.import("lavascript", "value_type", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "abs", wasm_encoder::EntityType::Function(0));
    im.import("lavascript", "min", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "max", wasm_encoder::EntityType::Function(1));
    im.import("lavascript", "assert", wasm_encoder::EntityType::Function(2));

    let import_count = 16u32;
    let mut sigs = HashMap::new();
    for (i, f) in p.functions.iter().enumerate() {
        sigs.insert(f.name.clone(), (import_count + i as u32, f.params.len()));
    }

    let mut fs = FunctionSection::new();
    for f in &p.functions {
        types.ty().function(std::iter::repeat(ValType::I32).take(f.params.len()), [ValType::I32]);
    }
    let main_ty = 5 + p.functions.len() as u32;
    types.ty().function([], []);
    for i in 0..p.functions.len() {
        fs.function(5 + i as u32);
    }
    fs.function(main_ty);

    let mut code = CodeSection::new();
    let mut data = DataSection::new();
    let mut strings = StringTable::new();

    for f in &p.functions {
        let (locals, consts) = collect_locals(&f.body, &f.params)?;
        let mut fun = Function::new(local_decls(&locals, f.params.len()));
        emit_statements(&mut fun, &f.body, &locals, &consts, &sigs, true, &mut strings, 0, 0, locals.len() as u32)?;
        fun.instruction(&Instruction::I32Const(0));
        fun.instruction(&Instruction::End);
        code.function(&fun);
    }

    let (locals, consts) = collect_locals(&p.main, &[])?;
    let mut main = Function::new(local_decls(&locals, 0));
    emit_statements(&mut main, &p.main, &locals, &consts, &sigs, false, &mut strings, 0, 0, locals.len() as u32)?;
    main.instruction(&Instruction::End);
    code.function(&main);

    let mut mem = MemorySection::new();
    mem.memory(wasm_encoder::MemoryType { minimum: 1, maximum: None, memory64: false, shared: false, page_size_log2: None });
    for (o, b) in &strings.entries {
        data.active(0, &ConstExpr::i32_const(*o as i32), b.iter().copied());
    }

    let mut ex = ExportSection::new();
    ex.export("main", ExportKind::Func, import_count + p.functions.len() as u32);
    ex.export("memory", ExportKind::Memory, 0);
    m.section(&types).section(&im).section(&fs).section(&mem).section(&ex).section(&code).section(&data);
    Ok(m.finish())
}

struct StringTable {
    entries: Vec<(u32, Vec<u8>)>,
    next: u32,
}

impl StringTable {
    fn new() -> Self {
        Self { entries: Vec::new(), next: 0 }
    }

    fn intern(&mut self, s: &str) -> u32 {
        if let Some((o, _)) = self.entries.iter().find(|(_, b)| b.as_slice() == s.as_bytes()) {
            return *o;
        }
        let o = self.next;
        self.next += s.len() as u32 + 1;
        self.entries.push((o, s.as_bytes().to_vec()));
        o
    }
}

fn local_decls(loc: &HashMap<String, u32>, params: usize) -> Vec<(u32, ValType)> {
    let n = loc.len().saturating_sub(params) as u32 + TEMP_LOCALS;
    vec![(n, ValType::I32)]
}

fn collect_locals(ss: &[Stmt], params: &[String]) -> Result<(HashMap<String, u32>, HashSet<String>), String> {
    let mut l = HashMap::new();
    let mut c = HashSet::new();
    for (i, n) in params.iter().enumerate() {
        l.insert(n.clone(), i as u32);
    }
    collect(ss, &mut l, &mut c)?;
    Ok((l, c))
}

fn collect(ss: &[Stmt], l: &mut HashMap<String, u32>, c: &mut HashSet<String>) -> Result<(), String> {
    for s in ss {
        match s {
            Stmt::Let(n, _) | Stmt::Const(n, _) => {
                if l.contains_key(n) {
                    return Err(format!("variable `{n}` is already declared"));
                }
                let i = l.len() as u32;
                l.insert(n.clone(), i);
                if matches!(s, Stmt::Const(..)) {
                    c.insert(n.clone());
                }
            }
            Stmt::Assign(n, _) => {
                if !l.contains_key(n) {
                    return Err(format!("undefined variable `{n}`"));
                }
                if c.contains(n) {
                    return Err(format!("cannot assign to constant `{n}`"));
                }
            }
            Stmt::For { name, body, .. } => {
                if !l.contains_key(name) {
                    let i = l.len() as u32;
                    l.insert(name.clone(), i);
                }
                collect(body, l, c)?;
            }
            Stmt::SetIndex(a, b, e) => {
                let _ = (a, b, e);
            }
            Stmt::SetProperty(a, _, e) => {
                let _ = (a, e);
            }
            Stmt::If { then_body, else_body, .. } => {
                collect(then_body, l, c)?;
                collect(else_body, l, c)?;
            }
            Stmt::While { body, .. } | Stmt::Repeat { body, .. } | Stmt::Do { body } => {
                collect(body, l, c)?;
            }
            Stmt::Print(e) | Stmt::Return(e) => {
                let _ = e;
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
    Ok(())
}

fn emit_statements(
    f: &mut Function,
    ss: &[Stmt],
    vars: &HashMap<String, u32>,
    cs: &HashSet<String>,
    sigs: &HashMap<String, (u32, usize)>,
    inf: bool,
    st: &mut StringTable,
    ld: usize,
    cd: usize,
    temp: u32,
) -> Result<(), String> {
    for x in ss {
        match x {
            Stmt::Let(n, e) | Stmt::Const(n, e) => {
                emit_expr(f, e, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::LocalSet(*vars.get(n).unwrap()));
            }
            Stmt::Assign(n, e) => {
                emit_expr(f, e, vars, sigs, st, temp, 0)?;
                if cs.contains(n) {
                    return Err(format!("cannot assign to constant `{n}`"));
                }
                f.instruction(&Instruction::LocalSet(*vars.get(n).unwrap()));
            }
            Stmt::SetIndex(a, i, v) => {
                emit_expr(f, a, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::LocalSet(temp));
                emit_expr(f, i, vars, sigs, st, temp + 1, 1)?;
                emit_expr(f, v, vars, sigs, st, temp + 1, 1)?;
                f.instruction(&Instruction::LocalGet(temp));
                f.instruction(&Instruction::LocalGet(temp + 1));
                f.instruction(&Instruction::LocalGet(temp + 2));
                f.instruction(&Instruction::Call(6));
            }
            Stmt::SetProperty(a, k, v) => {
                emit_expr(f, a, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::LocalSet(temp));
                emit_string(f, k, st);
                emit_expr(f, v, vars, sigs, st, temp + 1, 1)?;
                f.instruction(&Instruction::LocalGet(temp));
                f.instruction(&Instruction::LocalGet(temp + 1));
                f.instruction(&Instruction::LocalGet(temp + 2));
                f.instruction(&Instruction::Call(11));
            }
            Stmt::Print(e) => {
                emit_expr(f, e, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::Call(0));
            }
            Stmt::Return(e) => {
                if !inf {
                    return Err("return outside function".into());
                }
                emit_expr(f, e, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::Return);
            }
            Stmt::If { condition, then_body, else_body } => {
                emit_expr(f, condition, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::If(BlockType::Empty));
                emit_statements(f, then_body, vars, cs, sigs, inf, st, ld, cd, temp)?;
                if !else_body.is_empty() {
                    f.instruction(&Instruction::Else);
                    emit_statements(f, else_body, vars, cs, sigs, inf, st, ld, cd, temp)?;
                }
                f.instruction(&Instruction::End);
            }
            Stmt::While { condition, body } => {
                f.instruction(&Instruction::Block(BlockType::Empty));
                f.instruction(&Instruction::Loop(BlockType::Empty));
                emit_expr(f, condition, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::I32Eqz);
                f.instruction(&Instruction::BrIf(1));
                emit_statements(f, body, vars, cs, sigs, inf, st, ld + 1, 0, temp)?;
                f.instruction(&Instruction::Br(0));
                f.instruction(&Instruction::End);
                f.instruction(&Instruction::End);
            }
            Stmt::For { name, start, end, step, body } => {
                let sv = match step {
                    Expr::Number(n) if *n != 0 => *n,
                    _ => return Err("for step must be a non-zero numeric literal".into()),
                };
                let i = *vars.get(name).unwrap();
                emit_expr(f, start, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::LocalSet(i));
                f.instruction(&Instruction::Block(BlockType::Empty));
                f.instruction(&Instruction::Loop(BlockType::Empty));
                f.instruction(&Instruction::LocalGet(i));
                emit_expr(f, end, vars, sigs, st, temp, 0)?;
                if sv > 0 {
                    f.instruction(&Instruction::I32GtS);
                } else {
                    f.instruction(&Instruction::I32LtS);
                }
                f.instruction(&Instruction::I32Eqz);
                f.instruction(&Instruction::BrIf(1));
                emit_statements(f, body, vars, cs, sigs, inf, st, ld + 1, 0, temp)?;
                f.instruction(&Instruction::LocalGet(i));
                f.instruction(&Instruction::I32Const(sv));
                f.instruction(&Instruction::I32Add);
                f.instruction(&Instruction::LocalSet(i));
                f.instruction(&Instruction::Br(0));
                f.instruction(&Instruction::End);
                f.instruction(&Instruction::End);
            }
            Stmt::Repeat { body, condition } => {
                f.instruction(&Instruction::Block(BlockType::Empty));
                f.instruction(&Instruction::Loop(BlockType::Empty));
                emit_statements(f, body, vars, cs, sigs, inf, st, ld + 1, 1, temp)?;
                emit_expr(f, condition, vars, sigs, st, temp, 0)?;
                f.instruction(&Instruction::BrIf(1));
                f.instruction(&Instruction::Br(0));
                f.instruction(&Instruction::End);
                f.instruction(&Instruction::End);
            }
            Stmt::Do { body } => {
                emit_statements(f, body, vars, cs, sigs, inf, st, ld, cd, temp)?;
            }
            Stmt::Break => {
                if ld == 0 {
                    return Err("break outside loop".into());
                }
                f.instruction(&Instruction::Br(ld as u32));
            }
            Stmt::Continue => {
                if ld == 0 {
                    return Err("continue outside loop".into());
                }
                f.instruction(&Instruction::Br(cd as u32));
            }
        }
    }
    Ok(())
}

fn emit_string(f: &mut Function, s: &str, st: &mut StringTable) {
    let o = st.intern(s);
    f.instruction(&Instruction::I32Const(o as i32));
    f.instruction(&Instruction::I32Const(s.len() as i32));
    f.instruction(&Instruction::Call(1));
}

fn emit_expr(
    f: &mut Function,
    e: &Expr,
    v: &HashMap<String, u32>,
    s: &HashMap<String, (u32, usize)>,
    st: &mut StringTable,
    temp: u32,
    depth: u32,
) -> Result<(), String> {
    if depth >= TEMP_LOCALS {
        return Err("expression nesting is too deep".into());
    }

    match e {
        Expr::Number(n) => {
            f.instruction(&Instruction::I32Const(*n));
        }
        Expr::Bool(b) => {
            f.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
        }
        Expr::Nil => {
            f.instruction(&Instruction::I32Const(0));
        }
        Expr::String(x) => {
            emit_string(f, x, st);
        }
        Expr::Variable(n) => {
            let local = *v.get(n).ok_or_else(|| format!("undefined variable `{n}`"))?;
            f.instruction(&Instruction::LocalGet(local));
        }
        Expr::Binary(a, o, b) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            emit_expr(f, b, v, s, st, temp + 1, depth + 1)?;
            match o {
                Op::Add => f.instruction(&Instruction::I32Add),
                Op::Sub => f.instruction(&Instruction::I32Sub),
                Op::Mul => f.instruction(&Instruction::I32Mul),
                Op::Div => f.instruction(&Instruction::I32DivS),
                Op::Mod => f.instruction(&Instruction::I32RemS),
            };
        }
        Expr::Compare(a, o, b) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            emit_expr(f, b, v, s, st, temp + 1, depth + 1)?;
            match o {
                CompareOp::Eq => f.instruction(&Instruction::I32Eq),
                CompareOp::Ne => f.instruction(&Instruction::I32Ne),
                CompareOp::Lt => f.instruction(&Instruction::I32LtS),
                CompareOp::Le => f.instruction(&Instruction::I32LeS),
                CompareOp::Gt => f.instruction(&Instruction::I32GtS),
                CompareOp::Ge => f.instruction(&Instruction::I32GeS),
            };
        }
        Expr::Logical(a, o, b) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            emit_expr(f, b, v, s, st, temp + 1, depth + 1)?;
            match o {
                LogicalOp::And => f.instruction(&Instruction::I32And),
                LogicalOp::Or => f.instruction(&Instruction::I32Or),
            };
        }
        Expr::Not(a) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            f.instruction(&Instruction::I32Eqz);
        }
        Expr::Call(n, args) => {
            let (i, a) = s.get(n).copied().ok_or_else(|| format!("undefined function `{n}`"))?;
            if args.len() != a {
                return Err(format!("function `{n}` expects {a} argument(s), got {}", args.len()));
            }
            for x in args {
                emit_expr(f, x, v, s, st, temp, depth + 1)?;
            }
            if crate::stdlib::is_builtin(n) {
                match n.as_str() {
                    "len" => f.instruction(&Instruction::Call(3)),
                    "type" => f.instruction(&Instruction::Call(12)),
                    "abs" => f.instruction(&Instruction::Call(13)),
                    "min" => f.instruction(&Instruction::Call(14)),
                    "max" => f.instruction(&Instruction::Call(15)),
                    "assert" => f.instruction(&Instruction::Call(16)),
                    _ => return Err(format!("unknown builtin `{n}`")),
                };
            } else {
                f.instruction(&Instruction::Call(i));
            }
        }
        Expr::Array(xs) => {
            f.instruction(&Instruction::I32Const(xs.len() as i32));
            f.instruction(&Instruction::Call(4));
            f.instruction(&Instruction::LocalSet(temp));
            for (i, x) in xs.iter().enumerate() {
                emit_expr(f, x, v, s, st, temp + 1, depth + 1)?;
                f.instruction(&Instruction::LocalSet(temp + 1));
                f.instruction(&Instruction::LocalGet(temp));
                f.instruction(&Instruction::I32Const(i as i32));
                f.instruction(&Instruction::LocalGet(temp + 1));
                f.instruction(&Instruction::Call(6));
            }
            f.instruction(&Instruction::LocalGet(temp));
        }
        Expr::Table(xs) => {
            f.instruction(&Instruction::I32Const(xs.len() as i32));
            f.instruction(&Instruction::Call(9));
            f.instruction(&Instruction::LocalSet(temp));
            for (k, x) in xs {
                emit_string(f, k, st);
                f.instruction(&Instruction::LocalSet(temp + 1));
                emit_expr(f, x, v, s, st, temp + 2, depth + 1)?;
                f.instruction(&Instruction::LocalSet(temp + 2));
                f.instruction(&Instruction::LocalGet(temp));
                f.instruction(&Instruction::LocalGet(temp + 1));
                f.instruction(&Instruction::LocalGet(temp + 2));
                f.instruction(&Instruction::Call(11));
            }
            f.instruction(&Instruction::LocalGet(temp));
        }
        Expr::Index(a, i) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            f.instruction(&Instruction::LocalSet(temp));
            emit_expr(f, i, v, s, st, temp + 1, depth + 1)?;
            f.instruction(&Instruction::LocalGet(temp));
            f.instruction(&Instruction::LocalGet(temp + 1));
            f.instruction(&Instruction::Call(5));
        }
        Expr::Property(a, k) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            f.instruction(&Instruction::LocalSet(temp));
            emit_string(f, k, st);
            f.instruction(&Instruction::LocalGet(temp));
            f.instruction(&Instruction::LocalGet(temp + 1));
            f.instruction(&Instruction::Call(10));
        }
        Expr::Method(a, n, args) => {
            emit_expr(f, a, v, s, st, temp, depth + 1)?;
            f.instruction(&Instruction::LocalSet(temp));
            for x in args {
                emit_expr(f, x, v, s, st, temp + 1, depth + 1)?;
                f.instruction(&Instruction::LocalSet(temp + 1));
            }
            match n.as_str() {
                "push" => {
                    if args.len() != 1 {
                        return Err("push expects one argument".into());
                    }
                    f.instruction(&Instruction::LocalGet(temp));
                    f.instruction(&Instruction::LocalGet(temp + 1));
                    f.instruction(&Instruction::I32Const(0));
                    f.instruction(&Instruction::Call(7));
                    f.instruction(&Instruction::I32Const(0));
                }
                "pop" => {
                    f.instruction(&Instruction::LocalGet(temp));
                    f.instruction(&Instruction::Call(8));
                }
                "contains" => {
                    f.instruction(&Instruction::LocalGet(temp));
                    f.instruction(&Instruction::LocalGet(temp + 1));
                    f.instruction(&Instruction::Call(5));
                }
                "get" => {
                    f.instruction(&Instruction::LocalGet(temp));
                    f.instruction(&Instruction::LocalGet(temp + 1));
                    f.instruction(&Instruction::Call(10));
                }
                "set" => {
                    f.instruction(&Instruction::LocalGet(temp));
                    f.instruction(&Instruction::LocalGet(temp + 1));
                    f.instruction(&Instruction::LocalGet(temp + 2));
                    f.instruction(&Instruction::Call(11));
                    f.instruction(&Instruction::I32Const(0));
                }
                "upper" | "lower" => {
                    return Err(format!("string method `{n}` is not yet host-backed"));
                }
                _ => {
                    return Err(format!("unknown method `{n}`"));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compile;

    fn v(s: &str) {
        let w = compile(s).unwrap();
        wasmparser::Validator::new().validate_all(&w).unwrap();
    }

    #[test]
    fn values() {
        v("let a=[1,2,3]\nprint a[1]");
    }

    #[test]
    fn tables() {
        v("let t={name:'lava',x:7}\nprint t.x");
    }

    #[test]
    fn strings() {
        v("print 'hello'\nprint len('abc')");
    }

    #[test]
    fn old_math() {
        v("print 2+3*4");
    }
}
