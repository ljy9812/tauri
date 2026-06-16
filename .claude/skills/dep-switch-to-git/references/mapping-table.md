# Dependency Mapping Table

This table maps every cross-repo dependency between its **path** form (local development) and **git** form (Eulogizethesun/ohdev-git).

## Git URL Pattern

All git dependencies use: `git = "https://github.com/Eulogizethesun/<repo>", branch = "ohdev-git"`

| Crate | Git Repo | Local Path (relative to repo root) |
|-------|----------|------------------------------------|
| `wry` | `Eulogizethesun/wry` | `../wry` |
| `tao` | `Eulogizethesun/tao` | `../tao` |
| `muda` | `Eulogizethesun/muda` | `../muda` |
| `tray-icon` | `Eulogizethesun/tray-icon` | `../tray-icon` |
| `openharmony-ability` | `Eulogizethesun/openharmony-ability` | `../openharmony-ability/crates/ability` |
| `openharmony-ability-derive` | `Eulogizethesun/openharmony-ability` | `../openharmony-ability/crates/derive` |
| `tauri-plugin-*` (all plugins) | `Eulogizethesun/plugins-workspace` | `../../../../plugins-workspace/plugins/<name>` |

## Entries NOT modified (stay as-is in both modes)

- `cargo-mobile2` in `tauri/Cargo.toml` [patch] — git dep to `tauri-apps/cargo-mobile2` branch `feat/ohos`
- `cargo-mobile2` in `tauri/crates/tauri-cli/Cargo.toml` — direct git dep to `tauri-apps/cargo-mobile2` branch `feat/ohos`
- `schemars_derive` in `tauri/Cargo.toml` [patch] — git dep to `tauri-apps/schemars`
- `tauri-plugin` in `plugins-workspace/Cargo.toml` [patch] — already git dep to `Eulogizethesun/tauri` branch `ohdev-git`
- All intra-repo (workspace-internal) path dependencies

## Downstream User [patch.crates-io]

Users who depend on Eulogizethesun forks via git need this in their own `Cargo.toml`:

```toml
[patch.crates-io]
wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }
muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }
tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }
openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

All 6 crates are published on crates.io, so patches are necessary to redirect transitive crates.io dependencies to our git forks.
