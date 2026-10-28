const KIB: f64 = 1_024.0;
const MIB: f64 = 1_024.0 * 1_024.0;
const GIB: f64 = 1_024.0 * 1_024.0 * 1_024.0;

// e.g. "3.42 MB".
pub(crate) fn size(bytes: u64) -> String {
    let b = bytes as f64;
    if bytes < 1_024 {
        format!("{} B", bytes)
    } else if b < MIB {
        format!("{:.2} KB", b / KIB)
    } else if b < GIB {
        format!("{:.2} MB", b / MIB)
    } else {
        format!("{:.2} GB", b / GIB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_have_no_decimals() {
        assert_eq!(size(0), "0 B");
        assert_eq!(size(1), "1 B");
        assert_eq!(size(1_023), "1023 B");
    }

    #[test]
    fn unit_boundaries() {
        assert_eq!(size(1_024), "1.00 KB");
        assert_eq!(size(1_048_576), "1.00 MB");
        assert_eq!(size(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn two_decimal_precision() {
        assert_eq!(size(1_536), "1.50 KB");
        // 3.42 MB -> 3.42 * 1024 * 1024 bytes
        assert_eq!(size(3_585_573), "3.42 MB");
    }

    #[test]
    fn large_values_stay_in_gb() {
        assert_eq!(size(10_737_418_240), "10.00 GB");
    }
}
