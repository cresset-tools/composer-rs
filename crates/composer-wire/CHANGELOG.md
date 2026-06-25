# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release. Composer v2 repository wire documents (`PackageDocument`,
  `RootManifest`) and the `composer/2.0` minify/expand algorithm
  (`expand_versions` / `minify_versions`). Extracted from bougie's
  `bougie-composer` so the cresset-tools Composer stack shares one
  implementation of the wire format.
