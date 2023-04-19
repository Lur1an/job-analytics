use async_openai::Client;
use async_openai::types::{CreateChatCompletionRequest, ChatCompletionRequestMessage};
use lazy_static::lazy_static;

lazy_static! {
    const SYSTEM_MESSAGE: &str = ChatCompletionRequestMessage::default().text("System: ");
}

async fn extract_job_details(content: &str) {
    let client = Client::new();
    let request = CreateChatCompletionRequest::default().messages(
        vec![content.to_string()]
    );
    let response = client.chat().create_completion(request).await;
}