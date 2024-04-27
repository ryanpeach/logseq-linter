//! Meilisearch is a powerful, fast, open-source, easy to use text search engine.

/// Taken from meilisearch readme
#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use meilisearch_sdk::client::*;
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize, Deserialize, Debug)]
    struct Movie {
        id: usize,
        title: String,
        genres: Vec<String>,
    }

    #[tokio::test]
    async fn test_add_documents_and_search() {
        dotenv().ok();
        let meilisearch_url =
            env::var("MEILISEARCH_URL").unwrap_or("http://localhost:7700".to_string());
        let meilisearch_api_key =
            env::var("MEILISEARCH_API_KEY").unwrap_or("masterKey".to_string());

        // Create a client (without sending any request so that can't fail)
        let client = Client::new(meilisearch_url, Some(meilisearch_api_key));

        // An index is where the documents are stored.
        let movies = client.index("movies");

        // Add some movies in the index. If the index 'movies' does not exist, Meilisearch creates it when you first add the documents.
        movies
            .add_documents(
                &[
                    Movie {
                        id: 1,
                        title: String::from("Carol"),
                        genres: vec!["Romance".to_string(), "Drama".to_string()],
                    },
                    Movie {
                        id: 2,
                        title: String::from("Wonder Woman"),
                        genres: vec!["Action".to_string(), "Adventure".to_string()],
                    },
                    Movie {
                        id: 3,
                        title: String::from("Life of Pi"),
                        genres: vec!["Adventure".to_string(), "Drama".to_string()],
                    },
                    Movie {
                        id: 4,
                        title: String::from("Mad Max"),
                        genres: vec!["Adventure".to_string(), "Science Fiction".to_string()],
                    },
                    Movie {
                        id: 5,
                        title: String::from("Moana"),
                        genres: vec!["Fantasy".to_string(), "Action".to_string()],
                    },
                    Movie {
                        id: 6,
                        title: String::from("Philadelphia"),
                        genres: vec!["Drama".to_string()],
                    },
                ],
                Some("id"),
            )
            .await
            .expect("Cannot add documents");

        // Meilisearch is typo-tolerant:
        println!(
            "{:?}",
            client
                .index("movies")
                .search()
                .with_query("caorl")
                .execute::<Movie>()
                .await
                .unwrap()
                .hits
        );
    }
}
