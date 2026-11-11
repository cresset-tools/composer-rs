//! Composer repository credentials, without an HTTP client.
//!
//! [`AuthCredentials`] is one host's credential and renders the header
//! Composer sends for it; [`parse_auth_object`] reads Composer's auth shape
//! (top level of `auth.json` and `COMPOSER_AUTH`, `config` in composer.json)
//! into a map keyed by host; [`auth_key`] is the key a URL is looked up under.
//! Reading files and the environment is left to the caller.

use std::collections::HashMap;
use std::fmt;

use base64::Engine as _;
use serde_json::{Map, Value};

/// One repository host's credential. `Debug` never shows the secret.
#[derive(Clone, PartialEq, Eq)]
pub enum AuthCredentials {
    /// HTTP basic auth — Composer's `http-basic`.
    Basic { username: String, password: String },
    /// A bearer token — Composer's `bearer` (and `gitlab-oauth`).
    Bearer { token: String },
    /// A GitHub token — Composer's `github-oauth`, sent as `token <token>`.
    GitHubToken { token: String },
    /// A GitLab private or CI token — Composer's `gitlab-token`, sent in the
    /// `PRIVATE-TOKEN` header.
    GitLabToken { token: String },
}

impl fmt::Debug for AuthCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            Self::Bearer { .. } => f
                .debug_struct("Bearer")
                .field("token", &"<redacted>")
                .finish(),
            Self::GitHubToken { .. } => f
                .debug_struct("GitHubToken")
                .field("token", &"<redacted>")
                .finish(),
            Self::GitLabToken { .. } => f
                .debug_struct("GitLabToken")
                .field("token", &"<redacted>")
                .finish(),
        }
    }
}

impl AuthCredentials {
    /// HTTP basic auth from `username:password`, split at the first `:` (a
    /// password may contain more). A secret without `:` is a username with
    /// an empty password.
    pub fn basic_from_userinfo(userinfo: &str) -> Self {
        let (username, password) = userinfo.split_once(':').unwrap_or((userinfo, ""));
        Self::Basic {
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    /// The HTTP header *name* this credential is sent in (lowercase).
    pub fn header_name(&self) -> &'static str {
        match self {
            Self::Basic { .. } | Self::Bearer { .. } | Self::GitHubToken { .. } => "authorization",
            Self::GitLabToken { .. } => "private-token",
        }
    }

    /// The HTTP header *value* for this credential.
    pub fn header_value(&self) -> String {
        match self {
            Self::Basic { username, password } => {
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{username}:{password}"));
                format!("Basic {encoded}")
            }
            Self::Bearer { token } => format!("Bearer {token}"),
            Self::GitHubToken { token } => format!("token {token}"),
            Self::GitLabToken { token } => token.clone(),
        }
    }
}

/// A malformed auth document. The message names where the value came from
/// and the offending key path, never a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthError(String);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AuthError {}

/// Parse an `auth.json` document (or the `COMPOSER_AUTH` value). Empty or
/// whitespace-only input is no credentials. `source` names the document in
/// error messages.
pub fn parse_auth_json(
    raw: &str,
    source: &str,
) -> Result<HashMap<String, AuthCredentials>, AuthError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(HashMap::new());
    }
    let value: Value = serde_json::from_str(raw)
        .map_err(|e| AuthError(format!("{source} is not valid JSON: {e}")))?;
    let obj = value
        .as_object()
        .ok_or_else(|| AuthError(format!("{source} must be a JSON object")))?;
    parse_auth_object(obj, source)
}

/// Parse Composer's auth object — `{http-basic, bearer, github-oauth,
/// gitlab-token, gitlab-oauth}`, each a map from host to credential — into a
/// map keyed by host. Other keys are ignored, so composer.json's whole
/// `config` block can be passed. `source` prefixes each error message.
pub fn parse_auth_object(
    obj: &Map<String, Value>,
    source: &str,
) -> Result<HashMap<String, AuthCredentials>, AuthError> {
    let err = |msg: String| AuthError(format!("{source}: {msg}"));
    let section = |key: &str| {
        obj.get(key)
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
    };
    let string_token = |key: &str, host: &str, val: &Value| {
        val.as_str()
            .map(str::to_owned)
            .ok_or_else(|| err(format!("{key}.{host} must be a string token")))
    };

    let mut out = HashMap::new();
    for (host, val) in section("bearer") {
        let token = string_token("bearer", host, val)?;
        out.insert(host.clone(), AuthCredentials::Bearer { token });
    }
    for (host, val) in section("github-oauth") {
        let token = string_token("github-oauth", host, val)?;
        out.insert(host.clone(), AuthCredentials::GitHubToken { token });
    }
    for (host, val) in section("gitlab-oauth") {
        let token = string_token("gitlab-oauth", host, val)?;
        out.insert(host.clone(), AuthCredentials::Bearer { token });
    }
    for (host, val) in section("gitlab-token") {
        let token = match val {
            Value::String(s) => s.clone(),
            Value::Object(o) => o
                .get("token")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    err(format!(
                        "gitlab-token.{host}.token is missing or not a string"
                    ))
                })?
                .to_owned(),
            _ => {
                return Err(err(format!(
                    "gitlab-token.{host} must be a string or an object with `token`"
                )));
            }
        };
        out.insert(host.clone(), AuthCredentials::GitLabToken { token });
    }
    for (host, val) in section("http-basic") {
        let entry = val.as_object().ok_or_else(|| {
            err(format!(
                "http-basic.{host} must be an object with `username` and `password`"
            ))
        })?;
        let field = |name: &str| {
            entry
                .get(name)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| {
                    err(format!(
                        "http-basic.{host}.{name} is missing or not a string"
                    ))
                })
        };
        out.insert(
            host.clone(),
            AuthCredentials::Basic {
                username: field("username")?,
                password: field("password")?,
            },
        );
    }
    Ok(out)
}

/// The key a URL's credential is looked up under, as Composer does: the
/// host, with its port when the URL has one (`repo.magento.com`,
/// `127.0.0.1:8080`). Userinfo is dropped. `None` when there is no host
/// (`file://` URLs, paths).
pub fn auth_key(url: &str) -> Option<&str> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme.split(['/', '?', '#']).next()?;
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    (!host.is_empty()).then_some(host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(v: &Value) -> Result<HashMap<String, AuthCredentials>, AuthError> {
        parse_auth_object(v.as_object().unwrap(), "test")
    }

    #[test]
    fn headers_match_composer() {
        let cases = [
            (
                AuthCredentials::Basic {
                    username: "user".into(),
                    password: "pass".into(),
                },
                "authorization",
                "Basic dXNlcjpwYXNz",
            ),
            (
                AuthCredentials::Bearer { token: "t".into() },
                "authorization",
                "Bearer t",
            ),
            (
                AuthCredentials::GitHubToken { token: "t".into() },
                "authorization",
                "token t",
            ),
            (
                AuthCredentials::GitLabToken { token: "t".into() },
                "private-token",
                "t",
            ),
        ];
        for (creds, name, value) in cases {
            assert_eq!(creds.header_name(), name, "{creds:?}");
            assert_eq!(creds.header_value(), value, "{creds:?}");
        }
    }

    #[test]
    fn basic_from_userinfo_splits_at_the_first_colon() {
        assert_eq!(
            AuthCredentials::basic_from_userinfo("token:a:b"),
            AuthCredentials::Basic {
                username: "token".into(),
                password: "a:b".into()
            }
        );
        assert_eq!(
            AuthCredentials::basic_from_userinfo("alone"),
            AuthCredentials::Basic {
                username: "alone".into(),
                password: String::new()
            }
        );
    }

    #[test]
    fn parses_every_kind() {
        let map = parse(&json!({
            "http-basic": { "repo.example.com": { "username": "u", "password": "p" } },
            "bearer": { "api.example.com": "b" },
            "github-oauth": { "github.com": "g" },
            "gitlab-oauth": { "gitlab.com": "o" },
            "gitlab-token": { "gitlab.example.com": "t", "git.example.org": { "token": "t2" } },
            "preferred-install": "dist"
        }))
        .unwrap();
        assert_eq!(map.len(), 6);
        assert_eq!(map["repo.example.com"].header_value(), "Basic dTpw");
        assert_eq!(map["api.example.com"].header_value(), "Bearer b");
        assert_eq!(map["github.com"].header_value(), "token g");
        assert_eq!(map["gitlab.com"].header_value(), "Bearer o");
        assert_eq!(map["gitlab.example.com"].header_name(), "private-token");
        assert_eq!(map["git.example.org"].header_value(), "t2");
    }

    #[test]
    fn errors_name_the_source_and_key_but_not_the_secret() {
        let e = parse(&json!({"http-basic": {"h": {"username": "u", "password": 5}}})).unwrap_err();
        assert_eq!(
            e.to_string(),
            "test: http-basic.h.password is missing or not a string"
        );
        let e = parse(&json!({"bearer": {"h": 1}})).unwrap_err();
        assert_eq!(e.to_string(), "test: bearer.h must be a string token");
        assert!(
            parse_auth_json("[]", "COMPOSER_AUTH")
                .unwrap_err()
                .to_string()
                .contains("COMPOSER_AUTH")
        );
        assert!(parse_auth_json("  ", "COMPOSER_AUTH").unwrap().is_empty());
    }

    #[test]
    fn debug_redacts_secrets() {
        let creds = AuthCredentials::Basic {
            username: "user".into(),
            password: "s3cret".into(),
        };
        let rendered = format!("{creds:?}");
        assert!(rendered.contains("user"));
        assert!(!rendered.contains("s3cret"), "{rendered}");
    }

    #[test]
    fn auth_key_is_host_and_port() {
        assert_eq!(
            auth_key("https://repo.magento.com/archives/x.zip"),
            Some("repo.magento.com")
        );
        assert_eq!(
            auth_key("http://127.0.0.1:8080/p2/a/b.json"),
            Some("127.0.0.1:8080")
        );
        assert_eq!(
            auth_key("https://user:pw@host.example/x"),
            Some("host.example")
        );
        assert_eq!(auth_key("https://host.example?x=1"), Some("host.example"));
        assert_eq!(auth_key("file:///tmp/x.zip"), None);
        assert_eq!(auth_key("/tmp/x.zip"), None);
    }
}
