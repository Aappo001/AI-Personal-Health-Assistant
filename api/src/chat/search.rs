use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Execute, QueryBuilder, Sqlite, SqlitePool};
use tokio::sync::broadcast;

use crate::{chat::ChatMessage, error::AppError};

use super::SocketResponse;

#[derive(Deserialize, Debug)]
pub struct SearchMessage {
    conversations: Box<[i64]>,
    query: String,
}

pub async fn search_message(
    pool: &SqlitePool,
    search_message: &SearchMessage,
    sender: &broadcast::Sender<SocketResponse>,
) -> Result<(), AppError> {
    // Final query:
    //     "SELECT * FROM messages
    //     WHERE id IN
    //         (SELECT id FROM messages_fts
    //         WHERE conversation_id IN (query.conversations) AND message MATCH query.query)"
    let search_query = search_message.query.trim();
    if search_query.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
        "SELECT * FROM messages
        WHERE id IN
            (SELECT id FROM messages_fts
            WHERE ",
    );

    if !search_message.conversations.is_empty() {
        builder.push("conversation_id IN (");

        let mut separated = builder.separated(", ");
        for conversation in search_message.conversations.iter() {
            separated.push_bind(conversation);
        }
        separated.push_unseparated(") AND");
    }

    builder.push(r#" message MATCH 'NEAR(""#);
    {
        let mut separated = builder.separated(' ');
        for word in search_query.split_whitespace() {
            // FTS5 uses a special query syntax which does not work with normal sql binds and
            // doesn't require input sanitization so just raw dog it.
            // (I was banging my head against the wall for like an hour trying to figure out why it wasn't working)
            separated.push(word);
        }
    }
    builder.push(r#"", 5)');"#);

    let query = builder.build_query_as::<ChatMessage>();
    eprintln!("{}", query.sql());
    let mut query = query
        .fetch(pool);

    while let Some(message) = query.next().await {
        sender.send(SocketResponse::SearchMessage(message?))?;
    }
    Ok(())
}
