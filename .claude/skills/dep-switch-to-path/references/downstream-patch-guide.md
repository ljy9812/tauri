# 下游用户 [patch.crates-io] 指南

## 为什么需要 patch？

当你使用 Eulogizethesun fork 的 tauri 时，Cargo 的 `[patch]` 段**不会从 git 依赖传递到你的项目**。你的项目中任何从 crates.io 解析的传递依赖（如 `wry = "0.55"`）都会拉取**上游原版**，缺少 OHOS 适配。

## 需要的 [patch.crates-io] 配置

在你的项目 `Cargo.toml`（workspace 根）中添加：

```toml
[patch.crates-io]
wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }
muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }
tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }
openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

## 工作原理

```
你的项目 Cargo.toml:
  tauri = { git = "...Eulogizethesun/tauri", branch = "ohdev-git" }

解析链:
  tauri (git fork)
    → tauri-runtime-wry → wry (direct git dep) → ✅ Eulogizethesun/wry
    → some-plugin (crates.io) → wry "0.55" (crates.io)
                                  ↓
                        [patch.crates-io]  ← 你的 patch
                                  ↓
                        Eulogizethesun/wry ✅
```

## 关键事实

1. **所有 6 个 crate 都发布在 crates.io 上** — 这意味着 crates.io 上存在这些 crate 的上游版本，传递依赖可能拉到它们
2. **`[patch]` 只在 workspace root 生效** — 放在子 crate 的 Cargo.toml 中无效
3. **如果你只构建非 OHOS 目标**，可能不需要 openharmony-ability 的 patch（它是 cfg(target_env = "ohos") 下的依赖）

## cargo-mobile2

`cargo-mobile2` 保持上游原版（`tauri-apps/cargo-mobile2` branch `feat/ohos`），无需 patch。
