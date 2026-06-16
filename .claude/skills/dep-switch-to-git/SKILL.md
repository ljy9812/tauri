---
name: dep-switch-to-git
description: 将 Tauri OHOS 项目的本地 path 依赖替换为 Eulogizethesun/ohdev-git git 依赖。使用场景：(1) 本地开发完成后准备提交代码，(2) 确保 Cargo.toml 中无跨仓 path 引用，(3) CI 或发布前清理。
---

# dep-switch-to-git

将 10 个仓库中所有跨仓 `path` 依赖替换为 `git` 依赖（`Eulogizethesun/<repo>`, branch `ohdev-git`）。

> **不会修改**：仓内（intra-repo）path 依赖、`schemars_derive` 等已有 git 依赖。
>
> ⚠️ **`cargo-mobile2` 现在需要迁移**：早期 `cargo-mobile2` 指向 `tauri-apps/cargo-mobile2#feat/ohos`（已被 git 化，故旧版 skill 标注"不修改"）。但 `ohdev` 已将 `cargo-mobile2` 收回本地工作区（`path = "../cargo-mobile2"`）以加入 `app::build (assembleApp)` 等改动，而 `tauri-apps/cargo-mobile2#feat/ohos` 没有这些改动。因此 `cargo-mobile2` 必须迁移到 `Eulogizethesun/cargo-mobile2#ohdev-git`（与其他依赖一致），不能再用 `tauri-apps` 目标。

## 执行流程

### Step 1: 扫描当前状态

检查以下仓库中是否还有跨仓 path 依赖（路径均相对于 `D:\workspace\tauri\`）：

| 仓库 | 目录 |
|------|------|
| tauri | `tauri/` |
| tao | `tao/` |
| wry | `wry/` |
| muda | `muda/` |
| tray-icon | `tray-icon/` |
| plugins-workspace | `plugins-workspace/` |
| openharmony-ability | `openharmony-ability/`（叶子节点，无需改动） |
| cargo-mobile2 | `cargo-mobile2/`（叶子节点，无跨仓依赖，无需改动；但被 tauri 引用） |
| sentry-tauri | `sentry-tauri/`（叶子节点，无跨仓依赖；被 tauri 引用） |
| window-vibrancy | `window-vibrancy/`（迁移自身 `openharmony-ability` 依赖，见 2.13） |

对每个仓库执行：
```bash
cd <repo_dir>
grep -rn 'path\s*=\s*"\.\.' --include="Cargo.toml" . | grep -v target/
```

如果扫描结果中**没有**跨仓引用（即所有 `path = ".."` 都指向仓内），输出 "✅ 所有跨仓依赖已为 git 模式" 并退出。

### Step 2: 逐文件替换

使用 Edit 工具，按以下映射表逐条替换。**必须精确匹配 old_string**，仅修改 `path` 部分，保留所有 `features`、`default-features`、`optional`、`version` 等属性。

> ⚠️ **不要修改** 被注释的行（以 `#` 开头）。
> ⚠️ **不要修改** `schemars_derive` 相关行（已是 git 依赖）。
> ⚠️ **`cargo-mobile2` 必须迁移**（见 2.1、2.1b），目标为 `Eulogizethesun/cargo-mobile2#ohdev-git`，不要保留 path、也不要用旧的 `tauri-apps` 目标。
> ⚠️ **替换顺序很重要**：`openharmony-ability-derive` 必须在 `openharmony-ability` 之前替换。
> ⚠️ **插件必须逐个替换**：不能用通配模式。

---

#### 2.1 tauri/Cargo.toml — [patch.crates-io]

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }
to:   openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

from: openharmony-ability = { path = "../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

from: wry = { path = "../wry" }
to:   wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }

from: tao = { path = "../tao" }
to:   tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }

from: muda = { path = "../muda" }
to:   muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }

from: tray-icon = { path = "../tray-icon" }
to:   tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }

# cargo-mobile2: 目标必须是 Eulogizethesun (不是 tauri-apps), 因本地工作区有 app::build 改动
from: cargo-mobile2 = { path = "../cargo-mobile2", default-features = false }
to:   cargo-mobile2 = { git = "https://github.com/Eulogizethesun/cargo-mobile2", branch = "ohdev-git", default-features = false }

# window-vibrancy: ohdev 新增的跨仓依赖
from: window-vibrancy = { path = "../window-vibrancy" }
to:   window-vibrancy = { git = "https://github.com/Eulogizethesun/window-vibrancy", branch = "ohdev-git" }
```

#### 2.1b tauri/crates/tauri-cli/Cargo.toml — cargo-mobile2

```
from: cargo-mobile2 = { path = "../../../cargo-mobile2", default-features = false }
to:   cargo-mobile2 = { git = "https://github.com/Eulogizethesun/cargo-mobile2", branch = "ohdev-git", default-features = false }
```

#### 2.2 tauri/crates/tauri/Cargo.toml — desktop deps (cfg not ohos)

```
from: muda = { path = "../../../muda", default-features = false, features = [
        "serde",
        "gtk",
      ] }
to:   muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git", default-features = false, features = [
        "serde",
        "gtk",
      ] }

from: tray-icon = { path = "../../../tray-icon", default-features = false, features = [
        "serde",
      ], optional = true }
to:   tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ], optional = true }
```

#### 2.3 tauri/crates/tauri/Cargo.toml — ohos deps

```
from: muda = { path = "../../../muda", default-features = false, features = [
        "serde",
      ] }
to:   muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ] }

from: tray-icon = { path = "../../../tray-icon", default-features = false, features = [
        "serde",
      ], optional = true }
to:   tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ], optional = true }

# derive 必须在 ability 之前！
from: openharmony-ability-derive = { path = "../../../openharmony-ability/crates/derive" }
to:   openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["webview", "menu"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["webview", "menu"] }
```

#### 2.4 tauri/crates/tauri-runtime/Cargo.toml

```
from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

#### 2.5 tauri/crates/tauri-runtime-wry/Cargo.toml

```
from: wry = { path = "../../../wry", default-features = false, features = [
        "protocol",
        "os-webview",
        "linux-body",
      ] }
to:   wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git", default-features = false, features = [
        "protocol",
        "os-webview",
        "linux-body",
      ] }

from: tao = { path = "../../../tao",  default-features = false, features = ["rwh_06"] }
to:   tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git",  default-features = false, features = ["rwh_06"] }

from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

#### 2.6 tauri/examples/api/src-tauri/Cargo.toml — plugins (逐个替换)

> ⚠️ 每个插件必须单独替换，不能用批量模式。

```
from: tauri-plugin-http = { path = "../../../../plugins-workspace/plugins/http" }
to:   tauri-plugin-http = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-os = { path = "../../../../plugins-workspace/plugins/os" }
to:   tauri-plugin-os = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-fs = { path = "../../../../plugins-workspace/plugins/fs" }
to:   tauri-plugin-fs = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-shell = { path = "../../../../plugins-workspace/plugins/shell" }
to:   tauri-plugin-shell = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-clipboard-manager = { path = "../../../../plugins-workspace/plugins/clipboard-manager" }
to:   tauri-plugin-clipboard-manager = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-process = { path = "../../../../plugins-workspace/plugins/process" }
to:   tauri-plugin-process = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-updater = { path = "../../../../plugins-workspace/plugins/updater" }
to:   tauri-plugin-updater = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-autostart = { path = "../../../../plugins-workspace/plugins/autostart" }
to:   tauri-plugin-autostart = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-log = { path = "../../../../plugins-workspace/plugins/log" }
to:   tauri-plugin-log = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-notification = { path = "../../../../plugins-workspace/plugins/notification" }
to:   tauri-plugin-notification = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

from: tauri-plugin-dialog = { path = "../../../../plugins-workspace/plugins/dialog" }
to:   tauri-plugin-dialog = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
（注：dialog 出现两次，使用 replace_all: true）

from: tauri-plugin-single-instance = { path = "../../../../plugins-workspace/plugins/single-instance" }
to:   tauri-plugin-single-instance = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

# ohdev 新增插件
from: tauri-plugin-global-shortcut = { path = "../../../../plugins-workspace/plugins/global-shortcut" }
to:   tauri-plugin-global-shortcut = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }

# sentry-tauri 是独立仓 (不是 plugins-workspace 子目录), 指向 Eulogizethesun/sentry-tauri
from: tauri-plugin-sentry = { path = "../../../../sentry-tauri" }
to:   tauri-plugin-sentry = { git = "https://github.com/Eulogizethesun/sentry-tauri", branch = "ohdev-git" }
```

#### 2.7 tao/Cargo.toml

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }
to:   openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

from: openharmony-ability = { path = "../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

#### 2.8 wry/Cargo.toml

同 2.7 的 tao 映射（相同的 from/to 模式）。

#### 2.9 muda/Cargo.toml

```
from: openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["menu"] }
```

#### 2.10 tray-icon/Cargo.toml

```
from: muda = { path = "../muda" }          (所有出现位置，含 [patch.crates-io] 和直接依赖)
to:   muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }

from: openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu", "statusbar"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["menu", "statusbar"] }
```

#### 2.11 plugins-workspace/Cargo.toml — [patch.crates-io]

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }
to:   openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

from: openharmony-ability = { path = "../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
```

> ⚠️ 不要修改 `tauri-plugin` 行（已是 git 依赖）。

#### 2.12 plugins-workspace/plugins/*/Cargo.toml

```
autostart/Cargo.toml:
from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

clipboard-manager/Cargo.toml:
from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["clipboard"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["clipboard"] }

single-instance/Cargo.toml:
from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }

global-shortcut/Cargo.toml:   # ohdev 新增插件
from: openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["global_shortcut"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["global_shortcut"] }
```

#### 2.13 window-vibrancy/Cargo.toml — openharmony-ability

window-vibrancy 是 ohdev 新加入工作区的仓，自身有一处跨仓 `openharmony-ability` 依赖需要迁移（保留 `default-features = false, features = ["window"]`）。

```
from: openharmony-ability = { path = "../openharmony-ability/crates/ability", default-features = false, features = ["window"] }
to:   openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", default-features = false, features = ["window"] }
```

> 仓内自引用（如 `examples/tauri/src-tauri/Cargo.toml` 的 `window-vibrancy = { path = "../../../" }`）保留不动。

### Step 3: 验证

替换完成后执行：
```bash
cd D:/workspace/tauri/tauri
cargo check --workspace 2>&1 | head -50
```

如果网络可用，Cargo 会 clone 所有 git 依赖并验证解析。如果网络不可用，记录警告但不阻塞。

### Step 4: 报告

输出替换总结：
```
✅ 已替换 N 处跨仓 path 依赖
- tauri: X 处
- tao: X 处
- ...
```

## 参考文档

- [完整映射表](references/mapping-table.md) — crate 名称、git URL、本地路径对照
