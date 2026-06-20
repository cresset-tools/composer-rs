# composer-semver

A faithful Rust port of [`composer/semver`](https://github.com/composer/semver) —
the Composer-flavored version normalization, comparison, and constraint algebra
that PHP's Composer itself uses.

```rust
use composer_semver::{Constraint, Version};

let v = Version::parse("v1.2.3-beta2").unwrap();
let c = Constraint::parse("^1.2").unwrap();
assert!(c.matches(&v));
```

It mirrors Composer's `VersionParser::normalize`, `Comparator::compare`, and
`VersionParser::parseConstraints` against a pinned upstream commit (recorded in
`src/version.rs`), and the upstream data-provider cases are committed as Layer 1
conformance fixtures:

```
cargo test -p composer-semver
```

## What's covered

- **`Version`** — normalize (`v1.2` → `1.2.0.0`), branch versions (`dev-*`),
  stability detection, and Composer-correct comparison/ordering.
- **`Constraint`** — `^`, `~`, `*`/`x`, hyphen ranges, `>=`/`<`/`!=`, `||`
  unions, stability flags, and `dev-<branch>` references; `matches`,
  `lower_bound`/`upper_bound`, and `intersects`.
- **`Stability`** — `dev < alpha < beta < RC < stable`.

Part of the [`composer-rs`](https://github.com/cresset-tools/composer-rs)
workspace — shared Composer-domain crates for the cresset-tools family
([bougie](https://github.com/cresset-tools/bougie),
[sconce](https://github.com/cresset-tools/sconce)).

## License

EUPL-1.2.
