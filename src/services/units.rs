//! Unit converter module for converting between measurement units

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// A conversion result
#[derive(Debug, Clone)]
pub struct Conversion {
    pub from_value: f64,
    pub from_unit: String,
    pub to_value: f64,
    pub to_unit: String,
}

impl Conversion {
    /// Format as display string: "10 km = 6.21 miles"
    pub fn display(&self) -> String {
        format!(
            "{} {} = {} {}",
            format_number(self.from_value),
            self.from_unit,
            format_number(self.to_value),
            self.to_unit
        )
    }

    /// Get just the result value formatted
    pub fn result(&self) -> String {
        format!("{} {}", format_number(self.to_value), self.to_unit)
    }
}

/// Format a number nicely (remove trailing zeros)
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e12 {
        format!("{}", n as i64)
    } else if n.abs() < 0.01 || n.abs() >= 1e6 {
        // Scientific notation for very small/large numbers
        format!("{:.4e}", n)
    } else {
        // Normal formatting, trim trailing zeros
        let formatted = format!("{:.6}", n);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}

/// Try to parse and convert a unit expression
/// Examples: "10km to miles", "32 f to c", "5 kg to lb"
pub fn convert(query: &str) -> Option<Conversion> {
    let query = query.trim().to_lowercase();

    // Pattern: <number> <unit> to <unit>
    // or: <number><unit> to <unit>
    let parts: Vec<&str> = query.split(" to ").collect();
    if parts.len() != 2 {
        return None;
    }

    let from_part = parts[0].trim();
    let to_unit_raw = parts[1].trim();

    // Parse the "from" part: number + unit
    let (value, from_unit_raw) = parse_value_unit(from_part)?;

    // Normalize unit names
    let from_unit = normalize_unit(from_unit_raw)?;
    let to_unit = normalize_unit(to_unit_raw)?;

    // Check if conversion is possible (same category)
    let from_category = get_category(&from_unit)?;
    let to_category = get_category(&to_unit)?;

    if from_category != to_category {
        return None; // Can't convert between different categories
    }

    // Perform conversion
    let to_value = convert_value(value, &from_unit, &to_unit)?;

    Some(Conversion {
        from_value: value,
        from_unit: get_display_name(&from_unit),
        to_value,
        to_unit: get_display_name(&to_unit),
    })
}

/// Parse "10km" or "10 km" into (10.0, "km")
fn parse_value_unit(s: &str) -> Option<(f64, &str)> {
    let s = s.trim();

    // Find where the number ends
    let mut num_end = 0;
    let mut has_digit = false;
    let mut has_decimal = false;
    let mut has_sign = false;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() {
            has_digit = true;
            num_end = i + 1;
        } else if c == '.' && !has_decimal {
            has_decimal = true;
            num_end = i + 1;
        } else if (c == '-' || c == '+') && !has_sign && !has_digit {
            has_sign = true;
            num_end = i + 1;
        } else if c.is_whitespace() && has_digit {
            break;
        } else if has_digit {
            break;
        }
    }

    if !has_digit || num_end == 0 {
        return None;
    }

    let num_str = &s[..num_end];
    let unit_str = s[num_end..].trim();

    if unit_str.is_empty() {
        return None;
    }

    let value: f64 = num_str.parse().ok()?;
    Some((value, unit_str))
}

/// Unit categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Length,
    Weight,
    Volume,
    Temperature,
    Area,
    Speed,
    Time,
    Data,
}

/// Normalize unit aliases to canonical form
fn normalize_unit(s: &str) -> Option<String> {
    let s = s.trim().to_lowercase();
    UNIT_ALIASES.get(s.as_str()).map(|u| u.to_string())
}

/// Get display name for a unit
fn get_display_name(canonical: &str) -> String {
    UNIT_DISPLAY
        .get(canonical)
        .map(|s| s.to_string())
        .unwrap_or_else(|| canonical.to_string())
}

/// Get the category for a canonical unit
fn get_category(canonical: &str) -> Option<Category> {
    UNIT_CATEGORIES.get(canonical).copied()
}

/// Convert value between units in the same category
fn convert_value(value: f64, from: &str, to: &str) -> Option<f64> {
    // Special case for temperature
    if get_category(from)? == Category::Temperature {
        return convert_temperature(value, from, to);
    }

    // For other units, convert via base unit
    let from_factor = CONVERSION_FACTORS.get(from)?;
    let to_factor = CONVERSION_FACTORS.get(to)?;

    // Convert to base unit, then to target unit
    Some(value * from_factor / to_factor)
}

/// Convert temperature (special case - not multiplicative)
fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    // First convert to Kelvin
    let kelvin = match from {
        "celsius" => value + 273.15,
        "fahrenheit" => (value - 32.0) * 5.0 / 9.0 + 273.15,
        "kelvin" => value,
        _ => return None,
    };

    // Then convert from Kelvin to target
    let result = match to {
        "celsius" => kelvin - 273.15,
        "fahrenheit" => (kelvin - 273.15) * 9.0 / 5.0 + 32.0,
        "kelvin" => kelvin,
        _ => return None,
    };

    Some(result)
}

// Unit aliases mapping to canonical names
static UNIT_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Length
    m.insert("m", "meter");
    m.insert("meter", "meter");
    m.insert("meters", "meter");
    m.insert("metre", "meter");
    m.insert("metres", "meter");
    m.insert("km", "kilometer");
    m.insert("kilometer", "kilometer");
    m.insert("kilometers", "kilometer");
    m.insert("kilometre", "kilometer");
    m.insert("kilometres", "kilometer");
    m.insert("cm", "centimeter");
    m.insert("centimeter", "centimeter");
    m.insert("centimeters", "centimeter");
    m.insert("centimetre", "centimeter");
    m.insert("mm", "millimeter");
    m.insert("millimeter", "millimeter");
    m.insert("millimeters", "millimeter");
    m.insert("millimetre", "millimeter");
    m.insert("mi", "mile");
    m.insert("mile", "mile");
    m.insert("miles", "mile");
    m.insert("yd", "yard");
    m.insert("yard", "yard");
    m.insert("yards", "yard");
    m.insert("ft", "foot");
    m.insert("foot", "foot");
    m.insert("feet", "foot");
    m.insert("in", "inch");
    m.insert("inch", "inch");
    m.insert("inches", "inch");
    m.insert("\"", "inch");
    m.insert("nm", "nautical_mile");
    m.insert("nmi", "nautical_mile");
    m.insert("nautical mile", "nautical_mile");
    m.insert("nautical miles", "nautical_mile");

    // Weight/Mass
    m.insert("kg", "kilogram");
    m.insert("kilogram", "kilogram");
    m.insert("kilograms", "kilogram");
    m.insert("kilo", "kilogram");
    m.insert("kilos", "kilogram");
    m.insert("g", "gram");
    m.insert("gram", "gram");
    m.insert("grams", "gram");
    m.insert("mg", "milligram");
    m.insert("milligram", "milligram");
    m.insert("milligrams", "milligram");
    m.insert("lb", "pound");
    m.insert("lbs", "pound");
    m.insert("pound", "pound");
    m.insert("pounds", "pound");
    m.insert("oz", "ounce");
    m.insert("ounce", "ounce");
    m.insert("ounces", "ounce");
    m.insert("ton", "metric_ton");
    m.insert("tons", "metric_ton");
    m.insert("tonne", "metric_ton");
    m.insert("tonnes", "metric_ton");
    m.insert("t", "metric_ton");
    m.insert("st", "stone");
    m.insert("stone", "stone");
    m.insert("stones", "stone");

    // Volume
    m.insert("l", "liter");
    m.insert("liter", "liter");
    m.insert("liters", "liter");
    m.insert("litre", "liter");
    m.insert("litres", "liter");
    m.insert("ml", "milliliter");
    m.insert("milliliter", "milliliter");
    m.insert("milliliters", "milliliter");
    m.insert("millilitre", "milliliter");
    m.insert("gal", "gallon");
    m.insert("gallon", "gallon");
    m.insert("gallons", "gallon");
    m.insert("qt", "quart");
    m.insert("quart", "quart");
    m.insert("quarts", "quart");
    m.insert("pt", "pint");
    m.insert("pint", "pint");
    m.insert("pints", "pint");
    m.insert("cup", "cup");
    m.insert("cups", "cup");
    m.insert("fl oz", "fluid_ounce");
    m.insert("floz", "fluid_ounce");
    m.insert("fluid ounce", "fluid_ounce");
    m.insert("fluid ounces", "fluid_ounce");
    m.insert("tbsp", "tablespoon");
    m.insert("tablespoon", "tablespoon");
    m.insert("tablespoons", "tablespoon");
    m.insert("tsp", "teaspoon");
    m.insert("teaspoon", "teaspoon");
    m.insert("teaspoons", "teaspoon");

    // Temperature
    m.insert("c", "celsius");
    m.insert("celsius", "celsius");
    m.insert("f", "fahrenheit");
    m.insert("fahrenheit", "fahrenheit");
    m.insert("k", "kelvin");
    m.insert("kelvin", "kelvin");

    // Area
    m.insert("sqm", "square_meter");
    m.insert("m2", "square_meter");
    m.insert("m²", "square_meter");
    m.insert("square meter", "square_meter");
    m.insert("square meters", "square_meter");
    m.insert("sqft", "square_foot");
    m.insert("ft2", "square_foot");
    m.insert("ft²", "square_foot");
    m.insert("square foot", "square_foot");
    m.insert("square feet", "square_foot");
    m.insert("sqkm", "square_kilometer");
    m.insert("km2", "square_kilometer");
    m.insert("km²", "square_kilometer");
    m.insert("sqmi", "square_mile");
    m.insert("mi2", "square_mile");
    m.insert("mi²", "square_mile");
    m.insert("acre", "acre");
    m.insert("acres", "acre");
    m.insert("ha", "hectare");
    m.insert("hectare", "hectare");
    m.insert("hectares", "hectare");

    // Speed
    m.insert("mph", "miles_per_hour");
    m.insert("mi/h", "miles_per_hour");
    m.insert("kph", "kilometers_per_hour");
    m.insert("kmh", "kilometers_per_hour");
    m.insert("km/h", "kilometers_per_hour");
    m.insert("m/s", "meters_per_second");
    m.insert("mps", "meters_per_second");
    m.insert("knot", "knot");
    m.insert("knots", "knot");
    m.insert("kn", "knot");

    // Time
    m.insert("s", "second");
    m.insert("sec", "second");
    m.insert("second", "second");
    m.insert("seconds", "second");
    m.insert("min", "minute");
    m.insert("minute", "minute");
    m.insert("minutes", "minute");
    m.insert("h", "hour");
    m.insert("hr", "hour");
    m.insert("hour", "hour");
    m.insert("hours", "hour");
    m.insert("day", "day");
    m.insert("days", "day");
    m.insert("week", "week");
    m.insert("weeks", "week");
    m.insert("month", "month");
    m.insert("months", "month");
    m.insert("year", "year");
    m.insert("years", "year");
    m.insert("yr", "year");

    // Data
    m.insert("b", "byte");
    m.insert("byte", "byte");
    m.insert("bytes", "byte");
    m.insert("kb", "kilobyte");
    m.insert("kilobyte", "kilobyte");
    m.insert("kilobytes", "kilobyte");
    m.insert("mb", "megabyte");
    m.insert("megabyte", "megabyte");
    m.insert("megabytes", "megabyte");
    m.insert("gb", "gigabyte");
    m.insert("gigabyte", "gigabyte");
    m.insert("gigabytes", "gigabyte");
    m.insert("tb", "terabyte");
    m.insert("terabyte", "terabyte");
    m.insert("terabytes", "terabyte");
    m.insert("kib", "kibibyte");
    m.insert("kibibyte", "kibibyte");
    m.insert("mib", "mebibyte");
    m.insert("mebibyte", "mebibyte");
    m.insert("gib", "gibibyte");
    m.insert("gibibyte", "gibibyte");
    m.insert("tib", "tebibyte");
    m.insert("tebibyte", "tebibyte");

    m
});

// Display names for canonical units
static UNIT_DISPLAY: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert("meter", "m");
    m.insert("kilometer", "km");
    m.insert("centimeter", "cm");
    m.insert("millimeter", "mm");
    m.insert("mile", "mi");
    m.insert("yard", "yd");
    m.insert("foot", "ft");
    m.insert("inch", "in");
    m.insert("nautical_mile", "nmi");

    m.insert("kilogram", "kg");
    m.insert("gram", "g");
    m.insert("milligram", "mg");
    m.insert("pound", "lb");
    m.insert("ounce", "oz");
    m.insert("metric_ton", "t");
    m.insert("stone", "st");

    m.insert("liter", "L");
    m.insert("milliliter", "mL");
    m.insert("gallon", "gal");
    m.insert("quart", "qt");
    m.insert("pint", "pt");
    m.insert("cup", "cup");
    m.insert("fluid_ounce", "fl oz");
    m.insert("tablespoon", "tbsp");
    m.insert("teaspoon", "tsp");

    m.insert("celsius", "°C");
    m.insert("fahrenheit", "°F");
    m.insert("kelvin", "K");

    m.insert("square_meter", "m²");
    m.insert("square_foot", "ft²");
    m.insert("square_kilometer", "km²");
    m.insert("square_mile", "mi²");
    m.insert("acre", "acre");
    m.insert("hectare", "ha");

    m.insert("miles_per_hour", "mph");
    m.insert("kilometers_per_hour", "km/h");
    m.insert("meters_per_second", "m/s");
    m.insert("knot", "knot");

    m.insert("second", "s");
    m.insert("minute", "min");
    m.insert("hour", "hr");
    m.insert("day", "day");
    m.insert("week", "week");
    m.insert("month", "month");
    m.insert("year", "yr");

    m.insert("byte", "B");
    m.insert("kilobyte", "KB");
    m.insert("megabyte", "MB");
    m.insert("gigabyte", "GB");
    m.insert("terabyte", "TB");
    m.insert("kibibyte", "KiB");
    m.insert("mebibyte", "MiB");
    m.insert("gibibyte", "GiB");
    m.insert("tebibyte", "TiB");

    m
});

// Unit categories
static UNIT_CATEGORIES: Lazy<HashMap<&'static str, Category>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Length
    for u in [
        "meter",
        "kilometer",
        "centimeter",
        "millimeter",
        "mile",
        "yard",
        "foot",
        "inch",
        "nautical_mile",
    ] {
        m.insert(u, Category::Length);
    }

    // Weight
    for u in [
        "kilogram",
        "gram",
        "milligram",
        "pound",
        "ounce",
        "metric_ton",
        "stone",
    ] {
        m.insert(u, Category::Weight);
    }

    // Volume
    for u in [
        "liter",
        "milliliter",
        "gallon",
        "quart",
        "pint",
        "cup",
        "fluid_ounce",
        "tablespoon",
        "teaspoon",
    ] {
        m.insert(u, Category::Volume);
    }

    // Temperature
    for u in ["celsius", "fahrenheit", "kelvin"] {
        m.insert(u, Category::Temperature);
    }

    // Area
    for u in [
        "square_meter",
        "square_foot",
        "square_kilometer",
        "square_mile",
        "acre",
        "hectare",
    ] {
        m.insert(u, Category::Area);
    }

    // Speed
    for u in [
        "miles_per_hour",
        "kilometers_per_hour",
        "meters_per_second",
        "knot",
    ] {
        m.insert(u, Category::Speed);
    }

    // Time
    for u in ["second", "minute", "hour", "day", "week", "month", "year"] {
        m.insert(u, Category::Time);
    }

    // Data
    for u in [
        "byte", "kilobyte", "megabyte", "gigabyte", "terabyte", "kibibyte", "mebibyte", "gibibyte",
        "tebibyte",
    ] {
        m.insert(u, Category::Data);
    }

    m
});

// Conversion factors to base unit (meter for length, gram for weight, etc.)
static CONVERSION_FACTORS: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Length (base: meter)
    m.insert("meter", 1.0);
    m.insert("kilometer", 1000.0);
    m.insert("centimeter", 0.01);
    m.insert("millimeter", 0.001);
    m.insert("mile", 1609.344);
    m.insert("yard", 0.9144);
    m.insert("foot", 0.3048);
    m.insert("inch", 0.0254);
    m.insert("nautical_mile", 1852.0);

    // Weight (base: gram)
    m.insert("kilogram", 1000.0);
    m.insert("gram", 1.0);
    m.insert("milligram", 0.001);
    m.insert("pound", 453.592);
    m.insert("ounce", 28.3495);
    m.insert("metric_ton", 1_000_000.0);
    m.insert("stone", 6350.29);

    // Volume (base: liter)
    m.insert("liter", 1.0);
    m.insert("milliliter", 0.001);
    m.insert("gallon", 3.78541); // US gallon
    m.insert("quart", 0.946353);
    m.insert("pint", 0.473176);
    m.insert("cup", 0.236588);
    m.insert("fluid_ounce", 0.0295735);
    m.insert("tablespoon", 0.0147868);
    m.insert("teaspoon", 0.00492892);

    // Area (base: square meter)
    m.insert("square_meter", 1.0);
    m.insert("square_foot", 0.092903);
    m.insert("square_kilometer", 1_000_000.0);
    m.insert("square_mile", 2_589_988.0);
    m.insert("acre", 4046.86);
    m.insert("hectare", 10000.0);

    // Speed (base: m/s)
    m.insert("meters_per_second", 1.0);
    m.insert("kilometers_per_hour", 1.0 / 3.6);
    m.insert("miles_per_hour", 0.44704);
    m.insert("knot", 0.514444);

    // Time (base: second)
    m.insert("second", 1.0);
    m.insert("minute", 60.0);
    m.insert("hour", 3600.0);
    m.insert("day", 86400.0);
    m.insert("week", 604800.0);
    m.insert("month", 2_629_746.0); // Average month
    m.insert("year", 31_556_952.0); // Average year

    // Data (base: byte)
    m.insert("byte", 1.0);
    m.insert("kilobyte", 1000.0);
    m.insert("megabyte", 1_000_000.0);
    m.insert("gigabyte", 1_000_000_000.0);
    m.insert("terabyte", 1_000_000_000_000.0);
    m.insert("kibibyte", 1024.0);
    m.insert("mebibyte", 1_048_576.0);
    m.insert("gibibyte", 1_073_741_824.0);
    m.insert("tebibyte", 1_099_511_627_776.0);

    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_km_to_miles() {
        let result = convert("10km to miles").unwrap();
        assert!((result.to_value - 6.21371).abs() < 0.001);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        let result = convert("32f to c").unwrap();
        assert!((result.to_value - 0.0).abs() < 0.001);

        let result = convert("212f to c").unwrap();
        assert!((result.to_value - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_kg_to_lb() {
        let result = convert("1kg to lb").unwrap();
        assert!((result.to_value - 2.20462).abs() < 0.001);
    }

    #[test]
    fn test_parse_value_unit() {
        assert_eq!(parse_value_unit("10km"), Some((10.0, "km")));
        assert_eq!(parse_value_unit("10 km"), Some((10.0, "km")));
        assert_eq!(parse_value_unit("3.14 m"), Some((3.14, "m")));
        assert_eq!(parse_value_unit("-5 c"), Some((-5.0, "c")));
    }

    #[test]
    fn test_invalid_conversion() {
        // Can't convert between different categories
        assert!(convert("10km to kg").is_none());
    }
}
