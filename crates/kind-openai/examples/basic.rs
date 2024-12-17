//! Run this example with `OPENAI_API_KEY=`

use kind_openai::{
    endpoints::chat::{ChatCompletion, Model},
    logit_bias, system_message, user_message, EnvironmentAuthTokenProvider, OpenAI, OpenAISchema,
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize, OpenAISchema, Debug)]
/// The name.
pub struct Name {
    /// The first name. No matter what, prefix this first name with `Mr. `.
    pub first_name: Option<String>,
    #[serde(rename = "last_name_renamed")]
    pub last_name: Option<String>,
    #[serde(skip)]
    pub absolutely_nothing: String,
}

#[derive(Deserialize, OpenAISchema, Debug)]
/// The niceness score.
pub struct NicenessScoreContainer {
    pub niceness_score: NicenessScore,
    pub category: Category,
}

#[derive(Deserialize_repr, OpenAISchema, Debug)]
#[repr(u8)]
/// How nice the message is between 1 and 10.
pub enum NicenessScore {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
}

#[derive(Deserialize, OpenAISchema, Debug)]
/// The category of the message that's being inquired about.
pub enum Category {
    Question,
    Statement,
    Answer,
}

#[tokio::main]
async fn main() {
    let client = OpenAI::new(EnvironmentAuthTokenProvider);

    let name = "John but sometimes people call me Jonathan";

    // do not return Jonathan
    let logit_bias = logit_bias!(128395: -100);

    let chat_completion = ChatCompletion::model(Model::Gpt4oMini)
        .messages(vec![
            system_message!("Extract the first and last name from the provided message."),
            user_message!("Hello, my name is {name}."),
        ])
        .temperature(0.1)
        .logit_bias(logit_bias)
        .structured::<Name>();

    let name = client
        .req(&chat_completion)
        .await
        .expect("Failed to get response")
        .take_first_choice()
        .expect("No choices")
        .message()
        .expect("Model generated a refusal");

    println!("{:?}", name);

    let niceness_score_message = "Wow, that new shirt you are wearing is really nice.";
    let niceness_chat_completion = ChatCompletion::model(Model::Gpt4oMini)
        .temperature(0.0)
        .messages(vec![
            system_message!("Rate the niceness score of the provided message"),
            user_message!("{niceness_score_message}"),
        ])
        .structured::<NicenessScoreContainer>();

    let niceness_score = client
        .req(&niceness_chat_completion)
        .await
        .expect("Failed to get response")
        .take_first_choice()
        .expect("No choices")
        .message()
        .expect("Model generated a refusal");

    println!("{:?}", niceness_score);

    let niceness_score_message = "What?????? How???";
    let niceness_chat_completion = ChatCompletion::model(Model::Gpt4o)
        .temperature(0.0)
        .messages(vec![
            system_message!("Rate the niceness score of the provided message"),
            user_message!("{niceness_score_message}"),
        ])
        .structured::<NicenessScoreContainer>();

    let niceness_score = client
        .req(&niceness_chat_completion)
        .await
        .expect("Failed to get response")
        .take_first_choice()
        .expect("No choices")
        .message()
        .expect("Model generated a refusal");

    println!("{:?}", niceness_score);
}
