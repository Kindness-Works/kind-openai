# Kind OpenAI

An experimental, highly opinionated OpenAI API wrapper for Rust. This is primarily designed around making structured outputs easy to work with.

## Example

```rust
//! Run this example with `OPENAI_API_KEY=`

use kind_openai::{
    endpoints::chat::{ChatCompletionModel, ChatCompletionRequest, ChatCompletionRequestMessage},
    system_message, user_message, EnvironmentAuthTokenProvider, OpenAI, OpenAISchema,
};
use serde::Deserialize;

#[derive(Deserialize, OpenAISchema, Debug)]
pub struct Name {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[tokio::main]
async fn main() {
    let client = OpenAI::new(EnvironmentAuthTokenProvider);

    let name = "John Doe";

    let chat_completion: ChatCompletionRequest<Name> =
        ChatCompletionRequest::new_structured(ChatCompletionModel::Gpt4oMini)
            .message(system_message!(
                "Extract the first and last name from the provided message."
            ))
            .message(user_message!("Hello, my name is {name}."))
            .temperature(0.1);

    let name = client
        .create_chat_completion(&chat_completion)
        .await
        .unwrap()
        .take_first_choice()
        .expect("No choices")
        .message()
        .expect("Model generated a refusal");

    println!("{:?}", name);
}

```
