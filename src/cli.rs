//! Command line arguments accepted by the application.

use std::path::PathBuf;

/// Command line arguments accepted by the application.
#[derive(Debug, clap::Parser)]
#[command(author, version, about)]
pub(crate) struct Args {
    /// Input file to read the input text from.
    #[arg(short = 'f', long, value_name = "FILE")]
    pub(crate) input_file: Option<PathBuf>,

    /// Sqlite database file to persist user information. A database will be created if one doesn't
    /// already exist.
    #[arg(
        short = 'd',
        long,
        value_name = "DATABASE_URI",
        default_value_t = String::from("sqlite://users.db")
    )]
    pub(crate) database: String,

    /// Flag to specify whether the output should be written back to file, or to stdout.
    #[arg(short = 'i', long, requires = "input_file")]
    pub(crate) in_place: bool,
}
