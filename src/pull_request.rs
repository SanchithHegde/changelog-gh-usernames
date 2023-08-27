/// Regex used to search for pull request links in the provided input.
static PULL_REQUEST_LINK_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(
    || {
        #[allow(clippy::expect_used)]
        // No point in proceeding if the email search regex is itself invalid
        regex::Regex::new(
            // Reference: https://stackoverflow.com/a/59082561
            r"github\.com/(?P<owner>[\w.-]+)/(?P<repository>[\w.-]+)/pull/(?P<pull_request_number>\d+)",
        )
        .expect("Invalid pull request regex")
    },
);

/// A pull request
#[derive(Debug)]
pub(crate) struct PullRequest {
    pub(crate) owner: String,
    pub(crate) repository: String,
    pub(crate) pull_request_number: i64,
}

/// Returns a list of identified pull requests in the provided input text.
pub(crate) fn find_pull_requests(input: impl AsRef<str>) -> Vec<PullRequest> {
    PULL_REQUEST_LINK_REGEX
        .captures_iter(input.as_ref())
        .map(|capture| {
            #[allow(clippy::expect_used)]
            let owner = capture
                .name("owner")
                .expect("Pull request owner must be included in pull request URL")
                .as_str()
                .to_owned();
            #[allow(clippy::expect_used)]
            let repository = capture
                .name("repository")
                .expect("Pull request repository must be included in pull request URL")
                .as_str()
                .to_owned();
            #[allow(clippy::expect_used)]
            let pull_request_number = capture
                .name("pull_request_number")
                .expect("Pull request number must be included in pull request URL")
                .as_str()
                .parse()
                .expect("Pull request number must be an integer");

            PullRequest {
                owner,
                repository,
                pull_request_number,
            }
        })
        .collect()
}
