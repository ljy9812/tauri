## 1. Replace Path Dependencies with Git Dependencies — tauri repo

- [x] 1.1 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `wry = { path = "../wry" }` with `wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }`
- [x] 1.2 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `tao = { path = "../tao" }` with `tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }`
- [x] 1.3 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `muda = { path = "../muda" }` with `muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }`
- [x] 1.4 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `tray-icon = { path = "../tray-icon" }` with `tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }`
- [x] 1.5 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `openharmony-ability = { path = "../openharmony-ability/crates/ability" }` with `openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }`
- [x] 1.6 Convert `tauri/Cargo.toml` [patch.crates-io]: replace `openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }` with `openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }`
- [x] 1.7 Convert `tauri/crates/tauri/Cargo.toml`: replace 4 cross-repo path deps (muda x2, tray-icon x2) with git deps (preserving features, default-features, optional attributes)
- [x] 1.8 Convert `tauri/crates/tauri/Cargo.toml`: replace 2 openharmony-ability path deps (ability + derive) with git deps (preserving features)
- [x] 1.9 Convert `tauri/crates/tauri-runtime/Cargo.toml`: replace `openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }` with git dep
- [x] 1.10 Convert `tauri/crates/tauri-runtime-wry/Cargo.toml`: replace `wry = { path = "../../../wry" ... }` with git dep (preserving features, default-features)
- [x] 1.11 Convert `tauri/crates/tauri-runtime-wry/Cargo.toml`: replace `tao = { path = "../../../tao" ... }` with git dep (preserving features, default-features)
- [x] 1.12 Convert `tauri/crates/tauri-runtime-wry/Cargo.toml`: replace `openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }` with git dep
- [x] 1.13 Convert `tauri/examples/api/src-tauri/Cargo.toml`: replace all 12 `tauri-plugin-*` path deps (pointing to plugins-workspace) with git deps

## 2. Replace Path Dependencies with Git Dependencies — tao repo

- [x] 2.1 Convert `tao/Cargo.toml`: replace `openharmony-ability = { path = "../openharmony-ability/crates/ability" }` with git dep
- [x] 2.2 Convert `tao/Cargo.toml`: replace `openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }` with git dep

## 3. Replace Path Dependencies with Git Dependencies — wry repo

- [x] 3.1 Convert `wry/Cargo.toml`: replace `openharmony-ability = { path = "../openharmony-ability/crates/ability" }` with git dep
- [x] 3.2 Convert `wry/Cargo.toml`: replace `openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }` with git dep

## 4. Replace Path Dependencies with Git Dependencies — muda repo

- [x] 4.1 Convert `muda/Cargo.toml`: replace `openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }` with git dep (preserving features)

## 5. Replace Path Dependencies with Git Dependencies — tray-icon repo

- [x] 5.1 Convert `tray-icon/Cargo.toml`: replace `muda = { path = "../muda" }` direct dep with git dep
- [x] 5.2 Convert `tray-icon/Cargo.toml`: replace `openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu", "statusbar"] }` direct dep with git dep (preserving features)
- [x] 5.3 Convert `tray-icon/Cargo.toml` [patch.crates-io]: replace `muda = { path = "../muda" }` with git dep
- [x] 5.4 Verify tray-icon `[patch.crates-io]` only has `muda` (no openharmony-ability patch entry exists — confirm no action needed for patch section beyond 5.3)

## 6. Replace Path Dependencies with Git Dependencies — plugins-workspace repo

- [x] 6.1 Convert `plugins-workspace/Cargo.toml` [patch.crates-io]: replace `openharmony-ability` and `openharmony-ability-derive` path entries with git deps (keep the existing `tauri-plugin` git patch, convert the two openharmony-ability patches from path to git)
- [x] 6.2 Convert `plugins-workspace/plugins/autostart/Cargo.toml`: replace openharmony-ability path dep with git dep
- [x] 6.3 Convert `plugins-workspace/plugins/clipboard-manager/Cargo.toml`: replace openharmony-ability path dep with git dep (preserving features)
- [x] 6.4 Convert `plugins-workspace/plugins/single-instance/Cargo.toml`: replace openharmony-ability path dep with git dep

## 7. Verification

- [x] 7.1 Run `cargo check --workspace` in tauri repo root — verify all git deps resolve correctly
- [x] 7.2 Run `cargo check` in tao repo — verify dependency resolution
- [x] 7.3 Run `cargo check` in wry repo — verify dependency resolution
- [x] 7.4 Run `cargo check` in muda repo — verify dependency resolution
- [x] 7.5 Run `cargo check` in tray-icon repo — verify dependency resolution
- [x] 7.6 Run `cargo check --workspace` in plugins-workspace — verify dependency resolution

## 8. Create dep-switch-to-git Skill

- [x] 8.1 Create skill directory `tauri/.claude/skills/dep-switch-to-git/`
- [x] 8.2 Write `SKILL.md` with complete static mapping table: every file, exact "from" (path) text, exact "to" (git) text, covering all 39 entries across 7 repos
- [x] 8.3 Write `references/mapping-table.md` with the full dependency mapping (crate name → git URL → local path) for reference
- [x] 8.4 Include idempotency check in skill instructions (scan first, skip if no path deps found)
- [x] 8.5 Include cargo check verification step in skill instructions

## 9. Create dep-switch-to-path Skill

- [x] 9.1 Create skill directory `tauri/.claude/skills/dep-switch-to-path/`
- [x] 9.2 Write `SKILL.md` with complete static mapping table: every file, exact "from" (git) text, exact "to" (path) text — the inverse of the dep-switch-to-git mapping
- [x] 9.3 Write `references/mapping-table.md` with the full reverse dependency mapping for reference
- [x] 9.4 Include idempotency check in skill instructions (scan first, skip if no git deps found)
- [x] 9.5 Include `references/downstream-patch-guide.md` with the `[patch.crates-io]` snippet that downstream users need in their own projects
- [x] 9.6 Include cargo check verification step in skill instructions

## 10. Downstream User Documentation

- [x] 10.1 Create `references/downstream-patch-guide.md` in both skills with the complete `[patch.crates-io]` block that downstream users need to add to their own `Cargo.toml` when using Eulogizethesun forks

## 11. Migrate OHPM Central Package (@ohos-rs/ability → @ylong-rs/ohrs-ability)

- [x] 11.1 Update `crates/tauri-cli/templates/mobile/open-harmony/entry/oh-package.json5`: replace `"@ohos-rs/ability": "file:..."` with `"@ylong-rs/ohrs-ability": "0.4.0-beta.8"`
- [x] 11.2 Update `crates/tauri-cli/templates/mobile/open-harmony/entry/src/main/ets/entryability/EntryAbility.ets.hbs`: replace `import { NativeAbility } from '@ohos-rs/ability'` with `'@ylong-rs/ohrs-ability'`
- [x] 11.3 Reinstall tauri-cli (`cargo install --path crates/tauri-cli --locked`) so template changes are compiled into the binary
- [x] 11.4 Delete `examples/api/src-tauri/gen/ohos/` and regenerate via `tauri ohos init` to pick up new template
- [x] 11.5 Update generated `examples/api/src-tauri/gen/ohos/entry/oh-package.json5` to use `@ylong-rs/ohrs-ability: 0.4.0-beta.8`
- [x] 11.6 Update generated `examples/api/src-tauri/gen/ohos/entry/src/main/ets/entryability/EntryAbility.ets` import to `@ylong-rs/ohrs-ability`
- [x] 11.7 Run `ohpm install --all` in gen/ohos to resolve OHPM dependencies
- [x] 11.8 Verify OHOS desktop build succeeds (hvigorw assembleHap passes CompileArkTS)
