use futures::StreamExt;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};
use tokio::sync::broadcast;

use crate::{chat::ChatMessage, error::AppError, AppState};

use super::SocketResponse;

#[derive(Deserialize, Debug)]
pub struct SearchMessage {
    conversations: Box<[i64]>,
    query: String,
    #[serde(default = "Default::default")]
    order: SearchOrder,
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
// SELECT *, messages_fts.rank FROM messages
// JOIN messages_fts
// ON messages.id = messages_fts.rowid
// WHERE messages_fts.message MATCH 'NEAR("search_query", 5)'
// UNION
// SELECT *, messages_fts.rank FROM messages
// JOIN messages_fts
// ON messages.id = messages_fts.rowid
// WHERE messages_fts.stemmed_message
// MATCH 'NEAR("stem(search_query)" "linear", 5)' ORDER BY messages_fts.rank;
/// Search messages in the database according to given query
pub async fn search_message(
    state: &AppState,
    search_message: &SearchMessage,
    sender: &broadcast::Sender<SocketResponse>,
) -> Result<(), AppError> {

    let search_query = search_message.query.to_lowercase();
    let search_query = search_query.trim();
    if search_query.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new("");
    // Generate two queries, one for the normal message and one for the stemmed message.
    // Union them together to get the final result.
    for i in 0..2 {
        builder.push(
            "SELECT *, messages_fts.rank FROM messages
                JOIN messages_fts
                ON messages.id = messages_fts.rowid 
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
        if i == 0 {
            builder.push(" UNION ");
        }
    }

    builder.push(" ORDER BY ");
    builder.push(match search_message.order {
        SearchOrder::Newest => "messages.created_at DESC",
        SearchOrder::Oldest => "messages.created_at ASC",
        SearchOrder::Relevance => "messages_fts.rank DESC",
    });

    let query = builder.build_query_as::<ChatMessage>();
    let mut query = query.fetch(&state.pool);

    while let Some(message) = query.next().await {
        sender.send(SocketResponse::SearchMessage(message?))?;
    }
    Ok(())
}
