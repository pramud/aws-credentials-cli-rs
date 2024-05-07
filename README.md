# Introduction

`aws-credentials-cli` is a command line tool for acquiring temporary AWS
credentials via Microsoft Entra ID using the so called token exchange method.

> Caveat: Even though we believe this works fine in its current state, it is
> still a work in progress. You may find unconditional unwraps, strange module
> arrangement, strange imports etc. Any and all input is appreciated, so please
> don't hesitate to open an issue or a PR!

# Prerequisites

To use `aws-credentials-cli` the following steps must be completed:

1. Install AWS CLI. See instructions in the [AWS
  documentation](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html).
1. Install Azure CLI. See instructions in the [Azure
  documentation](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli).
1. Follow the steps in the [TokenExchange
   documentation](https://github.com/LEGO/IAM-CommonTools-OIDC2SAML-TokenExchange/tree/main/Examples).
   This must be done for each account you need access to.


# Usage

`aws-credentials-cli` has three modes for handling the temporary credentials. These are:

* a mode primarily for use in the `credential_process` option for AWS `config`
  file sections
* a mode for storing the credentials in a section of the AWS config files
* a mode for use cases where maintaining or in general writing to files is not
  feasible.

The first of these modes is highly recommended. See [the AWS documentation for
the `credential_process`
option](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sourcing-external.html)
and [the documentation for the `assume` subcommand](#assume) below for more information.

Only one of these modes can be used in a run of `aws-credentials-cli`. Each of
these modes are described in separate sections below.

`aws-credentials-cli` is based on subcommands like, e.g., the Git CLI.

After a successful run the credentials are cached, and automatically refreshed
when needed on subsequent runs.

## Common Options

These options are common to `aws-credentials-cli` itself and all subcommands.
They will therefore not be repeated for each subcommand.

`-v`, `--verbose...`: Increase logging verbosity. Can be used multiple times, e.g., `-vv`. Each added `v` increases verbosity.

`-q`, `--quiet...`: Decrease logging verbosity. Can be used multiple times, e.g., `-qq`. Each added `q` decreases verbosity.

`-h`, `--help`: Print help. Each subcommand has its own help text.

## Top-level Subcommands

The available subcommands are:

`assume`:  Assume a role and output or store the credentials.

`cache`: Manage the cached credentials.

`generate-completions`: Generate shell completions for your convenience.

---

### `assume`

This subcommand is for assuming a role and outputting or storing the credentials.

Usage:
```shell
aws-credentials-cli assume [OPTIONS] --account <ACCOUNT> --role <ROLE> [COMMAND]
```

#### `assume` Options

`-a`, `--account <ACCOUNT>`: The account on which to assume a role to get
temporary credentials.

`-r`, `--role <ROLE_NAME>`: The role to assume. The role name is the actual
name of the role, i.e., without the `role/` prefix.

`-d`, `--duration <DURATION>`: The AWS session duration in seconds. Must be
minimum 900 seconds (15 minutes). The default is 3600 seconds (1 hour).

`--region <REGION>`: The region to use. The default is `eu-west-1`.

`-f`, `--force`: Force fetching new credentials regardless of non-expired cached credentials.

`-aws-partition <AWS_PARTITION>`: The AWS partition for the account. The
default is `aws`. The possible values are `aws`, `aws-cn`, and `aws-us-gov`.

#### `assume` Subcommands

##### `json`

Usage:

```shell
aws-credentials-cli assume --account <ACCOUNT> --role <ROLE> json
```

Output in JSON format to standard output. The JSON output is suitable
for use in the `credential_process` option in the AWS config file. When using
this in combination with the `credential_process` option, you never have to
remember to refresh the credentials again.

This is the default output format.

###### Example

Let's say that you want to get a list of S3 buckets in an account.

If you have a profile in your AWS config file like this:

```ini
[profile my-profile]
credential_process = /path/to/aws-credentials-cli assume --account 123456789012 --role my-role json
```

and either run `aws --profile my-profile s3 ls` or set the `AWS_PROFILE`
environment variable to `my-profile` and run `aws s3 ls`, then the AWS CLI will
use the `my-profile` profile to get temporary credentials for that profile.

Setting  `AWS_PROFILE` is recommended if you use other tools that use the AWS
API, e.g., Terraform.


##### `credentials-file`
Usage: 
```shell
aws-credentials-cli assume --account <ACCOUNT> --role <ROLE> credentials-file [OPTIONS]
```

Output to the AWS config and credentials files.

You will need to refresh the credentials by running the command again when they
expire.

If you use this with the default profile name `default` then you can use the
AWS CLI and other tools using the AWS API without any additional configuration.
If you use a custom profile name then you must use the `--profile` option for
AWS CLI commands or set the `AWS_PROFILE` environment variable to the custom
profile name for this and other tools using the AWS API, e.g. Terraform.

###### Options

`--config-file <CONFIG_FILE>`: The config file to write to. If
the file does not exist it will be created. The default is `~/.aws/config`.

`--credentials-file <CREDENTIALS_FILE>`: The AWS credentials file to write to.
The default is `~/.aws/credentials`.

`-p`, `--profile <PROFILE>`: The profile to write to. The default profile name
is `default`.

###### Example

After running the following command:
```shell
aws-credentials-cli assume --account 123456789012 --role my-role credentials-file
```

the following will be written to the `~/.aws/config` file:
```ini
[default]
region = eu-west-1
```

and the following will be written to the `~/.aws/credentials` file:
```ini
[default]
aws_access_key_id = <SOME_ACCESS_KEY_ID>
aws_secret_access_key = <SOME_SECRET_ACCESS_KEY>
aws_session_token = <SOME_SESSION_TOKEN>
```

If you add, e.g., the `--profile my-profile` option to the command, then the
profile name will be `my-profile` instead of `default`.


 ##### `env-vars`

Output shell commands to set environment variables. Supported output formats
are `sh` (default) and `powershell`.

Usage: 
```sh
aws-credentials-cli assume --account <ACCOUNT> --role <ROLE> env-vars [OPTIONS]
```
###### Options

`-s`, `--style <STYLE>`: The shell type to output for. Possible values are `sh`
and `powershell`. The default is `sh`.

Use this output in
combination with `eval` in `sh`-like shells or `Invoke-Expression (iex)` in
PowerShell to actually set the environment variables.

###### Examples

After running the following command:
```shell
aws-credentials-cli assume --account 123456789012 --role my-role env-vars
```

the following will be output:
```sh
export AWS_ACCESS_KEY_ID=<SOME_ACCESS_KEY_ID>
export AWS_SECRET_ACCESS_KEy=<SOME_SECRET_ACCESS_KEY>
export AWS_SESSION_TOKEN=<SOME_SESSION_TOKEN>
export AWS_REGION=eu-west-1
export AWS_DEFAULT_REGION=eu-west-1
```

After running the following command:
```shell
aws-credentials-cli assume --account 123456789012 --role my-role env-vars
```

the following will be output:

```powershell
$env:AWS_ACCESS_KEY_ID="<SOME_ACCESS_KEY_ID>"
$env:AWS_SECRET_ACCESS_KEy="<SOME_SECRET_ACCESS_KEY>"
$env:AWS_SESSION_TOKEN="<SOME_SESSION_TOKEN>"
$env:AWS_REGION="eu-west-1"
$env:AWS_DEFAULT_REGION="eu-west-1"
```

To set the environment variables in the current shell, run the following for `sh`-like shells:

```sh
eval $(aws-credentials-cli assume --account 123456789012 --role my-role env-vars)
```

and the following for PowerShell:

```sh
aws-credentials-cli assume --account <ACCOUNT> --role <ROLE> env-vars --style powershell | iex
```

`iex` is a alias for `Invoke-Expression`.

---

### `cache`

This subcommand is for managing cached credentials.

Usage:
```shell
aws-credentials-cli cache [COMMAND]
```

#### `cache` Subcommands:

##### `path`
Prints the path to the cache file to standard output. This is the default subcommand.

##### `clear`
Clears the cache directory. By default the user is asked for confirmation before doing this.

###### `clear` Options:

`-y`, `--yes`: Do not ask for confirmation before clearing the cache directory.

---

### `generate-completions`

Generate completion scripts for a supported shell. Save the output in a
suitable file for your shell and run the intitialization command for the
completion system in the selected shell.

Usage:
```shell
aws-credentials-cli generate-completions [OPTIONS] <SHELL>
```
The value of the `<SHELL>` argument must be one of `bash`, `elvish`, `fish`, `powershell`, `zsh`.

#### Options

`-o`, `--output <OUTPUT>`: Write output to this file. Use `-` for standard output (default).
