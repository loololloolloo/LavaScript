use crate::value::{Signature, ValueType};
use std::collections::HashMap;

pub fn signatures() -> HashMap<&'static str, Signature> {
    let mut m = HashMap::new();
    m.insert("len", Signature::new(vec![ValueType::Unknown], ValueType::Number));
    m.insert("type", Signature::new(vec![ValueType::Unknown], ValueType::String));
    m.insert("abs", Signature::new(vec![ValueType::Number], ValueType::Number));
    m.insert("min", Signature::new(vec![ValueType::Number, ValueType::Number], ValueType::Number));
    m.insert("max", Signature::new(vec![ValueType::Number, ValueType::Number], ValueType::Number));
    m.insert("assert", Signature::new(vec![ValueType::Bool], ValueType::Nil));
    m
}

pub fn is_builtin(name: &str) -> bool { signatures().contains_key(name) }
