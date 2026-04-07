use crate::parser::ast::{Operator, Value, WhereExpr};

pub(crate) fn eval_where(
    fields: &[String],
    col_lookup: &dyn Fn(&str) -> Option<usize>,
    expr: &WhereExpr,
) -> bool {
    match expr {
        WhereExpr::Comparison { column, operator, value } => {
            let field = match col_lookup(column) {
                Some(idx) => fields.get(idx).map(|s| s.as_str()).unwrap_or(""),
                None => return false,
            };
            match_value(field, operator, value)
        }
        WhereExpr::In { column, values } => {
            let field = match col_lookup(column) {
                Some(idx) => fields.get(idx).map(|s| s.as_str()).unwrap_or(""),
                None => return false,
            };
            values.iter().any(|v| match_eq(field, v))
        }
        WhereExpr::Between { column, low, high } => {
            let field = match col_lookup(column) {
                Some(idx) => fields.get(idx).map(|s| s.as_str()).unwrap_or(""),
                None => return false,
            };
            !match_cmp(field, low, std::cmp::Ordering::Less)
                && !match_cmp(field, high, std::cmp::Ordering::Greater)
        }
        WhereExpr::And(a, b) => {
            eval_where(fields, col_lookup, a) && eval_where(fields, col_lookup, b)
        }
        WhereExpr::Or(a, b) => {
            eval_where(fields, col_lookup, a) || eval_where(fields, col_lookup, b)
        }
    }
}

fn match_value(field: &str, operator: &Operator, value: &Value) -> bool {
    match operator {
        Operator::Eq => match_eq(field, value),
        Operator::NotEq => !match_eq(field, value),
        Operator::Lt => match_cmp(field, value, std::cmp::Ordering::Less),
        Operator::Lte => match_cmp(field, value, std::cmp::Ordering::Less) || match_eq(field, value),
        Operator::Gt => match_cmp(field, value, std::cmp::Ordering::Greater),
        Operator::Gte => match_cmp(field, value, std::cmp::Ordering::Greater) || match_eq(field, value),
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
