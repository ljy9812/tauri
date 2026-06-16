## Context

The Tauri OHOS adaptation project spans 8 repositories that are currently linked via local `path` dependencies for convenient cross-repo debugging:

```
D:\workspace\tauri\
├── tauri/                  (main repo, workspace root)
├── tao/                    (window manager)
├── wry/                    (webview)
├── muda/                   (menu)
├── tray-icon/              (system tray)
├── openharmony-ability/    (OHOS system bridge, 2 sub-crates)
├── plugins-workspace/      (tauri plugins, 30+ sub-crates)
└── cargo-mobile2/          (mobile build tool, independent)
```

A scan of all `Cargo.toml` files found **39 cross-repo path dependency entries** across **13 files** in **7 repositories** (openharmony-ability and cargo-mobile2 have none).

Current state of cross-repo dependencies:

```
                    ┌──────────────┐
                    │     tauri    │
                    │  [patch.io]  │
                    │  + 4 crates  │
                    │  + examples  │
                    └──────┬───────┘
           ┌───────────┬───┴───┬────────────┬──────────────┐
           ▼           ▼       ▼            ▼              ▼
        ┌──────┐  ┌──────┐ ┌──────┐  ┌──────────┐  ┌──────────────┐
        │ wry  │  │ tao  │ │ muda │  │tray-icon │  │   plugins-   │
        │      │  │      │ │      │  │          │  │  workspace   │
        └──┬───┘  └──┬───┘ └──┬───┘  └──┬───┬───┘  └──────┬───────┘
           │         │        │         │   │             │
           ▼         ▼        ▼         ▼   ▼             ▼
        ┌──────────────────────────────────────────────────────┐
        │            openharmony-ability                        │
        │  (ability + derive — leaf node, depended on by all)  │
        └──────────────────────────────────────────────────────┘
```

Key constraints:
- `openharmony-ability` contains 2 crates (`ability` + `derive`) in one repo — Cargo resolves git deps by searching the entire repo for matching `package.name`, so no repo split needed
- `[patch.crates-io]` sections are used in tauri, tray-icon, and plugins-workspace to override crates.io versions
- `cargo-mobile2` is already a git dep in tauri's `[patch.crates-io]` and `tauri-cli/Cargo.toml`, pointing to `tauri-apps/cargo-mobile2` branch `feat/ohos` — this stays unchanged
- `plugins-workspace` already has a git patch for `tauri-plugin` pointing to `Eulogizethesun/tauri` branch `ohdev-git`

## Goals / Non-Goals

**Goals:**
- Replace all 39 cross-repo path dependencies with git dependencies (`Eulogizethesun/<repo>`, branch `ohdev-git`)
- Clean up `[patch.crates-io]` sections consistently
- Provide two Claude Code skills for developers to switch between git and path modes
- Ensure all repos compile cleanly after the switch
- Make the skills idempotent — running them multiple times produces the same result

**Non-Goals:**
- Changing intra-repo (workspace-internal) path dependencies (e.g., `tauri` → `tauri-utils` within the tauri workspace)
- Publishing any crate to crates.io
- Modifying the source code or API of any crate
- Changing the directory layout on disk
- CI/CD pipeline changes (each repo's CI already handles its own build)

## Decisions

### Decision 1: Use `git` + `branch` for all cross-repo deps

**Choice**: `git = "https://github.com/Eulogizethesun/<repo>", branch = "ohdev-git"`

**Alternatives considered**:
- `git` + `rev` (commit hash): More reproducible but requires updating hashes on every upstream change. Rejected — too much maintenance during active development.
- `git` + `tag`: Not practical since we don't tag every change on ohdev-git.
- `[patch.crates-io]` only (keep path in direct deps): Doesn't work — path deps take precedence over patches. Would still need to change direct deps.

**Rationale**: Branch-based git deps give a good balance of reproducibility (always latest from ohdev-git) and low maintenance (no hash updates needed).

### Decision 2: Keep ALL `[patch.crates-io]` entries, convert all to git

**Choice**: Convert ALL `[patch.crates-io]` path entries to git — including `openharmony-ability` and `openharmony-ability-derive`. Do NOT delete any patch entry.

**Audit finding**: All cross-repo dependency crates are **published on crates.io**:
```
wry                        → published ✅
tao                        → published ✅
muda                       → published ✅
tray-icon                  → published ✅
openharmony-ability        → published ✅ (since 2025-11-17)
openharmony-ability-derive → published ✅ (since 2025-11-17)
```

**Why patches are critical**: `[patch.crates-io]` redirects transitive crates.io dependencies to our git forks. Without patches, if a published crate (e.g., a tauri plugin on crates.io) declares `wry = "0.55"`, Cargo would resolve to the **upstream crates.io wry** — which lacks OHOS modifications.

```
┌──────────────────────────────────────────────────────────────┐
│  依赖传递链与 patch 的作用                                     │
│                                                              │
│  下游用户项目:                                                │
│    tauri = { git = "...Eulogizethesun/tauri", branch="ohdev-git" }│
│                                                              │
│  tauri workspace 解析:                                        │
│    tauri-runtime-wry → wry (direct dep) → git fork ✅        │
│    some-plugin (crates.io) → wry "0.55" (crates.io)         │
│                              ↓                               │
│                    [patch.crates-io]                          │
│                              ↓                               │
│                    wry git fork ✅                            │
│                                                              │
│  ⚠️ [patch] 不传递: tauri 的 patch 只在 tauri workspace 生效  │
│  ⚠️ 下游用户需要自己的 [patch.crates-io] 配置                  │
└──────────────────────────────────────────────────────────────┘
```

**Rationale**: All 6 crates (wry, tao, muda, tray-icon, openharmony-ability, openharmony-ability-derive) are published on crates.io, meaning transitive crates.io dependencies could pull upstream versions. Patches ensure the entire dependency tree resolves to our Eulogizethesun/ohdev-git forks.

### Decision 2b: Downstream user patch guide

**Choice**: Provide a `[patch.crates-io]` reference snippet in the dep-switch skills' documentation, so downstream users know what patches to add in their own `Cargo.toml`.

**Rationale**: Since `[patch]` does not propagate from git dependencies, every project using our forks needs its own patches. This is standard Cargo behavior but easy to miss.

### Decision 3: Do NOT change cargo-mobile2 dependencies

**Choice**: Keep cargo-mobile2 git dependencies as-is (`tauri-apps/cargo-mobile2`, branch `feat/ohos`) in both `tauri/Cargo.toml` [patch] and `tauri/crates/tauri-cli/Cargo.toml` direct dep.

**Rationale**: cargo-mobile2 is a build tool dependency and does not need to follow the Eulogizethesun/ohdev-git fork pattern. The existing `tauri-apps/cargo-mobile2` source is correct.

### Decision 4: Skills operate via sed/find string replacement on Cargo.toml files

**Choice**: The skills use a static mapping table (crate name → git URL + path) and perform targeted text replacements in Cargo.toml files using `sed` or Claude's Edit tool.

**Alternatives considered**:
- Cargo workspace inheritance: Doesn't support cross-repo deps.
- Custom Cargo alias/plugin: Over-engineered for this use case.
- Separate config file: Adds complexity without benefit since the mapping is fixed.

**Rationale**: Direct text replacement is simple, auditable, and easy to debug. The mapping is small enough to hardcode.

### Decision 5: Skills live in tauri repo's `.claude/skills/` directory

**Choice**: Both skills (`dep-switch-to-path` and `dep-switch-to-git`) are placed in `tauri/.claude/skills/` since tauri is the main workspace repo.

**Rationale**: The tauri repo is the central repo that all developers clone. Skills here are automatically available via Claude Code's skill system.

### Decision 6: Skill design — static mapping table with exact match/replace patterns

**Choice**: Each skill contains a complete mapping table listing every file, the exact "from" text, and the exact "to" text. This makes the skill deterministic and easy to audit.

**Format for each mapping entry**:
```
repo: tauri
file: Cargo.toml
from: wry = { path = "../wry" }
to:   wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
```

**Rationale**: Exact string matching avoids false positives. Every replacement is explicit and auditable. No regex guessing needed.

### Decision 7: Migrate OHPM central package from @ohos-rs/ability to @ylong-rs/ohrs-ability

**Choice**: Replace the old local HAR dependency `@ohos-rs/ability` with the published OHPM package `@ylong-rs/ohrs-ability@0.4.0-beta.8` in both the tauri-cli template and generated project files.

**Affected files**:

| File | Type | Change |
|------|------|--------|
| `crates/tauri-cli/templates/mobile/open-harmony/entry/oh-package.json5` | Template | `"@ohos-rs/ability": "file:..."` → `"@ylong-rs/ohrs-ability": "0.4.0-beta.8"` |
| `crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/ets/entryability/EntryAbility.ets.hbs` | Template | `import { NativeAbility } from '@ohos-rs/ability'` → `'@ylong-rs/ohrs-ability'` |
| `examples/api/src-tauri/gen/ohos/entry/oh-package.json5` | Generated | Same package name change |
| `examples/api/src-tauri/gen/ohos/entry/src/main/ets/entryability/EntryAbility.ets` | Generated | Same import change |

**Why the rename**: `openharmony-ability` 的 OHPM 发布包名为 `@ylong-rs/ohrs-ability`，旧名 `@ohos-rs/ability` 已弃用。

**Rationale**: Using a published OHPM package instead of a local HAR file eliminates the need to manually build and copy HAR files. `ohpm install` resolves the dependency automatically. The template change ensures all future `tauri ohos init` runs generate correct references.

### Decision 8: Reinstall tauri-cli after template changes

**Choice**: Template files (`.hbs`, `.json5`) are compiled into the tauri-cli binary. After modifying templates, `cargo install --path crates/tauri-cli --locked` must be run to rebuild the binary.

**Rationale**: Without reinstalling, `cargo tauri ohos init` uses the old cached binary with stale templates, regenerating files with the old `@ohos-rs/ability` references.

## Risks / Trade-offs

### Risk 1: First build slower with git deps
- **Impact**: `cargo build` will clone all git repos on first run (~30s-2min depending on network)
- **Mitigation**: Cargo caches git repos in `~/.cargo/git/`. Subsequent builds are fast. The `dep-switch-to-path` skill provides instant builds for active development.

### Risk 2: Git deps require network for initial clone
- **Impact**: Cannot build offline for the first time after switching
- **Mitigation**: `dep-switch-to-path` skill switches back to local paths for offline development. Cargo also caches git repos so re-cloning is rare.

### Risk 3: Version mismatch between git and crates.io
- **Impact**: If a published crate (e.g., `wry 0.55.0` on crates.io) has a different version than the git fork, Cargo might report version conflicts
- **Mitigation**: The `[patch.crates-io]` entries ensure all instances (direct and transitive) resolve to the git version. If conflicts arise, the patch forces resolution to git.

### Risk 4: Skill mapping becomes stale after Cargo.toml changes
- **Impact**: If someone adds a new cross-repo dependency, the skills won't know about it
- **Mitigation**: Skills include a scan step that warns about unknown path/git deps not in the mapping table. The mapping is easy to update.

### Risk 5: Cargo.lock changes on every switch
- **Impact**: Switching between path and git modes regenerates Cargo.lock entries
- **Mitigation**: Skills should run `cargo update` after switching. Cargo.lock changes should NOT be committed when using path mode.

### Risk 6: Downstream users missing [patch.crates-io]
- **Impact**: A project depending on our tauri fork via git will NOT inherit our `[patch.crates-io]` (Cargo only honors `[patch]` from the root manifest). If they have any transitive crates.io dependency on wry/tao/muda/tray-icon/openharmony-ability, they'll get upstream versions without OHOS support, causing build failures or runtime issues.
- **Mitigation**: Document the required `[patch.crates-io]` block in the dep-switch skills' `references/` directory and in the project README. The `dep-switch-to-path` skill should also output a reminder about downstream patches when invoked.

### Risk 7: plugins-workspace missing patches for tauri and tauri-utils (existing issue)
- **Impact**: plugins-workspace depends on `tauri = "2.10"`, `tauri-build = "2.5"`, `tauri-utils = "2.8"` from crates.io, but `[patch.crates-io]` only patches `tauri-plugin`. The plugins build with upstream tauri/tauri-utils instead of the Eulogizethesun fork. If the fork modifies tauri core APIs needed by plugins, this could cause build failures.
- **Mitigation**: This is a pre-existing issue, not introduced by this change. Monitor and add patches for `tauri` and `tauri-utils` in plugins-workspace if build issues arise. Note in the downstream user guide that users may also want to patch these.

### Trade-off: Static mapping vs. dynamic scanning
- Static mapping is more fragile (needs updates when deps change) but more predictable and auditable
- Dynamic scanning (parse Cargo.toml at runtime) is more flexible but harder to reason about
- **We chose static mapping** because the dependency graph is relatively stable and the auditability is important
