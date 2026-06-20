//! Composer-flavored semver primitives: [`Version`], [`Constraint`],
//! [`Stability`], [`Bound`].
//!
//! A faithful Rust port of `composer/semver` — the version
//! normalization, comparison, and constraint algebra that Composer
//! itself uses. It mirrors `VersionParser::normalize`,
//! `Comparator::compare`, and `VersionParser::parseConstraints`
//! against a pinned upstream commit (see `version.rs`), with the
//! upstream data-provider cases committed as Layer 1 conformance
//! fixtures (`tests/data/conformance.json`).
//!
//! It is the shared substrate for the `cresset-tools` Composer
//! stack — the `bougie` package manager (PHP/ext picker + dep
//! resolver) and the `sconce` repository server/mirror (version
//! sorting + constraint-based mirror subscriptions) — so the
//! constraint algebra is defined once here rather than re-derived,
//! drifting, in each.

// This crate is a close port of `composer/semver`. A few nested conditionals
// and match arms are deliberately kept in the upstream shape — even where
// clippy would collapse or merge them — so the Rust stays auditable against
// the PHP source (e.g. the one-arm-per-`Suffix` sort table in `version.rs`,
// and `~X` vs `~X.Y` written as separate cases per the comment in
// `constraint.rs`). Allowing the two lints crate-wide keeps that shape without
// scattering per-site `#[allow]`s.
#![allow(clippy::collapsible_if, clippy::match_same_arms)]

pub mod bound;
pub mod constraint;
pub mod stability;
pub mod version;

pub use bound::Bound;
pub use constraint::Constraint;
pub use stability::Stability;
pub use version::Version;
