# composer-wire

The [Composer](https://getcomposer.org/) v2 repository **wire format** — the
metadata documents a Composer 2 client and a Composer repository exchange — as
reusable Rust types, plus the `composer/2.0` minify/expand algorithm.

- **`PackageDocument`** — a `p2/{vendor}/{name}.json` document (the per-package
  version list), with `expand()` (apply the minified diff) and `minified()` /
  `flat()` constructors.
- **`RootManifest`** — the root `packages.json` (`metadata-url`,
  `available-packages`, …); unknown keys are preserved on round-trip.
- **`expand_versions`** / **`minify_versions`** — the `composer/2.0`
  minified-metadata algorithm and its exact inverse.

```rust
use composer_wire::PackageDocument;

// `body` is a `p2/<vendor>/<name>.json` response (minified or not).
let by_name = PackageDocument::parse(body)?.expand(); // fully-materialized versions
```

A client parses and *expands* upstream metadata; a server builds and *minifies*
metadata to serve. Sharing one implementation means the producer and consumer of
the wire format can't drift — `expand_versions(minify_versions(v)) == v`.

Part of the [`composer-rs`](https://github.com/cresset-tools/composer-rs)
workspace — shared Composer-domain crates for the cresset-tools family
([bougie](https://github.com/cresset-tools/bougie),
[sconce](https://github.com/cresset-tools/sconce)). Extracted from
`bougie-composer`.

## License

EUPL-1.2.
