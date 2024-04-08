# A tool for acquiring AWS temporary credentials via AzureAD for SSO

In its current form the tool is primarily meant for use in the `credential_process` option for

## Huge caveat!

For now, this project is purely a "learn and experiment wiht Rust" project. You will find
unconditional `unwrap`s, strange module arrangement, strange `import`s etc. etc.

## Usage

If calling the command directly:
```sh
aws-credentials-cli --help
```

If running via Cargo:
```sh
cargo run -- --help
```
