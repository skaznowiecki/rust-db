use crate::parser::ast::{Operator, Value};

pub(crate) fn match_value(field: &str, operator: &Operator, value: &Value) -> bool {
    match operator {
        Operator::Eq => match_eq(field, value),
        Operator::NotEq => !match_eq(field, value),
        Operator::Lt => match_cmp(field, value, std::cmp::Ordering::Less),
        Operator::Gt => match_cmp(field, value, std::cmp::Ordering::Greater),
        Operator::Like => match_like(field, value, false),
        Operator::ILike => match_like(field, value, true),
    }
}

fn match_eq(field: &str, value: &Value) -> bool {
    match value {
        Value::Number(n) => field == n.to_string(),
        Value::String(s) => field == s,
        Value::Bool(b) => field == b.to_string(),
        Value::Null => field.is_empty(),
    }
}

fn match_cmp(field: &str, value: &Value, expected: std::cmp::Ordering) -> bool {
    match value {
        Value::Number(n) => {
            if let Ok(f) = field.parse::<i64>() {
                f.cmp(n) == expected
            } else {
                false
            }
        }
        Value::String(s) => field.cmp(s.as_str()) == expected,
        _ => false,
    }
}

fn match_like(field: &str, value: &Value, case_insensitive: bool) -> bool {
    let pattern = match value {
        Value::String(s) => s,
        _ => return false,
    };

    let (field, pattern) = if case_insensitive {
        (field.to_lowercase(), pattern.to_lowercase())
    } else {
        (field.to_string(), pattern.clone())
    };

    let starts = pattern.starts_with('%');
    let ends = pattern.ends_with('%');

    match (starts, ends) {
        (true, true) => {
            let inner = &pattern[1..pattern.len() - 1];
            field.contains(inner)
        }
        (true, false) => {
            let suffix = &pattern[1..];
            field.ends_with(suffix)
        }
        (false, true) => {
            let prefix = &pattern[..pattern.len() - 1];
            field.starts_with(prefix)
        }
        (false, false) => field == pattern,
    }
}
