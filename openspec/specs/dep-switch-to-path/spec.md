# dep-switch-to-path Specification

## Purpose
TBD - created by archiving change path-to-git-deps. Update Purpose after archive.
## Requirements
### Requirement: Skill scans all repositories and identifies git dependencies to replace
The skill SHALL scan Cargo.toml files across all 7 repositories (tauri, tao, wry, muda, tray-icon, openharmony-ability, plugins-workspace) and identify all git dependency entries that match the known mapping table.

#### Scenario: Skill discovers all git dependencies in scope
- **WHEN** the skill is invoked with no arguments
- **THEN** it SHALL read each Cargo.toml file listed in the mapping table and identify all entries containing `git = "https://github.com/Eulogizethesun/..."` with `branch = "ohdev-git"` that are part of the cross-repo dependency set

#### Scenario: Skill reports current state before making changes
- **WHEN** the skill starts scanning
- **THEN** it SHALL output a summary showing how many git dependencies were found per repository, and how many will be converted to path dependencies

### Requirement: Skill replaces git dependencies with path dependencies using exact mapping
The skill SHALL replace each identified git dependency entry with the corresponding local path dependency entry as defined in the static mapping table.

#### Scenario: Convert tauri root Cargo.toml [patch.crates-io] entries
- **WHEN** processing `tauri/Cargo.toml` [patch.crates-io] section
- **THEN** `wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }` SHALL be replaced with `wry = { path = "../wry" }`, and the same pattern SHALL apply for tao, muda, tray-icon

#### Scenario: Convert direct dependency entries in sub-crates
- **WHEN** processing `tauri/crates/tauri-runtime-wry/Cargo.toml`
- **THEN** `wry = { path = "../../../wry" ... }` style entries (with any features/default-features) SHALL be converted — the skill SHALL preserve all `features`, `default-features`, `optional`, and `version` attributes, only changing the source from `git` to `path`

#### Scenario: Convert [patch.crates-io] entries for openharmony-ability back to path
- **WHEN** converting git deps back to path in repositories that have openharmony-ability patch entries
- **THEN** the skill SHALL convert `openharmony-ability` and `openharmony-ability-derive` patch entries from git back to path (same as all other patch entries), since these patches are always retained in both modes

#### Scenario: Preserve cargo-mobile2 patch unchanged
- **WHEN** processing `tauri/Cargo.toml` [patch.crates-io] section
- **THEN** `cargo-mobile2 = { git = "https://github.com/tauri-apps/cargo-mobile2", branch = "feat/ohos", default-features = false }` SHALL remain unchanged (not affected by path/git switching)

### Requirement: Skill runs cargo check to verify the conversion
After all replacements are made, the skill SHALL run `cargo check` in each affected workspace to verify that the dependency graph resolves correctly.

#### Scenario: Successful verification
- **WHEN** all replacements complete without errors
- **THEN** the skill SHALL run `cargo check --workspace` in the tauri root and report success or failure

#### Scenario: Verification failure
- **WHEN** `cargo check` reports dependency resolution errors
- **THEN** the skill SHALL display the error output and suggest the developer check for mapping table staleness

### Requirement: Skill is idempotent
Running the skill multiple times in succession SHALL produce the same result without errors.

#### Scenario: Second invocation finds no git deps to convert
- **WHEN** the skill is run after a successful first run
- **THEN** it SHALL report "0 git dependencies found, all already using path dependencies" and exit without modifying any files

### Requirement: Skill preserves file formatting and comments
The skill SHALL only modify the specific dependency lines in the mapping table, preserving all surrounding content, comments, blank lines, and formatting.

#### Scenario: Comments near dependency lines are preserved
- **WHEN** a Cargo.toml has comments adjacent to a dependency entry (e.g., `#tao = { git = ... }`)
- **THEN** the skill SHALL NOT modify commented-out lines, only active dependency entries

