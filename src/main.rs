//! The entry point of the program.
// #![warn(missing_docs)]
// #![warn(clippy::missing_docs_in_private_items)]

mod indexer;
mod meilisearch;
mod parsing;
use std::path::PathBuf;

use clap::{command, Parser};
use dotenv::dotenv;
use indexer::Indexer;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input folder path
    path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Load environment variables from a .env file
    dotenv().ok();

    // Load the files into the database
    let args = Args::parse();

    Indexer::new()
        .await
        .index_files(args.path.to_str().unwrap())
        .await
        .unwrap();
}
