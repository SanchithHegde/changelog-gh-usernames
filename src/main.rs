//! A simple application to find and replace emails in changelogs with GitHub usernames.

mod cli;
mod db;
mod emails;
mod pull_request;

use std::fs;

use anyhow::Context;
use once_cell::sync::OnceCell;

use crate::{
    db::{GithubUser, GithubUserRepository},
    pull_request::PullRequest,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = <cli::Args as clap::Parser>::parse();

    let pool = db::create_pool(args.database).await?;
    db::run_migrations(&pool).await?;
    let user_repository = GithubUserRepository::new(pool);
    let client = OnceCell::new();

    let input_lines = read_input_lines(args.input_file.as_ref())?;
    let mut output = Vec::new();
    let mut unavailable_emails = Vec::new();

    for line in input_lines {
        let result = replace_email_with_username(&user_repository, &client, line).await?;
        let replaced_line = match result {
            ReplaceEmailResult::Success { output } => output,
            ReplaceEmailResult::UsernameUnavailable { output, email } => {
                unavailable_emails.push(email);
                output
            }
        };
        output.push(replaced_line);
    }
    let output = output.join("\n");

    match (args.input_file, args.in_place) {
        (Some(file), true) => fs::write(file, output)?,
        _ => println!("{output}"),
    }

    // Logging status messages to stderr allows for piping stdout to other tools or clipboard,
    // while ensuring that status messages are displayed to the user.
    eprintln!("Email addresses replaced with corresponding GitHub usernames.");
    if !unavailable_emails.is_empty() {
        eprintln!(
            "\
            GitHub usernames for the following email addresses are unavailable. \
            Either the email addresses are invalid, or the users updated their publicly visible \
            email addresses recently.\n{}",
            unavailable_emails.join("\n")
        );
    }

    Ok(())
}

/// Read input from the file path if specified, or standard input if the file path is `None`.
fn read_input_lines(file: Option<impl AsRef<std::path::Path>>) -> anyhow::Result<Vec<String>> {
    use std::io::{self, BufRead, BufReader};

    let reader: Box<dyn BufRead> = match file {
        Some(file) => Box::new(BufReader::new(
            fs::File::open(file).context("Failed to open input file")?,
        )),
        None => Box::new(BufReader::new(io::stdin().lock())),
    };

    reader
        .lines()
        .collect::<Result<Vec<String>, _>>()
        .context("Failed to read input")
}

/// Replace emails in specified line with GitHub username.
/// If user search by email fails, try to find a pull request URL and obtain the username of the
/// pull request author.
///
/// The assumption throughout is that each changelog entry is on a single line, and is not split
/// into multiple lines.
async fn replace_email_with_username(
    user_repository: &GithubUserRepository,
    client: &OnceCell<octorust::Client>,
    line: impl AsRef<str>,
) -> anyhow::Result<ReplaceEmailResult> {
    let emails = emails::find_emails(&line);

    match emails.len() {
        0 => Ok(ReplaceEmailResult::Success {
            output: line.as_ref().to_owned(),
        }),
        len if len > 1 => {
            anyhow::bail!("More than one user per changelog entry are not yet supported")
        }
        1 => {
            #[allow(clippy::expect_used)]
            let email = emails.into_iter().next().expect("No email found");

            let username = if let Some(prefix) = email.strip_suffix("@users.noreply.github.com") {
                // GitHub no-reply email address
                // Format: 1234567+GitHubUsername@users.noreply.github.com
                if let Some((_, username)) = prefix.split_once('+') {
                    if let Some(bot_username) = username.strip_suffix("[bot]") {
                        // GitHub bot email address
                        // Format: 1234567+dependabot[bot]@users.noreply.github.com
                        bot_username.to_owned()
                    } else {
                        username.to_owned()
                    }
                } else {
                    eprintln!("Unknown GitHub no-reply email format");
                    return Ok(ReplaceEmailResult::UsernameUnavailable {
                        output: line.as_ref().to_owned(),
                        email,
                    });
                }
            } else {
                // Retrieve username from database if available, query GitHub API otherwise.
                let db_user = user_repository.get(&email).await?;
                if let Some(user) = db_user {
                    user.username
                } else {
                    // Construct GitHub client only when absolutely necessary
                    let users =
                        search_user_by_email(client.get_or_try_init(construct_client)?, &email)
                            .await?;

                    // This shouldn't ideally happen, since GitHub does not allow two accounts to be linked
                    // with the same email address.
                    anyhow::ensure!(
                        users.total_count <= 1,
                        "More than one user found for {email}"
                    );

                    if let Some(user) = users.items.first() {
                        let username = &user.login;

                        let user = GithubUser::new(email.clone(), username.clone());
                        user_repository.insert(user).await?;

                        username.clone()
                    } else {
                        // Search for pull request URLs in changelog entry, if available
                        let pull_requests = pull_request::find_pull_requests(&line);

                        let pull_request = if !pull_requests.is_empty() {
                            // Pick last PR link from commit message as GitHub's squash
                            // merge commit message has the format `{pr_title} ({pr_number})`.
                            // Picking the last PR link would pick the correct PR link even in case
                            // of reverts with format:
                            // `revert: {old_pr_title} ({old_pr_number}) ({current_pr_number})`
                            #[allow(clippy::expect_used)]
                            pull_requests.last().expect("No pull request found")
                        } else {
                            return Ok(ReplaceEmailResult::UsernameUnavailable {
                                output: line.as_ref().to_owned(),
                                email,
                            });
                        };

                        let username =
                            get_pr_author(client.get_or_try_init(construct_client)?, pull_request)
                                .await?;
                        let user = GithubUser::new(email.clone(), username.clone());
                        user_repository.insert(user).await?;

                        username
                    }
                }
            };

            Ok(ReplaceEmailResult::Success {
                output: line.as_ref().replace(&email, &format!("@{username}")),
            })
        }
        // rustc requires this check for usize
        #[allow(clippy::unreachable)]
        _ => unreachable!("Unknown number of emails found in line"),
    }
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
        .map(|response| response.body)
}

/// Find the GitHub user who authored the specified pull request.
async fn get_pr_author(
    client: &octorust::Client,
    pull_request: &PullRequest,
) -> anyhow::Result<String> {
    client
        .pulls()
        .get(
            &pull_request.owner,
            &pull_request.repository,
            pull_request.pull_request_number,
        )
        .await
        .context(format!(
            "Failed to obtain pull request information: {pull_request:?}"
        ))?
        .body
        .user
        .ok_or_else(|| {
            anyhow::anyhow!("The author information for the specified pull request is unavailable")
        })
        .map(|user| user.login)
}

/// The result of attempting to replace an email in a specified changelog entry.
enum ReplaceEmailResult {
    Success { output: String },
    UsernameUnavailable { output: String, email: String },
}
