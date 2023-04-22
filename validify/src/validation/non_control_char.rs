use unic_ucd_common::control;

#[must_use]
pub fn validate_non_control_character<T>(alphabetic: T) -> bool
where
    T: AsRef<str> + Clone,
{
    alphabetic
        .as_ref()
        .chars()
        .all(|code| !control::is_control(code))
}

#[cfg(test)]
mod tests {
    use super::validate_non_control_character;

    #[test]
    fn test_non_control_character() {
        let tests = vec![
            ("Himmel", true),
            ("आकाश", true),
            ("வானத்தில்", true),
            ("하늘", true),
            ("небо", true),
            ("2H₂ + O₂ ⇌ 2H₂O", true),
            ("\u{000c}", false),
            ("\u{009F}", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_non_control_character(input), expected);
        }
    }

    #[test]
    fn test_non_control_character_cow() {
        let test = "आकाश";
        assert!(validate_non_control_character(test));
        let test = String::from("வானத்தில்");
        assert!(validate_non_control_character(test));
        let test = "\u{000c}";
        assert!(!validate_non_control_character(test));
        let test = String::from("\u{009F}");
        assert!(!validate_non_control_character(test));
    }
}
