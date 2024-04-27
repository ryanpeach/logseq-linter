//! The entry point of the program.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#[macro_use]

mod meilisearch;
mod files;
mod logseq;
use std::path::Path;

use clap::{command, Parser};
use dotenv::dotenv;
use indicatif::ProgressIterator;
use logseq::File;
use markdown::mdast;
use meilisearch_sdk::Client;
use std::env;

use lazy_static::lazy_static;

lazy_static! {
    static ref MEILISEARCH_URL: String =
        env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    static ref MEILISEARCH_API_KEY: String =
        env::var("MEILISEARCH_API_KEY").unwrap_or_else(|_| "masterKey".to_string());
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input folder path
    path: Box<Path>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Load environment variables from a .env file
    dotenv().ok();

    // Create a client (without sending any request so that can't fail)
    let client = Client::new(&*MEILISEARCH_URL, Some(&*MEILISEARCH_API_KEY));

    // An index is where the documents are stored.
    let logseq = client.index("logseq");

    // Load the files into the database
    let args = Args::parse();
    let walker = files::MdWalker::new(args.path.to_str().unwrap());
    let mut i = 0;
    for file in walker
        .into_iter()
        .collect::<Vec<Result<(Box<Path>, mdast::Node), String>>>()
        .iter()
        .progress()
    {
        let doc = match file {
            Ok((path, ast)) => {
                // println!("{:?}", out);
                File::new(i, ast, path.clone())
            }
            Err(msg) => {
                eprintln!("{}", msg);
                continue;
            }
        };
        logseq
            .add_documents(&[doc], Some("id"))
            .await
            .expect("Cannot add documents");
        i += 1;
    }
}
