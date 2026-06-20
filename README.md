# composer-rs

Shared Composer-domain Rust crates for the [cresset-tools](https://github.com/cresset-tools)
family. These factor the parts of the Composer ecosystem that more than one
tool needs — so the [`bougie`](https://github.com/cresset-tools/bougie) package
manager and the [`sconce`](https://github.com/cresset-tools/sconce) repository
server/mirror consume **one** implementation rather than each carrying its own,
slowly drifting copy.

## Crates

| Crate | What it is |
|-------|------------|
| [`composer-semver`](crates/composer-semver) | A faithful Rust port of `composer/semver`: Composer-flavored `Version`, `Constraint`, and `Stability` — normalize, compare, and match version constraints. |

Planned: `composer-wire` — the typed Composer v2 repository metadata
(`/p2/<vendor>/<pkg>.json` documents, `dist`/`source` blocks, the root
`packages.json` shape, and the minified-diff expander).

## composer-semver

```rust
use composer_semver::{Constraint, Version};

let v = Version::parse("v1.2.3-beta2").unwrap();
let c = Constraint::parse("^1.2").unwrap();
assert!(c.matches(&v));
```

The implementation mirrors `composer/semver`'s `VersionParser` and `Comparator`
against a pinned upstream commit (recorded in `crates/composer-semver/src/version.rs`),
and the upstream data-provider cases are committed as conformance fixtures:

```
cargo test -p composer-semver
```

## License

EUPL-1.2. See [LICENSE](LICENSE).
