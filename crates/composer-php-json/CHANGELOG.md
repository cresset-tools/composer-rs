# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release. Byte-exact PHP `json_encode` for the two flag combinations
  Composer relies on (`content-hash` source bytes + `JsonFile::encode` file
  output). Extracted from `bougie-php-json` to be shared across the cresset-tools
  Composer stack.
