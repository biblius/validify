#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};
    use validify::{validify, Validify};

    #[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
        let t = T {
            a: String::new(),
            b: U { b: 2 },
            c: vec!["lmeo".to_string()],
        };
        T::validate(t.into()).unwrap();
    }
}
