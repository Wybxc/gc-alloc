# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/Wybxc/gc-alloc/compare/v0.2.0...v0.2.1) - 2026-06-21

### Other

- Require GcToken for boxed::alloc
- Wrap GC-allocated references in opaque types with token-gated access

## [0.2.0](https://github.com/Wybxc/gc-alloc/compare/v0.1.1...v0.2.0) - 2026-06-21

### Other

- Require GcToken for all allocation functions

## [0.1.1](https://github.com/Wybxc/gc-alloc/compare/v0.1.0...v0.1.1) - 2026-06-20

### Added

- add git submodules for vendored dependencies

### Other

- Add release-plz CI workflow for automated releases
- Switch from system libgc to vendored bdwgc via cmake
