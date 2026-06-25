# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/cresset-tools/composer-rs/compare/composer-semver-v0.1.0...composer-semver-v0.2.0) (2026-06-25)


### Features

* add composer-php-json crate ([b14e8e7](https://github.com/cresset-tools/composer-rs/commit/b14e8e739b06b2f586630b7892933bec79fdf353))
* add composer-wire and composer-php-json crates ([b443118](https://github.com/cresset-tools/composer-rs/commit/b443118549aa14624782baba8cf8b912345eba25))
* add composer-wire crate ([ecbab3a](https://github.com/cresset-tools/composer-rs/commit/ecbab3ae2e1d261c8169effe6d70ef84adccf7a4))

## [Unreleased]

## [0.1.0](https://github.com/cresset-tools/composer-rs/releases/tag/composer-semver-v0.1.0) - 2026-06-20

### Added

- Initial release. Composer-conformant `Version`, `Constraint`, `Stability`,
  and `Bound`, with Layer 1 conformance fixtures ported from `composer/semver`.
  Extracted from `bougie-semver` to be shared across the cresset-tools Composer
  stack.
