//! A simple application to find and replace emails in changelogs with GitHub usernames.

mod cli;
mod db;
mod emails;

use std::fs;

use anyhow::Context;
use once_cell::sync::OnceCell;

use crate::db::{GithubUser, GithubUserRepository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = <cli::Args as clap::Parser>::parse();

    let pool = db::create_pool(args.database).await?;
    db::run_migrations(&pool).await?;
    let user_repository = GithubUserRepository::new(pool);

    let input = read_input(args.input_file.as_ref())?;
    let emails = emails::find_emails(&input);

    let mut output = input.clone();
    let client = OnceCell::new();
    let mut unavailable_emails = Vec::with_capacity(emails.len());
    for email in emails {
        let username = if let Some(prefix) = email.strip_suffix("@users.noreply.github.com") {
            // GitHub no-reply email address
            // Format: 1234567+GitHubUsername@users.noreply.github.com
            if let Some((_, username)) = prefix.split_once('+') {
                Some(username.to_owned())
            } else {
                unavailable_emails.push(email.clone());
                None
            }
        } else {
            // Retrieve username from database if available, query GitHub API otherwise.
            let db_user = user_repository.get(&email).await?;
            if let Some(user) = db_user {
                Some(user.username)
            } else {
                // Construct GitHub client only when absolutely necessary
                let users =
                    search_user_by_email(client.get_or_try_init(construct_client)?, &email).await?;

                // This shouldn't ideally happen, since GitHub does not allow two accounts to be linked
                // with the same email address.
                anyhow::ensure!(
                    users.total_count == 1,
                    "More than one user found for {email}"
                );

                match users.items.first() {
                    None => {
                        unavailable_emails.push(email.clone());
                        None
                    }
                    Some(user) => {
                        let username = user.login.clone();

                        let user = GithubUser::new(email.clone(), username.clone());
                        user_repository.insert(user).await?;

                        Some(username)
                    }
                }
            }
        };

        if let Some(username) = username {
            output = output.replace(&email, &format!("@{username}"));
        }
    }

    match (args.input_file, args.in_place) {
        (Some(file), true) => fs::write(file, output)?,
        _ => println!("{output}"),
    }

    // Logging status messages to stderr allows for piping stdout to other tools or clipboard,
    // while ensuring that status messages are displayed to the user.
    eprintln!("Email addresses replaced with corresponding GitHub usernames.");
    if !unavailable_emails.is_empty() {
        eprintln!(
            "
            GitHub usernames for the following email addresses are unavailable.\
            Either the email addresses are invalid, or the users updated their publicly visible
            email addresses recently.\n{}",
            unavailable_emails.join("\n")
        );
    }

    Ok(())
}

/// Read input from the file path if specified, or standard input if the file path is `None`.
fn read_input(file: Option<impl AsRef<std::path::Path>>) -> anyhow::Result<String> {
    use std::io::{self, BufRead, BufReader};

    let mut reader: Box<dyn BufRead> = match file {
        Some(file) => Box::new(BufReader::new(
            fs::File::open(file).context("Failed to open input file")?,
        )),
        None => Box::new(BufReader::new(io::stdin().lock())),
    };

    let mut input = String::new();
    reader
        .read_to_string(&mut input)
        .context("Failed to read input")?;

    Ok(input)
}

/// Construct a GitHub API client.
fn construct_client() -> anyhow::Result<octorust::Client> {
    let token = std::env::var("GITHUB_TOKEN").context("`GITHUB_TOKEN` not set")?;
    octorust::Client::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        octorust::auth::Credentials::Token(token),
    )
    .context("Failed to construct GitHub client")
}

/// Search for a GitHub user with the specified email address.
async fn search_user_by_email(
    client: &octorust::Client,
    email: &str,
) -> anyhow::Result<octorust::types::SearchUsersResponse> {
    let per_page = 5;
    let page = 1;
    client
        .search()
        .users(
            email,
            octorust::types::SearchUsersSort::Noop,
            octorust::types::Order::Noop,
            per_page,
            page,
        )
        .await
        .context("Failed to search for GitHub user by email address")
}
