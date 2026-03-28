/// Compute Mod 10 check digit with alternating weights 1 and 3.
/// Used by EAN-13, EAN-8, UPC-A, ISBN-13, GTIN-14, ISMN.
/// `digits` should be the payload digits (without the check digit).
pub fn compute_mod10_check_digit(digits: &[u8]) -> u8 {
    let len = digits.len();
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            // For EAN-13: positions from left, odd positions (0-indexed even) get weight 1,
            // even positions (0-indexed odd) get weight 3.
            // The total length including check digit determines the pattern.
            let weight = if (len - i).is_multiple_of(2) { 1 } else { 3 };
            d as u32 * weight
        })
        .sum();
    ((10 - (sum % 10)) % 10) as u8
}

/// Validate a full number (including check digit) using Mod 10 (1,3 weights).
pub fn validate_mod10(digits: &[u8]) -> bool {
    if digits.is_empty() {
        return false;
    }
    let check = compute_mod10_check_digit(&digits[..digits.len() - 1]);
    check == digits[digits.len() - 1]
}

/// Validate an ISBN-10 string (9 digits + check digit which can be 0-9 or 'X').
/// Input should be stripped of hyphens/spaces and be exactly 10 characters.
pub fn validate_isbn10(input: &str) -> bool {
    if input.chars().count() != 10 {
        return false;
    }

    let chars: Vec<char> = input.chars().collect();

    // First 9 must be digits
    if !chars[..9].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Last can be digit or 'X'/'x'
    let check_val = match chars[9] {
        '0'..='9' => chars[9] as u32 - '0' as u32,
        'X' | 'x' => 10,
        _ => return false,
    };

    let sum: u32 = chars[..9]
        .iter()
        .enumerate()
        .map(|(i, &c)| (c as u32 - '0' as u32) * (10 - i as u32))
        .sum::<u32>()
        + check_val;

    sum.is_multiple_of(11)
}

/// Compute the ISBN-10 check digit for 9 payload digits.
/// Returns a char: '0'-'9' or 'X'.
pub fn compute_isbn10_check(digits: &[u8]) -> char {
    assert!(digits.len() == 9);
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| d as u32 * (10 - i as u32))
        .sum();
    let remainder = (11 - (sum % 11)) % 11;
    if remainder == 10 {
        'X'
    } else {
        (b'0' + remainder as u8) as char
    }
}

/// Validate an ISIN using the Luhn algorithm on alpha-converted digits.
/// Letters are converted: A=10, B=11, ..., Z=35, then Luhn is applied to the resulting digit string.
pub fn validate_isin_luhn(input: &str) -> bool {
    if input.chars().count() != 12 {
        return false;
    }

    // Convert alphanumeric to digit string
    let mut digit_string = String::new();
    for c in input.chars() {
        if c.is_ascii_digit() {
            digit_string.push(c);
        } else if c.is_ascii_uppercase() {
            let val = c as u32 - 'A' as u32 + 10;
            digit_string.push_str(&val.to_string());
        } else {
            return false;
        }
    }

    // Apply Luhn algorithm on the digit string
    luhn_check(&digit_string)
}

/// Standard Luhn algorithm on a string of digits.
fn luhn_check(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;

    for c in digits.chars().rev() {
        let Some(d) = c.to_digit(10) else {
            return false;
        };
        if double {
            let doubled = d * 2;
            sum += if doubled > 9 { doubled - 9 } else { doubled };
        } else {
            sum += d;
        }
        double = !double;
    }

    sum.is_multiple_of(10)
}

/// Validate an ISSN string (7 digits + check digit which can be 0-9 or 'X').
/// Input should be stripped of hyphens/spaces and be exactly 8 characters.
/// Weighted sum: d1*8 + d2*7 + d3*6 + d4*5 + d5*4 + d6*3 + d7*2 + check ≡ 0 (mod 11)
pub fn validate_issn(input: &str) -> bool {
    if input.chars().count() != 8 {
        return false;
    }

    let chars: Vec<char> = input.chars().collect();

    // First 7 must be digits
    if !chars[..7].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Last can be digit or 'X'/'x'
    let check_val = match chars[7] {
        '0'..='9' => chars[7] as u32 - '0' as u32,
        'X' | 'x' => 10,
        _ => return false,
    };

    let sum: u32 = chars[..7]
        .iter()
        .enumerate()
        .map(|(i, &c)| (c as u32 - '0' as u32) * (8 - i as u32))
        .sum::<u32>()
        + check_val;

    sum.is_multiple_of(11)
}

/// Validate using ISO 7064 MOD 11-2 algorithm.
/// Used by ISNI. Input should be stripped of spaces and be exactly 16 characters.
/// Last character can be 0-9 or X.
pub fn validate_iso7064_mod11_2(input: &str) -> bool {
    if input.chars().count() != 16 {
        return false;
    }

    let chars: Vec<char> = input.chars().collect();

    // First 15 must be digits
    if !chars[..15].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Last can be digit or 'X'
    let check_val: u32 = match chars[15] {
        '0'..='9' => chars[15] as u32 - '0' as u32,
        'X' | 'x' => 10,
        _ => return false,
    };

    // ISO 7064 MOD 11-2: process left to right
    let mut s: u32 = 0;
    for &c in &chars[..15] {
        let d = c as u32 - '0' as u32;
        s = ((s + d) * 2) % 11;
    }

    let expected = (12 - s) % 11;
    expected == check_val
}

/// Strip hyphens and spaces from input for flexible parsing.
pub fn strip_formatting(input: &str) -> String {
    input.chars().filter(|&c| c != '-' && c != ' ').collect()
}

/// Parse a string of digits into a Vec<u8> of digit values.
/// Returns None if any character is not a digit.
pub fn parse_digits(input: &str) -> Option<Vec<u8>> {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_digit() {
                Some(c as u8 - b'0')
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_mod10_ean13() {
        // EAN-13: 4006381333931
        let payload: Vec<u8> = vec![4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3];
        assert_eq!(compute_mod10_check_digit(&payload), 1);
    }

    #[test]
    fn test_validate_mod10_ean13() {
        let digits: Vec<u8> = vec![4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3, 1];
        assert!(validate_mod10(&digits));

        let bad: Vec<u8> = vec![4, 0, 0, 6, 3, 8, 1, 3, 3, 3, 9, 3, 2];
        assert!(!validate_mod10(&bad));
    }

    #[test]
    fn test_compute_mod10_isbn13() {
        // ISBN-13: 978-0-306-40615-7
        let payload: Vec<u8> = vec![9, 7, 8, 0, 3, 0, 6, 4, 0, 6, 1, 5];
        assert_eq!(compute_mod10_check_digit(&payload), 7);
    }

    #[test]
    fn test_validate_isbn10() {
        assert!(validate_isbn10("0306406152"));
        assert!(validate_isbn10("080442957X"));
        assert!(!validate_isbn10("0306406153")); // wrong check digit
        assert!(!validate_isbn10("short"));
    }

    #[test]
    fn test_compute_isbn10_check() {
        let digits: Vec<u8> = vec![0, 3, 0, 6, 4, 0, 6, 1, 5];
        assert_eq!(compute_isbn10_check(&digits), '2');

        let digits2: Vec<u8> = vec![0, 8, 0, 4, 4, 2, 9, 5, 7];
        assert_eq!(compute_isbn10_check(&digits2), 'X');
    }

    #[test]
    fn test_validate_isin_luhn() {
        assert!(validate_isin_luhn("US0378331005")); // Apple
        assert!(validate_isin_luhn("AU0000XVGZA3"));
        assert!(validate_isin_luhn("GB0002634946"));
        assert!(!validate_isin_luhn("US0378331006")); // wrong check
        assert!(!validate_isin_luhn("short"));
    }

    #[test]
    fn test_strip_formatting() {
        assert_eq!(strip_formatting("978-0-306-40615-7"), "9780306406157");
        assert_eq!(strip_formatting("0317 8471"), "03178471");
        assert_eq!(strip_formatting("plain"), "plain");
    }

    #[test]
    fn test_parse_digits() {
        assert_eq!(parse_digits("123"), Some(vec![1, 2, 3]));
        assert_eq!(parse_digits("12a"), None);
        assert_eq!(parse_digits(""), Some(vec![]));
    }

    #[test]
    fn test_luhn_check() {
        // Standard Luhn test
        assert!(luhn_check("79927398713"));
        assert!(!luhn_check("79927398714"));
    }

    #[test]
    fn test_validate_mod10_empty() {
        assert!(!validate_mod10(&[]));
    }

    #[test]
    fn test_validate_issn() {
        assert!(validate_issn("03785955")); // Nature
        assert!(validate_issn("03064530")); // Science (hypothetical)
        assert!(!validate_issn("03785956")); // wrong check digit
        assert!(!validate_issn("short"));
    }

    #[test]
    fn test_validate_issn_with_x() {
        assert!(validate_issn("0000006X")); // X as check digit
    }

    #[test]
    fn test_validate_iso7064_mod11_2() {
        assert!(validate_iso7064_mod11_2("0000000121032683")); // ISNI example
        assert!(!validate_iso7064_mod11_2("0000000121032684")); // wrong check
        assert!(!validate_iso7064_mod11_2("short"));
    }
}
