# Frequently Asked Questions (FAQ)

This section contains answers to the most frequently asked questions and common errors. If you believe something is missing, please feel free to create a new issue or submit a pull request with your suggestions.

## What ROLE should I use in the configuration?

When using the command `aws-credentials-cli assume [OPTIONS] --account <ACCOUNT> --role <ROLE>`, the `<ROLE>` should be the IAM Role on the AWS account. You can find this by logging into the account, navigating to IAM -> Roles, and searching for SSO. The role name you see there is the one you want to assume.

## Common Errors
### The provided token has expired

This error can occur if you have a `~/.aws/credentials` file with a default profile, as the `default` profile always takes precedence over specific profiles. This can also occur if you have the aws profile that you are trying use in the `~/.aws/credentials`.

### Error when retrieving credentials from custom-process: Application error: service error

This error can occur in the following scenarios:

- The token exchange service is down
- The Azure application has not been set up for token exchange. Follow the steps in the [TokenExchange documentation](https://github.com/LEGO/IAM-CommonTools-OIDC2SAML-TokenExchange/tree/main/Examples). This setup must be done for each account you need access to.
- You referred to the wrong role in the `--role` option. Check that the role exists on your AWS account. Look for roles starting with `SSO-` as described in [What ROLE should I use in the configuration?](#what-role-should-i-use-in-the-configuration). Don't confuse the role with the AWS profile name you may have set in the shell.