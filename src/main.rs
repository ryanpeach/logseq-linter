//! The entry point of the program.
mod indexer;
mod meilisearch;
mod parsing;
use std::path::Path;

use clap::{command, Parser};
use dotenv::dotenv;
use indexer::Indexer;

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

    // Load the files into the database
    let args = Args::parse();

    Indexer::new()
        .index_files(args.path.to_str().unwrap())
        .await
        .unwrap();
}
