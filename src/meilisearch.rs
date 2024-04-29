//! Meilisearch is a powerful, fast, open-source, easy to use text search engine.
use meilisearch_sdk::Client;
use std::env;

pub struct Meilisearch {
    pub client: Client,
}

impl Meilisearch {
    pub fn new() -> Meilisearch {
        let url =
            env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
        let api_key = env::var("MEILISEARCH_API_KEY").unwrap_or_else(|_| "masterKey".to_string());
        let client = Client::new(url, Some(api_key));
        Meilisearch { client }
    }
}

/// Taken from meilisearch readme
#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use serde::{Deserialize, Serialize};

    use super::Meilisearch;

    #[derive(Serialize, Deserialize, Debug)]
    struct Movie {
        id: usize,
        title: String,
        genres: Vec<String>,
    }

    #[tokio::test]
    async fn test_add_documents_and_search() {
        dotenv().ok();

        // Create a client (without sending any request so that can't fail)
        let client = Meilisearch::new().client;

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
