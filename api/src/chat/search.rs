use chrono::NaiveDate;
use futures::StreamExt;
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};
use tokio::sync::broadcast::Sender;

use crate::{chat::ChatMessage, error::AppError, state::AppState};

use super::SocketResponse;

#[derive(Deserialize, Debug)]
pub struct SearchMessage {
    conversations: Box<[i64]>,
    query: String,
    #[serde(default = "Default::default")]
    order: SearchOrder,
    #[serde(default = "Box::default")]
    filters: Box<[Filter]>,
}

#[derive(Deserialize, Debug)]
pub enum SearchOrder {
    Newest,
    Oldest,
    Relevance,
}

impl Default for SearchOrder {
    fn default() -> Self {
        Self::Newest
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Filter {
    Before(NaiveDate),
    After(NaiveDate),
    During(NaiveDate),
    User(Option<i64>),
    AiModel(Option<i64>),
}

// Note: This query can return duplicate rows because of the rank column being included.
// The rank column is used to determine the relevance of the search results and will be
// different depending on whether the search query matched the message or the stemmed message.
// The rank column must be included in order to rank the results by relevance, otherwise
// the database will return an error.
//
// Using union to query both the `message` and `stemmed_message` columns because nothing else worked.
// Attempting to use something simpler like a WHERE clause with a condition for `message` and
// another for `stemmed message`, while also using a ORDER BY clause to order the results by
// rank will result in an error from the database saying that match is not allowed in the given context.
// ¯\_(ツ)_/¯
//
// The final query will look something like:
// SELECT *, messages_fts.rank FROM chat_messages
// JOIN messages_fts
// ON messages.id = messages_fts.rowid
// WHERE messages_fts.message MATCH 'NEAR(search_query, 5)'
// UNION
// SELECT *, messages_fts.rank FROM chat_messages
// JOIN messages_fts
// ON messages.id = messages_fts.rowid
// WHERE messages_fts.stemmed_message
// MATCH 'NEAR(stem(search_query), 5)' ORDER BY messages_fts.rank;
/// Search messages in the database according to given query
pub async fn search_message(
    state: &AppState,
    search_message: &SearchMessage,
    sender: &Sender<SocketResponse>,
) -> Result<(), AppError> {
    // Escape single quotes and convert to lowercase
    let search_query = search_message.query.replace("'", "''").to_lowercase();
    let search_query = search_query.trim();
    if search_query.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new("");
    // Generate two queries, one for the normal message and one for the stemmed message.
    // Union them together to get the final result.
    for i in 0..2 {
        builder.push(
            "SELECT *, messages_fts.rank FROM chat_messages
                JOIN messages_fts
                ON chat_messages.id = messages_fts.rowid 
                WHERE ",
        );

        if !search_message.conversations.is_empty() {
            builder.push("conversation_id IN (");

            let mut separated = builder.separated(", ");
            for conversation in search_message.conversations.iter() {
                separated.push_bind(conversation);
            }
            separated.push_unseparated(") AND ");
        }

        builder.push(if i == 0 {
            r#"messages_fts.message MATCH 'NEAR("#
        } else {
            r#"messages_fts.stemmed_message MATCH 'NEAR("#
        });
        {
            let mut separated = builder.separated(' ');
            for word in search_query.split_whitespace() {
                let word = if i == 0 {
                    word
                } else {
                    &state.stemmer.stem(word)
                };
                // FTS5 uses a special query syntax which does not work with normal sql binds and
                // doesn't require input sanitization so just raw dog it.
                // (I was banging my head against the wall for like an hour trying to figure out why it wasn't working)
                separated.push(format!(r#""{}""#, word));
            }
        }
        builder.push(r#", 5)'"#);

        for filter in &search_message.filters {
            builder.push(" AND ");
            match filter {
                Filter::Before(date) => {
                    builder.push("chat_messages.created_at < ?");
                    builder.push_bind(date);
                }
                Filter::After(date) => {
                    builder.push("chat_messages.created_at > ?");
                    builder.push_bind(date);
                }
                Filter::During(date) => {
                    builder.push("chat_messages.created_at >= ? AND chat_messages.created_at < ?");
                    builder.push_bind(date);
                    builder.push_bind(*date + chrono::Duration::days(1));
                }
                Filter::User(Some(user_id)) => {
                    builder.push("user_id = ?");
                    builder.push_bind(user_id);
                }
                Filter::User(None) => {
                    builder.push("ai_model_id IS NULL");
                }
                Filter::AiModel(Some(model_id)) => {
                    builder.push("ai_model_id = ?");
                    builder.push_bind(model_id);
                }
                Filter::AiModel(None) => {
                    builder.push("user_id IS NULL");
                }
            }
        }
        if i == 0 {
            builder.push(" UNION ");
        }
    }

    builder.push(" ORDER BY ");
    builder.push(match search_message.order {
        SearchOrder::Newest => "chat_messages.created_at DESC",
        SearchOrder::Oldest => "chat_messages.created_at ASC",
        SearchOrder::Relevance => "chat_messages_fts.rank DESC",
    });

    let query = builder.build_query_as::<ChatMessage>();
    let mut query = query.fetch(&state.pool);

    while let Some(message) = query.next().await {
        match message {
            Ok(message) => sender.send(SocketResponse::SearchMessage(message))?,
            // Check if the error is a database error with code 1 which means the search query is invalid
            Err(e)
                if e.as_database_error()
                    .and_then(|e| e.code())
                    .is_some_and(|code| code == "1") =>
            {
                return Err(AppError::UserError((
                    StatusCode::BAD_REQUEST,
                    "Invalid search query".into(),
                )))
            }

            Err(e) => return Err(e.into()),
        };
    }
    Ok(())
}
