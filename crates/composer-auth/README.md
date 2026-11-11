# composer-auth

Composer repository credentials, without an HTTP client.

- `AuthCredentials`: one host's credential (`http-basic`, `bearer`,
  `github-oauth`, `gitlab-token`, `gitlab-oauth`), with the header name and
  value Composer sends for it. Its `Debug` output never shows the secret.
- `parse_auth_json` / `parse_auth_object`: read the `auth.json` shape (also
  used by `COMPOSER_AUTH` and composer.json's `config` block) into a map keyed
  by host.
- `auth_key`: the key a URL's credential is looked up under (host and port).

```rust
use composer_auth::{auth_key, parse_auth_json};

let auth = parse_auth_json(r#"{"bearer": {"repo.example.com": "tok"}}"#, "auth.json").unwrap();
let creds = &auth[auth_key("https://repo.example.com/p2/a/b.json").unwrap()];
assert_eq!(creds.header_name(), "authorization");
assert_eq!(creds.header_value(), "Bearer tok");
```

Where the credentials come from (which files, which environment variables, in
what order) is left to the caller.
