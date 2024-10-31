use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Execute, QueryBuilder, Sqlite, SqlitePool};
use tokio::sync::broadcast;

use crate::{chat::ChatMessage, error::AppError, AppState};

use super::SocketResponse;

#[derive(Deserialize, Debug)]
pub struct SearchMessage {
    conversations: Box<[i64]>,
    query: String,
}

pub async fn search_message(
    state: &AppState,
    search_message: &SearchMessage,
    sender: &broadcast::Sender<SocketResponse>,
) -> Result<(), AppError> {
    // Final query:
    //     SELECT * FROM messages
    //     WHERE id IN
    //         (SELECT id FROM messages_fts
    //         WHERE conversation_id IN (query.conversations) AND message MATCH 'NEAR("query.query", 5));
    let search_query = search_message.query.to_lowercase();
    let search_query = search_query.trim();
    if search_query.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new("");
    // Generate two queries, one for the normal message and one for the stemmed message.
    // Union them together to get the final result.
        builder.push(
            "SELECT * FROM messages
WHERE id IN
(SELECT rowid FROM messages_fts
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

    for i in 0..2 {
        builder.push(if i == 0 {
            r#"message MATCH 'NEAR(""#
        } else {
            r#"stemmed_message MATCH 'NEAR(""#
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
                separated.push(word);
            }
        }
        builder.push(r#"", 5)'"#);
        if i == 0 {
            builder.push(" OR ");
        }
    }

    builder.push(")");

    let query = builder.build_query_as::<ChatMessage>();
    eprintln!("{}", query.sql());
    let mut query = query.fetch(&state.pool);

    while let Some(message) = query.next().await {
        sender.send(SocketResponse::SearchMessage(message?))?;
    }
    Ok(())
}

fn prepare_query(query: &str) -> String {
    query
        .to_lowercase()
        .chars()
        .fold(
            (String::new(), false, false),
            |(mut acc, mut numeric, mut last_whitespace), cur| {
                if cur.is_alphabetic() {
                    last_whitespace = false;
                    numeric = false;
                    acc.push(cur);
                } else if cur.is_ascii_digit() {
                    last_whitespace = false;
                    numeric = true;
                    acc.push(cur);
                } else if cur.is_whitespace() {
                    if !last_whitespace {
                        acc.push(' ');
                    }
                    last_whitespace = true;
                    numeric = false;
                }

                (acc, numeric, last_whitespace)
            },
        )
        .0
}
