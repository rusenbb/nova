/// Format a floating-point number for display
/// Removes unnecessary decimal places (e.g., 4.0 -> "4")
/// Limits precision to 10 decimal places
pub fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e12 {
        format!("{}", value as i64)
    } else {
        let formatted = format!("{:.10}", value);
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_whole_numbers() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(100.0), "100");
        assert_eq!(format_number(-42.0), "-42");
    }

    #[test]
    fn test_format_decimals() {
        assert_eq!(format_number(3.14159), "3.14159");
        assert_eq!(format_number(0.5), "0.5");
        assert_eq!(format_number(1.200), "1.2");
    }

    #[test]
    fn test_format_precision() {
        let result = format_number(1.0 / 3.0);
        assert!(result.len() <= 12);
    }
}
