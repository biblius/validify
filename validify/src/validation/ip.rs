use std::net::IpAddr;
use std::str::FromStr;

/// Validates whether the given string is an IP V4
#[must_use]
pub fn validate_ip_v4<T>(val: T) -> bool
where
    T: AsRef<str>,
{
    IpAddr::from_str(val.as_ref()).map_or(false, |i| i.is_ipv4())
}

/// Validates whether the given string is an IP V6
#[must_use]
pub fn validate_ip_v6<T>(val: T) -> bool
where
    T: AsRef<str>,
{
    IpAddr::from_str(val.as_ref()).map_or(false, |i| i.is_ipv6())
}

/// Validates whether the given string is an IP
#[must_use]
pub fn validate_ip<T>(val: T) -> bool
where
    T: AsRef<str>,
{
    IpAddr::from_str(val.as_ref()).is_ok()
}

#[cfg(test)]
mod tests {

    use super::{validate_ip, validate_ip_v4, validate_ip_v6};

    #[test]
    fn test_validate_ip() {
        let tests = vec![
            ("1.1.1.1", true),
            ("255.0.0.0", true),
            ("0.0.0.0", true),
            ("256.1.1.1", false),
            ("25.1.1.", false),
            ("25,1,1,1", false),
            ("fe80::223:6cff:fe8a:2e8a", true),
            ("::ffff:254.42.16.14", true),
            ("2a02::223:6cff :fe8a:2e8a", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_ip(input), expected);
        }
    }

    #[test]
    fn test_validate_ip_cow() {
        let test = "1.1.1.1";
        assert!(validate_ip(test));
        let test = String::from("1.1.1.1");
        assert!(validate_ip(test.as_str()));
        let test = "2a02::223:6cff :fe8a:2e8a";
        assert!(!validate_ip(test));
        let test = String::from("2a02::223:6cff :fe8a:2e8a");
        assert!(!validate_ip(test.as_str()));
    }

    #[test]
    fn test_validate_ip_v4() {
        let tests = vec![
            ("1.1.1.1", true),
            ("255.0.0.0", true),
            ("0.0.0.0", true),
            ("256.1.1.1", false),
            ("25.1.1.", false),
            ("25,1,1,1", false),
            ("25.1 .1.1", false),
            ("1.1.1.1\n", false),
            ("٧.2٥.3٣.243", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_ip_v4(input), expected);
        }
    }

    #[test]
    fn test_validate_ip_v4_cow() {
        let test = "1.1.1.1";
        assert!(validate_ip_v4(test));
        let test = String::from("1.1.1.1");
        assert!(validate_ip_v4(test.as_str()));
        let test = "٧.2٥.3٣.243";
        assert!(!validate_ip_v4(test));
        let test = String::from("٧.2٥.3٣.243");
        assert!(!validate_ip_v4(test.as_str()));
    }

    #[test]
    fn test_validate_ip_v6() {
        let tests = vec![
            ("fe80::223:6cff:fe8a:2e8a", true),
            ("2a02::223:6cff:fe8a:2e8a", true),
            ("1::2:3:4:5:6:7", true),
            ("::", true),
            ("::a", true),
            ("2::", true),
            ("::ffff:254.42.16.14", true),
            ("::ffff:0a0a:0a0a", true),
            ("::254.42.16.14", true),
            ("::0a0a:0a0a", true),
            ("foo", false),
            ("127.0.0.1", false),
            ("12345::", false),
            ("1::2::3::4", false),
            ("1::zzz", false),
            ("1:2", false),
            ("fe80::223: 6cff:fe8a:2e8a", false),
            ("2a02::223:6cff :fe8a:2e8a", false),
            ("::ffff:999.42.16.14", false),
            ("::ffff:zzzz:0a0a", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_ip_v6(input), expected);
        }
    }

    #[test]
    fn test_validate_ip_v6_cow() {
        let test = "fe80::223:6cff:fe8a:2e8a";
        assert!(validate_ip_v6(test));
        let test = String::from("fe80::223:6cff:fe8a:2e8a");
        assert!(validate_ip_v6(test.as_str()));
        let test = "::ffff:zzzz:0a0a";
        assert!(!validate_ip_v6(test));
        let test = String::from("::ffff:zzzz:0a0a");
        assert!(!validate_ip_v6(test.as_str()));
    }
}
