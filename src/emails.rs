//! Utilities to find emails in the provided input text.

/// Regex used to search for email addresses in the provided input.
static EMAIL_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    #[allow(clippy::expect_used)] // No point in proceeding if the email search regex is itself invalid
    regex::Regex::new(
        // Reference: https://stackoverflow.com/a/201378
        r#"(?:[a-zA-Z0-9!#$%&'*+/=?^_`{|}~\-\[\]]+(?:\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~\-\[\]]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#
    )
    .expect("Invalid email validation regex")
});

/// Returns a list of identified email addresses in the provided input text.
pub(crate) fn find_emails(input: impl AsRef<str>) -> Vec<String> {
    EMAIL_REGEX
        .captures_iter(input.as_ref())
        .map(|capture| {
            #[allow(clippy::expect_used)] // Capture group 0 must have the entire text match
            capture
                .get(0)
                .expect("Regex capture group 0 must be an email address")
                .as_str()
                .to_owned()
        })
        .collect()
}

// TODO: Add unit tests for regex
