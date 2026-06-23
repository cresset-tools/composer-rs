# composer-php-json

Byte-exact [PHP `json_encode`](https://www.php.net/json_encode) output for the
two flag combinations [Composer](https://getcomposer.org/) relies on — so Rust
tooling can reproduce Composer's `content-hash` and its `composer.json` /
`composer.lock` file formatting byte-for-byte.

```rust
use composer_php_json::{encode, Mode};
use serde_json::json;

// Composer's content-hash encoding: compact, escaped slashes, \uXXXX non-ASCII.
let bytes = encode(&json!({"name": "acme/widget"}), Mode::Hash);
assert_eq!(String::from_utf8(bytes).unwrap(), r#"{"name":"acme\/widget"}"#);
```

## Modes

- **`Mode::Hash`** — `json_encode($d, 0)`. The byte stream MD5'd into
  `composer.lock`'s `content-hash`. Compact, `/` → `\/`, every non-ASCII code
  point escaped to lowercase `\uXXXX` (surrogate pairs above U+FFFF).
- **`Mode::Pretty`** — `json_encode($d, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES
  | JSON_UNESCAPED_UNICODE)`, the flag set `Composer\Json\JsonFile::encode` uses
  when writing files. 4-space indent, raw `/`, raw UTF-8 (except U+2028 / U+2029,
  which Composer keeps escaped).

Object key order is preserved as-is — callers that need Composer's content-hash
must `ksort` the top level themselves before encoding, exactly as
`Locker::getContentHash` does.

Part of the [`composer-rs`](https://github.com/cresset-tools/composer-rs)
workspace — shared Composer-domain crates for the cresset-tools family
([bougie](https://github.com/cresset-tools/bougie),
[sconce](https://github.com/cresset-tools/sconce)). Extracted from
`bougie-php-json`.

## License

EUPL-1.2.
