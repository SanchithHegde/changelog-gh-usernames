# changelog-gh-usernames

> [!NOTE]
> This project is no longer being maintained and has been archived.
> Moreover, `git-cliff` now includes support for obtaining GitHub username
> information from commits, so I don't see the need to maintain this tool
> anymore.

A simple application to find and replace emails in changelogs with GitHub
usernames.
This was developed to work with [`git-cliff`][git-cliff], but it should be easy
enough to have this working with other similar tools.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Reading changelog from file](#reading-changelog-from-file)
  - [With `git-cliff`](#with-git-cliff)
- [Known issues](#known-issues)
- [Contributing](#contributing)
- [License](#license)

## Features

- Supports reading input from file and standard input.
  This allows for using the app in shell scripts.
- Persistence: Usernames are stored in an SQLite database after the first fetch,
  preventing repeated GitHub API calls for the same users.

## Installation

As of now, you can build and install the application from source.
I'll consider publishing it to the `crates.io` registry if there are a good
number of users using this application, and once I have a better name.

1. Install the Rust toolchain.
   Refer to their [installation docs][rust-install] for more information.
2. Install the application using `cargo`:

   ```shell
   cargo install --git https://github.com/SanchithHegde/changelog-gh-usernames
   ```

## Usage

Generate a
[fine-grained GitHub personal access token][github-personal-access-token]
from your account settings and set the personal access token in the
`GITHUB_TOKEN` environment variable before running the application.
The token need not have any additional permissions, read-only access to public
repositories should suffice since all this application performs is a user search.

```shell
export GITHUB_TOKEN="github_pat_MySecretGithubPersonalAccessToken"
```

Then proceed to run the application.

```text
$ changelog-gh-usernames -h
A simple application to find and replace emails in changelogs with GitHub usernames.

Usage: changelog-gh-usernames [OPTIONS]

Options:
  -f, --input-file <FILE>        Input file to read the input text from
  -d, --database <DATABASE_URI>  Sqlite database file to persist user information. A database will be created if one doesn't already exist [default: sqlite://users.db]
  -i, --in-place                 Flag to specify whether the output should be written back to file, or to stdout
  -h, --help                     Print help
  -V, --version                  Print version
```

### Reading changelog from file

```shell
changelog-gh-usernames -f /path/to/changelog.md
```

Optionally, you can provide the `--in-place` or `-i` flag to have the
application replace email addresses with GitHub usernames in place.

### With [`git-cliff`][git-cliff]

Assuming that you have a configuration file set up for [`git-cliff`][git-cliff]
to write committer emails to file, you can use the application like so
(providing any more flags to either `git-cliff` or `changelog-gh-usernames` as
required):

```shell
git-cliff | changelog-gh-usernames
```

The changelog with GitHub usernames would be printed to the console.
You can then choose to either pipe the output to a tool that copies to system
clipboard, or redirect the output to a file, as required.

This integration is expected to be much better once the
[PR for integrating post-processors][git-cliff-post-processors-pr] gets merged.

## Known issues

- This will not work if the committer email address is not their primary public
  email address linked to their GitHub account, since the GitHub API does not
  return any results in this case.
  This is also the case if the user has changed their primary public email
  address on their GitHub account since they made the commit.

  A possible temporary solution to this is to manually insert their details in
  the database if their GitHub username is known.

## Contributing

Feel free to open issues or discussions if you'd like to provide feedback or
suggestions.

Suggestions for a better name are also welcome!

## License

Dual licensed under Apache 2.0 or MIT at your option.

See the [LICENSE-APACHE] and [LICENSE-MIT] files for license details.

[git-cliff]: https://github.com/orhun/git-cliff
[rust-install]: https://www.rust-lang.org/tools/install
[github-personal-access-token]: https://github.com/settings/personal-access-tokens/new
[git-cliff-post-processors-pr]: https://github.com/orhun/git-cliff/pull/155
[LICENSE-APACHE]: ./LICENSE-APACHE
[LICENSE-MIT]: ./LICENSE-MIT
