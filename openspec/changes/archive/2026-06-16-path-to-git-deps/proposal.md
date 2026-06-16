## Why

The Tauri OHOS project currently uses local `path = "../xxx"` dependencies across 8 repositories (tauri, tao, wry, muda, tray-icon, openharmony-ability, plugins-workspace, cargo-mobile2) to enable cross-repo development. This requires every developer to clone all repos into a specific directory layout, making the build fragile and non-standard. We need to replace these cross-repo path dependencies with proper `git` dependencies pointing to the `Eulogizethesun/ohdev-git` branch, while providing developer tooling (skills) to temporarily switch back to path dependencies for local development and testing.

## What Changes

- Replace **39 cross-repo path dependency entries** across **13 Cargo.toml files** in **7 repositories** with `git = "https://github.com/Eulogizethesun/<repo>", branch = "ohdev-git"` dependencies
- Clean up `[patch.crates-io]` sections: convert all cross-repo path patches to git patches (all 6 crates — wry, tao, muda, tray-icon, openharmony-ability, openharmony-ability-derive — are published on crates.io, so all patches are necessary for transitive dependency resolution)
- Keep `cargo-mobile2` git dependencies unchanged (still pointing to `tauri-apps/cargo-mobile2` branch `feat/ohos`)
- Migrate OHPM central package from `@ohos-rs/ability` (local HAR) to `@ylong-rs/ohrs-ability@0.4.0-beta.8` (published OHPM package) in tauri-cli templates and generated project files
- Create **skill: `dep-switch-to-path`** — automated tool to switch git dependencies back to local path dependencies for local development
- Create **skill: `dep-switch-to-git`** — automated tool to switch path dependencies back to git dependencies before committing/pushing
- Create **skill: `ohos-migration`** — downstream user guide for migrating from official Tauri to OHOS fork
- Preserve all intra-repo (workspace-internal) path dependencies unchanged

## Capabilities

### New Capabilities
- `dep-switch-to-path`: Skill that scans all 7 repositories and replaces git dependency entries with local path dependency entries, enabling developers to work with all repos locally. Includes a mapping table of crate→repo→path for accurate reverse conversion.
- `dep-switch-to-git`: Skill that scans all 7 repositories and replaces local path dependency entries with git dependency entries pointing to `Eulogizethesun/ohdev-git`, ensuring clean commits and PRs without local path references.
- `ohos-migration`: Downstream user guide for migrating an existing Tauri app (official crates.io) to OHOS fork. Covers Cargo.toml patching, tauri-cli installation, `tauri ohos init`, OHPM package setup, build environment, and signing.

### Modified Capabilities
<!-- No existing capabilities are modified -->

## Impact

- **Repositories affected**: tauri (5 files), tao (1), wry (1), muda (1), tray-icon (1), plugins-workspace (4), openharmony-ability (0), cargo-mobile2 (0)
- **Build system**: First build after switching to git deps will clone all git dependencies (slower). Subsequent builds use Cargo's git cache.
- **Offline development**: Git deps require network for initial clone. The `dep-switch-to-path` skill provides an escape hatch for offline work.
- **CI/CD**: CI will use git deps natively — no need for multi-repo checkout just for dependency resolution (each repo still needs to be checked out for its own build).
- **Cargo.lock**: Will be regenerated after the switch. All repos need `cargo update` after changes.
- **Developer workflow**: New workflow: develop locally with path deps → run `dep-switch-to-git` skill → commit/push → run `dep-switch-to-path` skill → continue developing.
