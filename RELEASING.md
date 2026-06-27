# Releasing sql-dialect-fmt

sql-dialect-fmt ships as a set of crates that share **one workspace version**, declared
once in the root `Cargo.toml` under `[workspace.package]` and inherited by every
crate via `version.workspace = true`. Bumping that single line versions the whole
workspace coherently.

## Crate publication map

Published to crates.io (in dependency order):

| order | crate | depends on |
| --- | --- | --- |
| 1 | `sql-dialect-fmt-syntax` | — |
| 2 | `sql-dialect-fmt-lexer` | syntax |
| 3 | `sql-dialect-fmt-parser` | syntax, lexer |
| 4 | `sql-dialect-fmt-formatter` | syntax, parser |
| 5 | `sql-dialect-fmt-highlight` | syntax, lexer |
| 6 | `sql-dialect-fmt-hover` | syntax, lexer |
| 7 | `sql-dialect-fmt-encoding` | — |
| 8 | `sql-dialect-fmt` | encoding, formatter |
| 9 | `sql-dialect-fmt-lsp` | formatter, highlight, hover, parser, syntax |
| 10 | `sql-dialect-fmt-wasm` | formatter |

**Not published** (`publish = false`):

- `sql-dialect-fmt-test-fixtures` — embedded golden fixtures used only by tests.
- `sql-dialect-fmt-test-support` — shared assertion helpers used only by tests.
- `sql-dialect-fmt-tree-sitter` — its `build.rs` compiles the bundled tree-sitter C
  parser/scanner from `../../tree-sitter-snowflake`, which lives outside the crate
  directory and is therefore not included in the `cargo package` tarball, so
  `cargo publish` verification cannot rebuild it.

## Release procedure

1. **Bump the workspace version.** Edit `version` under `[workspace.package]` in the
   root `Cargo.toml`. Because every crate uses `version.workspace = true`, and every
   internal path dependency pins `version = "X.Y.Z"` to match, update those pins to
   the new version as well (search for the previous version string across all
   `*/Cargo.toml`).

2. **Update the changelog.** Move the `## [Unreleased]` notes in `CHANGELOG.md` into a
   new `## [X.Y.Z] - YYYY-MM-DD` section and refresh the compare links.

3. **Run the green gate** from the workspace root:

   ```sh
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all --check
   scripts/run-external-corpus.sh --sample
   scripts/conformance-report.py --path crates/sql-dialect-fmt-formatter/tests/corpus_sample \
     --out target/conformance-report.md
   ```

4. **Package release assets**:

   ```sh
   scripts/package-extensions.sh
   ```

   This builds the Snowsight Chrome extension zip and the VS Code VSIX under `target/dist/`.
   The GitHub Release workflow uploads those alongside the CLI tarball and checksum.

5. **Dry-run packaging** of each publishable crate, in dependency order:

   ```sh
   cargo publish --dry-run -p sql-dialect-fmt-syntax
   cargo publish --dry-run -p sql-dialect-fmt-lexer
   cargo publish --dry-run -p sql-dialect-fmt-parser
   cargo publish --dry-run -p sql-dialect-fmt-formatter
   cargo publish --dry-run -p sql-dialect-fmt-highlight
   cargo publish --dry-run -p sql-dialect-fmt-hover
   cargo publish --dry-run -p sql-dialect-fmt-encoding
   cargo publish --dry-run -p sql-dialect-fmt
   cargo publish --dry-run -p sql-dialect-fmt-lsp
   cargo publish --dry-run -p sql-dialect-fmt-wasm
   ```

   (`cargo package -p <crate>` produces the tarball without the dry-run upload check.)

6. **Commit and tag:**

   ```sh
   git commit -am "release: vX.Y.Z"
   git tag vX.Y.Z
   git push && git push --tags
   ```

7. **Publish in dependency order.** Each `cargo publish` must complete and the new
   version must be indexed before publishing a dependent crate:

   ```sh
   cargo publish -p sql-dialect-fmt-syntax
   cargo publish -p sql-dialect-fmt-lexer
   cargo publish -p sql-dialect-fmt-parser
   cargo publish -p sql-dialect-fmt-formatter
   cargo publish -p sql-dialect-fmt-highlight
   cargo publish -p sql-dialect-fmt-hover
   cargo publish -p sql-dialect-fmt-encoding
   cargo publish -p sql-dialect-fmt
   cargo publish -p sql-dialect-fmt-lsp
   cargo publish -p sql-dialect-fmt-wasm
   ```

   The canonical order is **syntax → lexer → parser → formatter → highlight → hover →
   cli / lsp / wasm** (with `encoding` published any time before `cli`, and `wasm`
   any time after `formatter`).

8. **Store publishing** is manual-dispatch and credential-gated:

   - VS Code Marketplace requires `VSCE_PAT`.
   - Chrome Web Store requires `CHROME_EXTENSION_ID`, `CHROME_CLIENT_ID`,
     `CHROME_CLIENT_SECRET`, and `CHROME_REFRESH_TOKEN`.

   Run the `Extension Packages` workflow with `publish=true` only after the initial store listing,
   publisher identity, privacy disclosures, and permissions review are ready in the store consoles.

## Notes

- License is `MIT OR Apache-2.0`; both `LICENSE-MIT` and `LICENSE-APACHE` live at the
  repo root.
- Crate metadata (`description`, `keywords`, `categories`, `readme`, `repository`,
  `homepage`, `license`) is inherited from `[workspace.package]` where possible; the
  per-crate `description` is the only required field set locally.
