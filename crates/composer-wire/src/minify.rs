//! The Composer v2 "minified" repository-metadata algorithm.
//!
//! Packagist serves each `/p2/<vendor>/<name>.json` document in minified
//! form (`"minified": "composer/2.0"`): the first version entry is fully
//! expanded and every later entry is a sparse diff against the running
//! accumulation of the entries before it. A value of the literal string
//! `"__unset"` removes an inherited key (NOT JSON `null`, which would set
//! the key to null). [`expand_versions`] materializes a minified list;
//! [`minify_versions`] is its exact inverse.
//!
//! Mirrors Composer's `Composer\MetadataMinifier\MetadataMinifier`.

use serde_json::{Map, Value};

/// The `minified` marker value Packagist sets for the diff format.
pub const MINIFIED_MARKER: &str = "composer/2.0";

/// Deletion sentinel inside a minified diff entry. A key whose value is
/// exactly this string is removed from the accumulator on expansion.
pub const UNSET: &str = "__unset";

/// Expand a minified `composer/2.0` version list (newest-first) into a
/// list of fully-materialized version objects.
///
/// Each entry is layered onto a running accumulator: present keys
/// overwrite, [`UNSET`] removes, absent keys inherit. The returned list
/// has one fully-expanded object per input entry, in the same order.
#[must_use]
pub fn expand_versions(versions: Vec<Map<String, Value>>) -> Vec<Map<String, Value>> {
    let mut acc: Map<String, Value> = Map::new();
    let mut out = Vec::with_capacity(versions.len());
    for diff in versions {
        apply_diff(&mut acc, diff);
        out.push(acc.clone());
    }
    out
}

/// Minify a fully-materialized version list (newest-first) into the
/// `composer/2.0` sparse-diff form. Inverse of [`expand_versions`]:
/// `expand_versions(minify_versions(v)) == v` for any well-formed `v`
/// (compared as key/value sets — key *order* is not preserved).
///
/// The first entry is emitted whole; each later entry carries only the
/// keys that changed versus the entry before it, plus an [`UNSET`]
/// sentinel for every key the previous entry had that this one drops.
#[must_use]
pub fn minify_versions(versions: &[Map<String, Value>]) -> Vec<Map<String, Value>> {
    let mut out: Vec<Map<String, Value>> = Vec::with_capacity(versions.len());
    for (i, cur) in versions.iter().enumerate() {
        if i == 0 {
            out.push(cur.clone());
            continue;
        }
        let prev = &versions[i - 1];
        let mut diff = Map::new();
        // Changed or newly-added keys (relative to the previous entry).
        for (k, v) in cur {
            if prev.get(k) != Some(v) {
                diff.insert(k.clone(), v.clone());
            }
        }
        // Keys present in the previous entry but gone in this one → unset.
        for k in prev.keys() {
            if !cur.contains_key(k) {
                diff.insert(k.clone(), Value::String(UNSET.to_owned()));
            }
        }
        out.push(diff);
    }
    out
}

/// Apply one minified-diff entry onto the running accumulator. A value of
/// [`UNSET`] removes the key; anything else (including JSON `null`)
/// overwrites verbatim.
fn apply_diff(acc: &mut Map<String, Value>, diff: Map<String, Value>) {
    for (k, v) in diff {
        if v.as_str() == Some(UNSET) {
            acc.remove(&k);
        } else {
            acc.insert(k, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => panic!("expected a JSON object"),
        }
    }

    #[test]
    fn expand_inherits_and_overrides() {
        let versions = vec![
            obj(json!({"name":"a/b","version":"2.0.0","require":{"php":">=8"}})),
            obj(json!({"version":"1.0.0"})),
        ];
        let expanded = expand_versions(versions);
        assert_eq!(expanded[1]["name"], json!("a/b"));
        assert_eq!(expanded[1]["require"]["php"], json!(">=8"));
        assert_eq!(expanded[1]["version"], json!("1.0.0"));
    }

    #[test]
    fn minify_is_inverse_of_expand() {
        // A fully-materialized version list (what a server holds).
        let full = vec![
            obj(
                json!({"name":"a/b","version":"3.0.0","type":"library","require":{"php":">=8.1"},"dist":{"type":"zip","url":"u3"}}),
            ),
            obj(
                json!({"name":"a/b","version":"2.0.0","type":"library","require":{"php":">=8.1"},"dist":{"type":"zip","url":"u2"}}),
            ),
            obj(
                json!({"name":"a/b","version":"1.0.0","type":"library","require":{"php":">=7.4"},"dist":{"type":"zip","url":"u1"}}),
            ),
        ];
        let mini = minify_versions(&full);
        // The second entry is sparse: name/type/require unchanged, omitted.
        assert!(!mini[1].contains_key("name"));
        assert!(!mini[1].contains_key("type"));
        assert!(!mini[1].contains_key("require"));
        assert!(mini[1].contains_key("version"));
        // The third entry's require changed, so it is carried.
        assert!(mini[2].contains_key("require"));
        // And it round-trips back to the originals (order-independent eq).
        assert_eq!(expand_versions(mini), full);
    }

    #[test]
    fn minify_emits_unset_for_dropped_keys() {
        let full = vec![
            obj(json!({"name":"a/b","version":"2.0.0","require":{"php":">=8"}})),
            obj(json!({"name":"a/b","version":"1.0.0"})), // dropped `require`
        ];
        let mini = minify_versions(&full);
        assert_eq!(mini[1]["require"], json!("__unset"));
        assert_eq!(expand_versions(mini), full);
    }

    #[test]
    fn unset_in_input_removes_key() {
        let versions = vec![
            obj(json!({"name":"a/b","version":"2.0.0","require":{"php":">=8"}})),
            obj(json!({"version":"1.0.0","require":"__unset"})),
        ];
        let expanded = expand_versions(versions);
        assert!(expanded[1].get("require").is_none());
        assert_eq!(expanded[1]["name"], json!("a/b"));
    }
}
