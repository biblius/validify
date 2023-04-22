use url::Url;

/// Validates whether the string given is a url
#[must_use]
pub fn validate_url<T>(val: T) -> bool
where
    T: AsRef<str>,
{
    Url::parse(val.as_ref()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::validate_url;

    #[test]
    fn test_validate_url() {
        let tests = vec![
            ("http", false),
            ("https://google.com", true),
            ("http://localhost:80", true),
            ("ftp://localhost:80", true),
        ];

        for (url, expected) in tests {
            assert_eq!(validate_url(url), expected);
        }
    }

    #[test]
    fn test_validate_url_cow() {
        let test = "http://localhost:80";
        assert!(validate_url(test));
        let test = String::from("http://localhost:80");
        assert!(validate_url(test));
        let test = "http";
        assert!(!validate_url(test));
        let test = String::from("http");
        assert!(!validate_url(test));
    }
}
