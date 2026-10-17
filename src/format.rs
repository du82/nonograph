// 浮点数十进制格式化算法
pub(crate) fn format_count(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let magnitude = value.abs();
    let rendered = if magnitude.fract() == 0.0 && magnitude < 1e15 {
        format!("{}", magnitude as u64)
    } else {
        format!("{}", magnitude)
    };
    let (int_part, frac_part) = rendered.split_once('.').unzip();
    let int_part = int_part.unwrap_or(&rendered);

    let mut out = String::new();
    if value.is_sign_negative() && value != 0.0 {
        out.push('-');
    }
    for (i, ch) in int_part.chars().enumerate() {
        if i > 0 && int_part.len() > 4 && (int_part.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(frac) = frac_part {
        out.push('.');
        out.push_str(frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_specified_progression() {
        assert_eq!(format_count(0.009), "0.009");
        assert_eq!(format_count(0.09), "0.09");
        assert_eq!(format_count(0.9), "0.9");
        assert_eq!(format_count(9.0), "9");
        assert_eq!(format_count(99.0), "99");
        assert_eq!(format_count(999.0), "999");
        assert_eq!(format_count(9999.0), "9999");
        assert_eq!(format_count(99_999.0), "99,999");
        assert_eq!(format_count(999_999.0), "999,999");
        assert_eq!(format_count(9_999_999.0), "9,999,999");
    }

    #[test]
    fn four_vs_five_digit_boundary() {
        assert_eq!(format_count(1000.0), "1000");
        assert_eq!(format_count(9999.0), "9999");
        assert_eq!(format_count(10_000.0), "10,000");
    }

    #[test]
    fn larger_numbers_group_in_threes() {
        assert_eq!(format_count(1_234_567.0), "1,234,567");
        assert_eq!(format_count(123_456_789.0), "123,456,789");
    }

    #[test]
    fn negatives_and_fractions() {
        assert_eq!(format_count(-9999.0), "-9999");
        assert_eq!(format_count(-10_000.0), "-10,000");
        assert_eq!(format_count(-0.009), "-0.009");
        assert_eq!(format_count(1234.5), "1234.5");
        assert_eq!(format_count(12_345.5), "12,345.5");
    }

    #[test]
    fn non_finite_is_passed_through() {
        assert_eq!(format_count(f64::NAN), "NaN");
        assert_eq!(format_count(f64::INFINITY), "inf");
    }
}
