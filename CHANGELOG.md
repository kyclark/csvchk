# Changelog

Release notes for each version. The release workflow copies the section
matching a pushed `vX.Y.Z` tag into the GitHub release, so add the entry
here before tagging.

## [0.2.5] - 2026-11-23

### Fixed

- `cargo install csvchk` failed on Rust 1.80+ because 0.2.4 depended on
  `cargo-release`, which pulled in an old `time` crate that no longer
  compiles. That dependency is gone.
- `--limit` with `--grep` now counts only matching records. Previously it
  counted every record read, so it could show fewer matches than requested.
- Piping output into a command that exits early (e.g., `head`) no longer
  panics with "Broken pipe".

### Changed

- Exit with status 1 if any input file can't be opened or read (the other
  files are still processed).
- An invalid `--separator` or `--grep` pattern is reported once at startup
  instead of once per file.
- Clearer error when `--columns` doesn't match the number of fields.
- `--help` now describes every option.
- License changed from MIT to BSD-3-Clause.

### Added

- Prebuilt binaries for Linux, macOS (Intel and Apple Silicon), and Windows
  are attached to each GitHub release.
