use teloxide::RequestError;

use crate::bot_error::BotError::TelegramError;

#[derive(Debug)]
pub(crate) enum BotError {
    RequestError(reqwest::Error),
    ThrottleError,
    GenericError(anyhow::Error),
    TelegramError(RequestError),
}

impl From<reqwest::Error> for BotError {
    fn from(value: reqwest::Error) -> Self {
        BotError::RequestError(value)
    }
}

impl From<anyhow::Error> for BotError {
    fn from(err: anyhow::Error) -> Self {
        BotError::GenericError(err.into())
    }
}

impl From<RequestError> for BotError {
    fn from(value: RequestError) -> Self {
        TelegramError(value)
    }
}
