# `aws-pass`

## Summary

`aws-pass` is a password manager built in Rust and ontop of AWS SecretsManager.  It is heavily inspired by the Linux tool [`pass`](https://linux.die.net/man/1/pass).

**NOTE:** Setting this tool up requires an AWS IAM User's credentials to be saved locally in a way similar to how you store your usual AWS credentials.  Please make sure these additional credentials have the minimum required access (only access to SecretsManager under MFA).

## Quick Start

### Installation

To install `aws-pass` you can download this source repository and build it via

```
cargo build
```

The built executable will be located in `target/release/aws-pass`.

### Setup

`aws-pass` requires some minor setup.  You will need an AWS Account that includes an MFA-enabled IAM User for which you have credentials (to be provided to `aws-pass` in initialization) with the following policy attached.

```
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "VisualEditor0",
            "Effect": "Allow",
            "Action": [
                "secretsmanager:UntagResource",
                "secretsmanager:DescribeSecret",
                "secretsmanager:PutSecretValue",
                "secretsmanager:CreateSecret",
                "secretsmanager:DeleteSecret",
                "secretsmanager:CancelRotateSecret",
                "secretsmanager:ListSecretVersionIds",
                "secretsmanager:UpdateSecret",
                "secretsmanager:GetRandomPassword",
                "secretsmanager:GetSecretValue",
                "secretsmanager:RestoreSecret",
                "secretsmanager:RotateSecret",
                "secretsmanager:UpdateSecretVersionStage",
                "secretsmanager:ListSecrets",
                "secretsmanager:TagResource"
            ],
            "Resource": "*",
            "Condition": {
                "NumericLessThanEquals": {
                    "aws:MultiFactorAuthAge": "30"
                },
                "BoolIfExists": {
                    "aws:MultiFactorAuthPresent": "true"
                }
            }
        }
    ]
}
```

## Usage

`aws-pass` has seven commands: `init`, `list`, `show`, `insert`, `edit`, `generate`, and `remove`.  Each of these commands will ask for an MFA 

### `init`

```
aws-pass init
```

The `init` command should be run only once and with your MFA-enabled IAM User credentials and MFA token serial handy.  The `init` command will ask for the AWS Access Key Id and AWS Secret Key and save them to `$PASSWORD_STORE_DIR/.credentials`.  `$PASSWORD_STORE_DIR` is `$HOME/.aws-pass` by default.

The credentials stored in this credentials file will be used for making calls to AWS SecretsManager under MFA.

### `list`

```
aws-pass list [--prefix <prefix>]
```

The `list` command lists the passwords in your store, optionally filtering by the provided `prefix`.

### `show`

```
aws-pass show --name <name>
```

The `show` command prints the password's value to stdout for the provided password name.

### `insert`

```
aws-pass insert --name <name>
```

The `insert` command inserts as password into the store under the provided name and with a value collected from stdin.  Collecting the value from stdin ensures that the password is not saved to command history.

### `edit`

```
aws-pass edit --name <name>
```

The `edit` command edits the value of the password for the given name.  Editing the password takes place in the editor specified by `$EDITOR` by opening a temporary file that is deleted once closed (this functionality provided by the [`edit`](https://crates.io/crates/edit) crate).

### `generate`

```
aws-pass generate \
  [--exclude-chars <exclude-chars>] \
  [--length <length] \
  --name <name>
```

The `generate` command generates and inserts a password into the store with the provided name.  Optionally allows characters to be excluded when generating and optionally allows specifying the generated value length.

### `remove`

```
aws-pass remove --name <name>
```

The `remove` command removes the password for the provided name from the store.

## Improvements

The following is a list of improvements for the tool for which I welcome help implementing.

* Hide the password as it's being entered into stdin.  Ask for it twice to confirm the value.
* Allow copying a password's value to the clipboard without showing it on stdout.  Auto-clearing the clipboard after 30 seconds.
* Adding an interactive session so that re-entering an MFA token is not required between comands.

## About and Motivation

I wrote `aws-pass` because I was tired of attempting to use the Linux tool `pass` on multiple different computers and needing to export and import my gpg key.  With `aws-pass`, I'm able to access my passwords from anywhere as long as I have my MFA token.

Additionally, I used this project in order to learn Rust which I am still very much a novice at writing.  If you are a more seasoned Rustacean, please feel free to make suggestions on how to improve the code.