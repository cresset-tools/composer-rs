//! Composer v2 repository wire documents: the per-package `p2/*.json`
//! ([`PackageDocument`]) and the root `packages.json` ([`RootManifest`]).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use crate::minify::{MINIFIED_MARKER, expand_versions, minify_versions};

/// A `/p2/<vendor>/<name>.json` document: a map (usually single-entry,
/// keyed by `vendor/name`) of version lists. When
/// [`minified`](Self::minified) is `Some("composer/2.0")`, the version
/// lists are sparse diffs (see [`crate::minify`]); otherwise each entry
/// stands alone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageDocument {
    /// The minified-format marker. `Some("composer/2.0")` means the
    /// `packages` version lists are diffs; any other value (or `None`)
    /// means they stand alone.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub minified: Option<String>,
    /// `vendor/name` → version list (Packagist orders newest-first).
    /// Each version is a raw object so the minify/expand pass can run
    /// before any typed deserialization.
    #[serde(default)]
    pub packages: BTreeMap<String, Vec<Map<String, Value>>>,
}

impl PackageDocument {
    /// Parse a `/p2/` document from JSON bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Serialize to compact JSON bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Build a non-minified document: each version stands alone.
    #[must_use]
    pub fn flat(packages: BTreeMap<String, Vec<Map<String, Value>>>) -> Self {
        Self {
            minified: None,
            packages,
        }
    }

    /// Build a minified `composer/2.0` document from fully-materialized
    /// version lists, applying [`minify_versions`] to each package.
    #[must_use]
    pub fn minified(packages: BTreeMap<String, Vec<Map<String, Value>>>) -> Self {
        let packages = packages
            .into_iter()
            .map(|(name, versions)| (name, minify_versions(&versions)))
            .collect();
        Self {
            minified: Some(MINIFIED_MARKER.to_owned()),
            packages,
        }
    }

    /// Expand into fully-materialized version objects, honoring the
    /// [`minified`](Self::minified) marker. A document whose marker is
    /// anything other than `composer/2.0` is returned as-is (each entry
    /// already stands alone) — defensive against a future format bump.
    #[must_use]
    pub fn expand(self) -> BTreeMap<String, Vec<Map<String, Value>>> {
        let is_minified = self.minified.as_deref() == Some(MINIFIED_MARKER);
        self.packages
            .into_iter()
            .map(|(name, versions)| {
                let expanded = if is_minified {
                    expand_versions(versions)
                } else {
                    versions
                };
                (name, expanded)
            })
            .collect()
    }
}

/// The root `packages.json` of a Composer v2 repository. Tells a client
/// where to fetch per-package metadata ([`metadata_url`](Self::metadata_url),
/// a template with a `%package%` placeholder) and, optionally, which
/// packages exist. Unknown root keys round-trip through
/// [`extra`](Self::extra).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootManifest {
    /// `metadata-url` — the per-package metadata template, e.g.
    /// `https://repo.test/p2/%package%.json`.
    #[serde(
        rename = "metadata-url",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub metadata_url: Option<String>,
    /// `available-packages` — the full list of package names served.
    #[serde(
        rename = "available-packages",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub available_packages: Option<Vec<String>>,
    /// `available-package-patterns` — wildcard patterns for served names.
    #[serde(
        rename = "available-package-patterns",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub available_package_patterns: Option<Vec<String>>,
    /// `providers-url` — the legacy v1 provider template, if advertised.
    #[serde(
        rename = "providers-url",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub providers_url: Option<String>,
    /// Any other root keys (e.g. `notify-batch`, `search`, `mirrors`,
    /// `provider-includes`), preserved verbatim on round-trip.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl RootManifest {
    /// The v2 `metadata-url` template for a repository served at
    /// `base_url`: `<base>/p2/%package%.json` (trailing slash trimmed).
    #[must_use]
    pub fn metadata_template(base_url: &str) -> String {
        format!("{}/p2/%package%.json", base_url.trim_end_matches('/'))
    }

    /// Build a minimal v2 root manifest: a `metadata-url` for `base_url`
    /// plus an `available-packages` list.
    #[must_use]
    pub fn v2(base_url: &str, available_packages: Vec<String>) -> Self {
        Self {
            metadata_url: Some(Self::metadata_template(base_url)),
            available_packages: Some(available_packages),
            ..Default::default()
        }
    }

    /// Parse a root `packages.json` from JSON bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Serialize to compact JSON bytes.
    pub fn to_vec(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_a_single_fully_expanded_version() {
        let body = br#"{ "packages": { "monolog/monolog": [
            {"name":"monolog/monolog","version":"3.0.0","require":{"php":">=8.1"}}
        ] } }"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        let v = &expanded["monolog/monolog"];
        assert_eq!(v.len(), 1);
        assert_eq!(v[0]["version"], json!("3.0.0"));
        assert_eq!(v[0]["require"]["php"], json!(">=8.1"));
    }

    #[test]
    fn expands_minified_composer_2_0_inheritance() {
        let body = br#"{ "minified":"composer/2.0", "packages": { "acme/foo": [
            {"name":"acme/foo","version":"3.0.0","type":"library","dist":{"type":"zip","url":"https://e/3.0.0.zip"},"require":{"php":">=8.1"}},
            {"version":"2.5.0","dist":{"type":"zip","url":"https://e/2.5.0.zip"}},
            {"version":"2.0.0","dist":{"type":"zip","url":"https://e/2.0.0.zip"},"require":{"php":">=7.4"},"require-dev":{"phpunit/phpunit":"^9"}}
        ] } }"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        let v = &expanded["acme/foo"];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0]["version"], json!("3.0.0"));
        // v1 inherits name + type + require; overrides version + dist.
        assert_eq!(v[1]["version"], json!("2.5.0"));
        assert_eq!(v[1]["name"], json!("acme/foo"));
        assert_eq!(v[1]["type"], json!("library"));
        assert_eq!(v[1]["require"]["php"], json!(">=8.1"));
        assert_eq!(v[1]["dist"]["url"], json!("https://e/2.5.0.zip"));
        // v2 overrides require, adds require-dev, still inherits name+type.
        assert_eq!(v[2]["require"]["php"], json!(">=7.4"));
        assert_eq!(v[2]["require-dev"]["phpunit/phpunit"], json!("^9"));
        assert_eq!(v[2]["name"], json!("acme/foo"));
    }

    #[test]
    fn unset_sentinel_resets_inherited_key() {
        let body = br#"{ "minified":"composer/2.0", "packages": { "acme/bar": [
            {"name":"acme/bar","version":"2.0.0","require":{"php":">=8.0"}},
            {"version":"1.0.0","require":"__unset"}
        ] } }"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        let v = &expanded["acme/bar"];
        assert_eq!(v[1]["version"], json!("1.0.0"));
        assert!(v[1].get("require").is_none());
        assert_eq!(v[1]["name"], json!("acme/bar"));
    }

    #[test]
    fn non_minified_response_is_returned_as_is() {
        let body = br#"{ "packages": { "acme/baz": [
            {"name":"acme/baz","version":"2.0.0","require":{"php":">=8.0"}},
            {"name":"acme/baz","version":"1.0.0"}
        ] } }"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        let v = &expanded["acme/baz"];
        assert_eq!(v[0]["require"]["php"], json!(">=8.0"));
        assert!(v[1].get("require").is_none());
    }

    #[test]
    fn unknown_minified_marker_is_treated_as_non_minified() {
        let body = br#"{ "minified":"composer/2.1", "packages": { "acme/qux": [
            {"name":"acme/qux","version":"2.0.0"}
        ] } }"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        assert_eq!(expanded["acme/qux"].len(), 1);
        assert_eq!(expanded["acme/qux"][0]["version"], json!("2.0.0"));
    }

    #[test]
    fn empty_packages_map_parses() {
        let body = br#"{"minified":"composer/2.0","packages":{}}"#;
        let expanded = PackageDocument::parse(body).unwrap().expand();
        assert!(expanded.is_empty());
    }

    #[test]
    fn malformed_json_errors() {
        assert!(PackageDocument::parse(br"{not json").is_err());
    }

    #[test]
    fn minified_constructor_round_trips_through_expand() {
        let mut packages = BTreeMap::new();
        packages.insert(
            "acme/foo".to_owned(),
            vec![
                match json!({"name":"acme/foo","version":"2.0.0","require":{"php":">=8"}}) {
                    Value::Object(m) => m,
                    _ => unreachable!(),
                },
                match json!({"name":"acme/foo","version":"1.0.0","require":{"php":">=8"}}) {
                    Value::Object(m) => m,
                    _ => unreachable!(),
                },
            ],
        );
        let doc = PackageDocument::minified(packages.clone());
        assert_eq!(doc.minified.as_deref(), Some("composer/2.0"));
        // The serialized-then-parsed document expands back to the input.
        let bytes = doc.to_vec().unwrap();
        let expanded = PackageDocument::parse(&bytes).unwrap().expand();
        assert_eq!(expanded, packages);
    }

    #[test]
    fn root_manifest_v2_render_and_round_trip() {
        let root = RootManifest::v2(
            "https://r.test/",
            vec!["acme/widget".to_owned(), "acme/gadget".to_owned()],
        );
        assert_eq!(
            root.metadata_url.as_deref(),
            Some("https://r.test/p2/%package%.json")
        );
        let back = RootManifest::parse(&root.to_vec().unwrap()).unwrap();
        assert_eq!(back.metadata_url, root.metadata_url);
        assert_eq!(back.available_packages, root.available_packages);
    }

    #[test]
    fn root_manifest_preserves_unknown_fields() {
        let body = br#"{"metadata-url":"/p2/%package%.json","notify-batch":"https://r/notify"}"#;
        let root = RootManifest::parse(body).unwrap();
        assert_eq!(root.metadata_url.as_deref(), Some("/p2/%package%.json"));
        assert_eq!(root.extra["notify-batch"], json!("https://r/notify"));
        let back = RootManifest::parse(&root.to_vec().unwrap()).unwrap();
        assert_eq!(back.extra["notify-batch"], json!("https://r/notify"));
    }
}
