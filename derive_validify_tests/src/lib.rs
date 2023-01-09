#[cfg(test)]
mod tests {

    use validify::{validify, Validify};

    #[derive(Debug)]
    #[validify]
    struct T {
        #[modify(custom = "foo", uppercase)]
        #[validate(length(min = 1))]
        a: String,
        #[validify]
        b: U,
        #[modify(trim, lowercase, capitalize)]
        c: Vec<String>,
    }

    #[derive(Debug)]
    #[validify]
    struct U {
        #[validate(range(min = 1))]
        b: usize,
    }

    fn foo(a: &mut String) {
        *a = "foo".to_string();
    }

    #[test]
    fn validate() {
        let mut t = T {
            a: String::new(),
            b: U { b: 2 },
            c: vec!["lmeo".to_string()],
        };
        t.validate().unwrap();
    }
}
