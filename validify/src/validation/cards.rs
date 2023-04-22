use card_validate::Validate as CardValidate;

#[must_use]
pub fn validate_credit_card<T>(card: T) -> bool
where
    T: AsRef<str>,
{
    CardValidate::from(card.as_ref()).is_ok()
}

#[cfg(test)]
mod tests {

    use super::validate_credit_card;

    #[test]
    fn test_credit_card() {
        let tests = vec![
            ("4539571147647251", true),
            ("343380440754432", true),
            ("zduhefljsdfKJKJZHUI", false),
            ("5236313877109141", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_credit_card(input), expected);
        }
    }

    #[test]
    fn test_credit_card_cow() {
        let test: &'static str = "4539571147647251";
        assert!(validate_credit_card(test));
        let test = String::from("4539571147647251");
        assert!(validate_credit_card(test.as_str()));
        let test = "5236313877109141";
        assert!(!validate_credit_card(test));
        let test = String::from("5236313877109141");
        assert!(!validate_credit_card(test.as_str()));
    }
}
