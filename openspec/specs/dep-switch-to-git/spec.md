# dep-switch-to-git Specification

## Purpose
TBD - created by archiving change path-to-git-deps. Update Purpose after archive.
## Requirements
### Requirement: Skill scans all repositories and identifies path dependencies to replace
The skill SHALL scan Cargo.toml files across all 7 repositories (tauri, tao, wry, muda, tray-icon, openharmony-ability, plugins-workspace) and identify all cross-repo path dependency entries that match the known mapping table.

#### Scenario: Skill discovers all path dependencies in scope
- **WHEN** the skill is invoked with no arguments
- **THEN** it SHALL read each Cargo.toml file listed in the mapping table and identify all entries containing `path = "../..."` that reference a sibling repository (i.e., cross-repo, not intra-workspace)

#### Scenario: Skill reports current state before making changes
- **WHEN** the skill starts scanning
- **THEN** it SHALL output a summary showing how many path dependencies were found per repository, and how many will be converted to git dependencies

### Requirement: Skill replaces path dependencies with git dependencies using exact mapping
The skill SHALL replace each identified cross-repo path dependency entry with the corresponding git dependency entry pointing to `Eulogizethesun/<repo>` on branch `ohdev-git`.

#### Scenario: Convert tauri root Cargo.toml [patch.crates-io] entries
- **WHEN** processing `tauri/Cargo.toml` [patch.crates-io] section
- **THEN** `wry = { path = "../wry" }` SHALL be replaced with `wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }`, and the same pattern SHALL apply for tao, muda, tray-icon

#### Scenario: Convert [patch.crates-io] entries for openharmony-ability to git
- **WHEN** processing `[patch.crates-io]` sections that contain openharmony-ability path entries
- **THEN** the skill SHALL convert `openharmony-ability` and `openharmony-ability-derive` patch entries from path to git (NOT delete them), since all 6 cross-repo crates are published on crates.io and patches are needed for transitive dependency resolution

#### Scenario: Preserve cargo-mobile2 patch unchanged
- **WHEN** processing `tauri/Cargo.toml` [patch.crates-io] section
- **THEN** `cargo-mobile2 = { git = "https://github.com/tauri-apps/cargo-mobile2", branch = "feat/ohos", default-features = false }` SHALL remain unchanged (not part of the cross-repo fork migration)

#### Scenario: Convert direct dependency entries preserving attributes
- **WHEN** processing dependency entries with `features`, `default-features`, `optional`, or `version` attributes
- **THEN** the skill SHALL preserve all non-source attributes, only replacing `path = "..."` with `git = "...", branch = "ohdev-git"`

#### Scenario: Convert plugins-workspace plugin dependencies
- **WHEN** processing `plugins-workspace/plugins/autostart/Cargo.toml`, `plugins/clipboard-manager/Cargo.toml`, `plugins/single-instance/Cargo.toml`
- **THEN** openharmony-ability path dependencies SHALL be replaced with `git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git"`

#### Scenario: Convert tauri examples plugin dependencies
- **WHEN** processing `tauri/examples/api/src-tauri/Cargo.toml`
- **THEN** all 12 `tauri-plugin-*` entries with `path = "../../../../plugins-workspace/plugins/*"` SHALL be replaced with `git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git"`

### Requirement: Skill does not modify intra-repo path dependencies
The skill SHALL NOT modify any path dependency that references a crate within the same repository/workspace.

#### Scenario: Intra-workspace deps in tauri are untouched
- **WHEN** processing `tauri/crates/tauri/Cargo.toml`
- **THEN** entries like `tauri-runtime = { path = "../tauri-runtime" }`, `tauri-utils = { path = "../tauri-utils" }`, etc. SHALL remain unchanged

#### Scenario: Intra-workspace deps in plugins-workspace are untouched
- **WHEN** processing `plugins-workspace/plugins/dialog/Cargo.toml`
- **THEN** `tauri-plugin-fs = { path = "../fs", version = "2.5.1" }` SHALL remain unchanged (fs is within plugins-workspace)

### Requirement: Skill runs cargo check to verify the conversion
After all replacements are made, the skill SHALL run `cargo check` in each affected workspace to verify that the dependency graph resolves correctly.

#### Scenario: Successful verification
- **WHEN** all replacements complete without errors
- **THEN** the skill SHALL run `cargo check --workspace` in the tauri root and report success or failure

#### Scenario: Network unavailable
- **WHEN** `cargo check` fails because git repositories cannot be cloned (network error)
- **THEN** the skill SHALL display a clear error message indicating that network access is required for git dependencies and suggest using `dep-switch-to-path` for offline development

### Requirement: Skill is idempotent
Running the skill multiple times in succession SHALL produce the same result without errors.

#### Scenario: Second invocation finds no path deps to convert
- **WHEN** the skill is run after a successful first run
- **THEN** it SHALL report "0 cross-repo path dependencies found, all already using git dependencies" and exit without modifying any files

### Requirement: Skill preserves file formatting and comments
The skill SHALL only modify the specific dependency lines in the mapping table, preserving all surrounding content, comments, blank lines, and formatting.

#### Scenario: Comments near dependency lines are preserved
- **WHEN** a Cargo.toml has comments adjacent to a dependency entry
- **THEN** the skill SHALL NOT modify commented-out lines, only active dependency entries

### Requirement: Skill provides dry-run mode
The skill SHALL support a `--dry-run` option that shows what changes would be made without actually modifying any files.

#### Scenario: Dry run shows planned changes
- **WHEN** invoked with `--dry-run`
- **THEN** the skill SHALL display each file that would be modified, the old line and the new line, without writing any changes to disk

