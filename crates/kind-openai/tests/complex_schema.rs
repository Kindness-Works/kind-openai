use kind_openai_schema::OpenAISchema;

#[test]
#[allow(dead_code)]
fn it_generates_complex_schema_correctly() {
    #[derive(OpenAISchema)]
    /// Hello friends
    struct SuperComplexSchema {
        /// The first one.
        optional_string: Option<String>,
        regular_string: String,
        int: i32,
    }

    #[derive(OpenAISchema)]
    enum BasicEnum {
        Variant1,
        Variant2,
    }

    assert_eq!(
        SuperComplexSchema::openai_schema().to_string(),
        String::from(
            r#"{"description":"Hello friends","name":"SuperComplexSchema","schema":{"additionalProperties":false,"properties":{"int":{"type":"integer"},"optional_string":{"description":"The first one.","type":["string","null"]},"regular_string":{"type":"string"}},"required":["regular_string","int"],"type":"object"},"strict":true}"#
        )
    );

    assert_eq!(
        BasicEnum::openai_schema().to_string(),
        String::from(
            r#"{"description":null,"name":"BasicEnum","schema":{"enum":["Variant1","Variant2"],"type":"string"}}"#
        )
    );
}
