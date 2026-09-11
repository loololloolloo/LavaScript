#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Number,
    Bool,
    String,
    Array,
    Table,
    Function,
    Nil,
    Unknown,
}

impl ValueType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Bool => "boolean",
            Self::String => "string",
            Self::Array => "array",
            Self::Table => "table",
            Self::Function => "function",
            Self::Nil => "nil",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub params: Vec<ValueType>,
    pub result: ValueType,
}

impl Signature {
    pub fn new(params: Vec<ValueType>, result: ValueType) -> Self {
        Self { params, result }
    }
}
