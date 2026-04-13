use heck::{ToPascalCase, ToLowerCamelCase};

#[derive(Debug, Clone)]
pub struct ContractNames {
    /// PascalCase  — e.g. `SimpleContract`
    pub pascal: String,
    /// camelCase   — e.g. `simpleContract`
    pub camel: String,
    /// all-lowercase — e.g. `simplecontract`
    pub lower: String,
}

impl ContractNames {
    pub fn from_raw(raw: &str) -> Self {
        Self {
            pascal: raw.to_pascal_case(),
            camel:  raw.to_lower_camel_case(),
            lower:  raw.to_lowercase(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_names() {
        let names = ContractNames::from_raw("MyContract");
        assert_eq!(names.pascal, "MyContract");
        assert_eq!(names.camel, "myContract");
        assert_eq!(names.lower, "mycontract");

        let names_snake = ContractNames::from_raw("simple_token");
        assert_eq!(names_snake.pascal, "SimpleToken");
        assert_eq!(names_snake.camel, "simpleToken");
        assert_eq!(names_snake.lower, "simple_token");
    }
}
