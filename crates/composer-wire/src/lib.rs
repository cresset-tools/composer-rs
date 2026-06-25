//! The Composer v2 repository **wire format** — the metadata documents a
//! Composer 2 client and a Composer repository exchange.
//!
//! - [`PackageDocument`] / [`RootManifest`] — the `p2/*.json` and root
//!   `packages.json` document types (the [`document`] module).
//! - [`expand_versions`] / [`minify_versions`] — the `composer/2.0`
//!   minified-metadata algorithm and its exact inverse (the [`minify`]
//!   module).
//!
//! A client parses and *expands* upstream metadata; a server builds and
//! *minifies* metadata to serve. Sharing one implementation means the
//! producer and consumer of the wire format can't drift:
//! `expand_versions(minify_versions(v)) == v`.
//!
//! Part of the `composer-rs` workspace; extracted from bougie's
//! `bougie-composer`.

pub mod document;
pub mod minify;

pub use document::{PackageDocument, RootManifest};
pub use minify::{MINIFIED_MARKER, UNSET, expand_versions, minify_versions};
