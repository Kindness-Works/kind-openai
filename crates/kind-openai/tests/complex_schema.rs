use kind_openai_schema::OpenAISchema;
use serde::Deserialize;

#[test]
#[allow(dead_code)]
fn it_generates_complex_schema_correctly() {
    #[derive(Deserialize, OpenAISchema)]
    /// Hello friends
    struct SuperComplexSchema {
        /// The first one.
        optional_string: Option<String>,
        regular_string: String,
        int: i32,
        basic_enum: BasicEnum,
    }

    #[derive(Deserialize, OpenAISchema)]
    enum BasicEnum {
        #[serde(rename = "variant1")]
        Variant1,
        Variant2,
        #[serde(
            rename = "this-has-a-really-really-long-name-that-strangely-broke-some-thing-with-this-maybe-the-attrs-have-a-max-len-i-dont-really-know"
        )]
        ReallyLong,
    }

    assert_eq!(
        SuperComplexSchema::openai_schema().to_string(),
        String::from(
            r#"{"name":"SuperComplexSchema","description":"Hello friends","strict":true,"schema":{"type":"object","additionalProperties":false,"properties":{"optional_string":{"description":"The first one.","type":["string","null"]},"regular_string":{"type":"string"},"int":{"type":"integer"},"basic_enum":{"enum":["variant1","Variant2","this-has-a-really-really-long-name-that-strangely-broke-some-thing-with-this-maybe-the-attrs-have-a-max-len-i-dont-really-know"],"type":"string"}},"required":["optional_string","regular_string","int","basic_enum"]}}"#
        )
    );
}
