---
name: ohos-migration
description: 将现有 Tauri 应用从官方 crates.io 依赖迁移到 OHOS fork（Eulogizethesun/ohdev-git），实现 OpenHarmony 平台编译。覆盖 Rust 依赖 patch、OHPM 包配置、模板修复、构建环境搭建全流程。
---

# OHOS 迁移指南

将一个使用 Tauri 官方 crates.io 依赖的应用，迁移到 OHOS fork 以支持 OpenHarmony 平台编译。

> **前提**：你已有一个可编译运行的 Tauri 应用（桌面端），现在需要让它也能在 OHOS 上跑。

## 概览

```
迁移全景图
══════════════════════════════════════════════════════════════

  你的 Tauri App (crates.io)        OHOS Fork (Eulogizethesun/ohdev-git)
  ─────────────────────────         ────────────────────────────────
  tauri = "2.x"            ──→      tauri = { git = "...", branch = "ohdev-git" }
  tauri-plugin-* = "2.x"   ──→      tauri-plugin-* = { git = "...", branch = "ohdev-git" }
  (crates.io: wry/tao/...) ──→      [patch.crates-io] → Eulogizethesun forks
  @ohos-rs/ability (local)  ──→     @ylong-rs/ohrs-ability (OHPM 包)

  需要改的：
  ① Cargo.toml         — Rust 依赖 + patch
  ② tauri-cli          — 安装 fork 版本
  ③ tauri ohos init    — 生成 OHOS 项目
  ④ oh-package.json5   — OHPM 包名
  ⑤ EntryAbility.ets   — import 语句
  ⑥ 构建环境           — Rust target + DevEco SDK
  ⑦ 签名               — DevEco Studio
```

---

## Step 1: 修改 Cargo.toml — Rust 依赖

### 1.1 主依赖：tauri

在你的 `src-tauri/Cargo.toml` 中，将 `tauri` 从 crates.io 版本改为 git fork：

```toml
# 改前
tauri = { version = "2", features = [...] }

# 改后
tauri = { git = "https://github.com/Eulogizethesun/tauri", branch = "ohdev-git", features = [...] }
```

### 1.2 插件依赖：tauri-plugin-*

如果你使用了 tauri 官方插件，需要逐个改为 fork 版本：

```toml
# 改前
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-http = "2"
# ... 其他插件同理

# 改后 — 全部指向同一个 plugins-workspace 仓库
tauri-plugin-dialog = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
tauri-plugin-fs = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
tauri-plugin-http = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
# ... 其他插件同理
```

> ⚠️ **注意**：所有 tauri-plugin-* 都来自同一个 `plugins-workspace` 仓库，但每个插件需要**单独一行**声明，不能合并。

### 1.3 添加 [patch.crates-io] — 传递依赖重定向

这是**最关键的一步**。Cargo 的 `[patch]` 不会从 git 依赖传递到你的项目。即使你的 tauri 指向了 fork，fork 内部引用的 `wry`、`tao` 等传递依赖仍然会从 crates.io 拉取上游版本（没有 OHOS 支持）。

在你的 **workspace 根** `Cargo.toml` 中添加：

```toml
[patch.crates-io]
wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }
muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }
tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }
openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

**为什么需要 patch 这 6 个 crate？**

```
解析链：
  tauri (git fork)
    → tauri-runtime-wry → wry "0.55" (crates.io 上游版本，无 OHOS)
                                  ↓
                        [patch.crates-io]  ← 你的 patch
                                  ↓
                        Eulogizethesun/wry (有 OHOS 支持) ✅
```

所有 6 个 crate 都发布在 crates.io 上，传递依赖可能拉到它们的上游版本，所以必须 patch。

> ⚠️ `[patch]` **只在 workspace root 的 Cargo.toml 中生效**，放在子 crate 中无效。

### 1.4 tauri-build 和 tauri-plugin（如果使用）

如果你的 `build-dependencies` 中有 `tauri-build`，也需要改为 fork：

```toml
# 改前
[build-dependencies]
tauri-build = { version = "2", features = [] }

# 改后
[build-dependencies]
tauri-build = { git = "https://github.com/Eulogizethesun/tauri", branch = "ohdev-git" }
```

---

## Step 2: 安装 Fork 版 tauri-cli

tauri-cli 包含 `tauri ohos init` 等 OHOS 专用命令，官方版本没有这些命令：

```bash
cargo install --git https://github.com/Eulogizethesun/tauri \
  --branch ohdev-git \
  --locked \
  tauri-cli
```

验证安装：
```bash
cargo tauri ohos --help
```

---

## Step 3: 初始化 OHOS 项目

### 3.1 安装 Rust 交叉编译 target

```bash
rustup target add aarch64-unknown-linux-ohos
```

### 3.2 运行 tauri ohos init

```bash
cd src-tauri
cargo tauri ohos init --skip-targets-install --ci
```

这会在 `src-tauri/gen/ohos/` 下生成完整的 OHOS 项目结构：

```
gen/ohos/
├── AppScope/          # 应用级配置
├── entry/             # 主模块
│   ├── src/main/ets/  # ArkTS 代码
│   ├── oh-package.json5
│   └── hvigorfile.ts
├── dialog/            # dialog 插件模块（如果使用了 tauri-plugin-dialog）
├── tauri/             # @tauri/app 模块
└── build-profile.json5
```

---

## Step 4: 修复生成的文件

### 4.1 oh-package.json5 — OHPM 包名

`tauri ohos init` 生成的 `entry/oh-package.json5` 中，ability 包名可能还是旧名。

```json5
// gen/ohos/entry/oh-package.json5

// 改前（如果有 @ohos-rs/ability）
"@ohos-rs/ability": "file:..."

// 改后 — 使用 OHPM 上发布的包
"@ylong-rs/ohrs-ability": "0.4.0-beta.8"
```

> `@ylong-rs/ohrs-ability` 是 `openharmony-ability` 的 OHPM 发布包名，提供 ArkTS 侧的 NativeAbility 基类和 ArkHelper。

### 4.2 EntryAbility.ets — import 语句

```typescript
// gen/ohos/entry/src/main/ets/entryability/EntryAbility.ets
import { NativeAbility } from '@ylong-rs/ohrs-ability'
```

> 如果使用的 tauri-cli 版本较旧（模板未更新），生成文件中可能是 `@ohos-rs/ability`，需要手动改为 `@ylong-rs/ohrs-ability`。

### 4.3 ohpm install

```bash
cd gen/ohos
ohpm install --all
```

这会安装 `@ylong-rs/ohrs-ability`、`@tauri/app`、`@tauri/plugin-dialog` 等 OHPM 包到 `oh_modules/`。

---

## Step 5: 配置构建环境

### 5.1 DevEco Studio

需要安装 DevEco Studio（含 OpenHarmony SDK），提供：
- OHOS NDK（clang 交叉编译器）
- hvigorw（ArkTS 构建工具）
- ohpm（包管理器）
- JBR（Java 运行时）

### 5.2 环境变量

需要设置以下环境变量（以 Git Bash 为例）：

```bash
# DevEco Studio 路径
export DEVECO_HOME="/d/app/DevEco-Studio"    # 根据实际路径修改
export OHOS_HOME="$DEVECO_HOME/sdk/default/openharmony"
export JAVA_HOME="$DEVECO_HOME/jbr"

# PATH
export PATH="$DEVECO_HOME/jbr/bin:$PATH:$DEVECO_HOME/tools/hvigor/bin:$DEVECO_HOME/tools/ohpm/bin:$OHOS_HOME/toolchains"

# Rust 交叉编译 — CC 和 linker
OHOS_CLANG="$OHOS_HOME/native/llvm/bin/clang.exe"   # Windows 路径需转义
export CC_aarch64_unknown_linux_ohos="$OHOS_CLANG"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER="$OHOS_CLANG"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_RUSTFLAGS="-C link-arg=--target=aarch64-linux-ohos -C link-arg=--sysroot=$OHOS_HOME/native/sysroot -C link-arg=-D__MUSL__"

# OHPM / hvigorw 需要
export OHOS_NDK_HOME="$DEVECO_HOME\\sdk\\default\\openharmony"   # Windows 格式
```

### 5.3 Rust 交叉编译

```bash
# 编译 OHOS aarch64
cargo build --target aarch64-unknown-linux-ohos --release
```

编译产物在 `target/aarch64-unknown-linux-ohos/release/` 下。

---

## Step 6: 构建 HAP

### 6.1 拷贝 .so 到 OHOS 项目

```bash
cp target/aarch64-unknown-linux-ohos/release/lib<your_app>.so \
   src-tauri/gen/ohos/entry/src/main/cpp/types/lib<your_app>/
```

### 6.2 签名配置

首次构建需要在 DevEco Studio 中配置签名：
1. 打开项目 `src-tauri/gen/ohos/`
2. File → Project Structure → Signing Configs
3. 配置自动签名或手动签名
4. 保存

> 签名配置保存在 `build-profile.json5` 的 `signingConfigs` 中。如果重新 `tauri ohos init` 会丢失，需要重新配置。

### 6.3 hvigorw 构建

```bash
cd src-tauri/gen/ohos
hvigorw assembleHap
```

成功后 HAP 在 `entry/build/default/outputs/default/` 下。

---

## Step 7: 安装到设备

```bash
# 查看已连接设备
hdc list targets

# 安装 HAP
hdc install entry/build/default/outputs/default/entry-default-signed.hap

# 启动应用
hdc shell aa start -a EntryAbility -b com.yourcompany.yourapp
```

---

## 常见问题

### Q: 编译报 `Cannot find module '@ohos-rs/ability'`

oh-package.json5 或 EntryAbility.ets 中的包名还是旧的。按 Step 4 修改。

### Q: 编译报 `Cannot find module '@tauri/app'`

ohpm install 没有运行，或 oh_modules/ 缺失：
```bash
cd gen/ohos && ohpm install --all
```

### Q: 编译报 `cc not found` 或 linker 错误

Step 5.2 的环境变量没有正确设置。确保 `CC_aarch64_unknown_linux_ohos` 指向 OHOS SDK 的 clang。

### Q: `tauri ohos init` 后签名丢失

每次 init 都会重置 `build-profile.json5`。需要在 DevEco Studio 中重新配置签名。

### Q: 其他平台的编译受影响吗？

不受影响。OHOS 代码全部通过 `cfg(target_env = "ohos")` 隔离。编译 Windows/macOS/Linux 时走原有路径。

### Q: `[patch.crates-io]` 会影响非 OHOS 构建吗？

会。patch 是全局生效的，但 fork 版本在非 OHOS 平台上与上游行为一致（OHOS 改动都在 `cfg(target_env = "ohos")` 下）。

### Q: 如何使用本地 path 依赖进行开发调试？

如果你 clone 了所有 fork 仓库到本地，可以用 `path` 依赖代替 `git` 依赖，方便联调：

```toml
# 例：使用本地 wry
[patch.crates-io]
wry = { path = "../wry" }
tao = { path = "../tao" }
# ... 其他同理
```

---

## 完整 Cargo.toml 示例

```toml
[package]
name = "my-tauri-app"
version = "0.1.0"

[dependencies]
# 核心 — 指向 OHOS fork
tauri = { git = "https://github.com/Eulogizethesun/tauri", branch = "ohdev-git", features = [...] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 插件 — 指向 OHOS fork（按需添加你使用的插件）
tauri-plugin-dialog = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
tauri-plugin-fs = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
tauri-plugin-http = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

[build-dependencies]
tauri-build = { git = "https://github.com/Eulogizethesun/tauri", branch = "ohdev-git" }

[patch.crates-io]
# 传递依赖重定向 — 必须有！
wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }
muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }
tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }
openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

## Fork 仓库清单

| 仓库 | 用途 | URL |
|------|------|-----|
| tauri | 核心框架 + tauri-cli | `Eulogizethesun/tauri` branch `ohdev-git` |
| wry | WebView 封装 | `Eulogizethesun/wry` branch `ohdev-git` |
| tao | 窗口管理 | `Eulogizethesun/tao` branch `ohdev-git` |
| muda | 菜单管理 | `Eulogizethesun/muda` branch `ohdev-git` |
| tray-icon | 系统托盘 | `Eulogizethesun/tray-icon` branch `ohdev-git` |
| openharmony-ability | ArkTS 桥接层 | `Eulogizethesun/openharmony-ability` branch `ohdev-git` |
| plugins-workspace | 官方插件集合 | `Eulogizethesun/plugins-workspace` branch `ohdev-git` |
