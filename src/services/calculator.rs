//! Calculator module for evaluating math expressions

/// Evaluate a math expression and return the result
/// Returns None if the expression is invalid or not a math expression
pub fn evaluate(expr: &str) -> Option<f64> {
    let expr = expr.trim();

    // Skip if empty or doesn't look like math
    if expr.is_empty() {
        return None;
    }

    // Must contain at least one digit
    if !expr.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Try to evaluate using meval
    match meval::eval_str(expr) {
        Ok(result) => {
            // Filter out NaN and infinity
            if result.is_finite() {
                Some(result)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Format a result for display
/// Removes unnecessary decimal places (e.g., 4.0 -> "4")
pub fn format_result(value: f64) -> String {
    super::format::format_number(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        assert_eq!(evaluate("2+2"), Some(4.0));
        assert_eq!(evaluate("10 - 3"), Some(7.0));
        assert_eq!(evaluate("5 * 6"), Some(30.0));
        assert_eq!(evaluate("20 / 4"), Some(5.0));
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(evaluate("2^10"), Some(1024.0));
        assert_eq!(evaluate("sqrt(16)"), Some(4.0));
        assert_eq!(evaluate("(10 + 5) * 2"), Some(30.0));
    }

    #[test]
    fn test_invalid_expressions() {
        assert_eq!(evaluate("hello"), None);
        assert_eq!(evaluate(""), None);
        assert_eq!(evaluate("abc + def"), None);
    }

    #[test]
    fn test_format_result() {
        assert_eq!(format_result(4.0), "4");
        assert_eq!(format_result(1.23456), "1.23456");
        assert_eq!(format_result(100.0), "100");
    }
}
