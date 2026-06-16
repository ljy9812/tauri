---
name: dep-switch-to-path
description: 将 Tauri OHOS 项目的 git 依赖恢复为本地 path 依赖。使用场景：(1) 需要本地调测多个仓库，(2) 离线开发，(3) 快速迭代时避免 git clone 开销。
---

# dep-switch-to-path

将 7 个仓库中所有跨仓 `git` 依赖恢复为本地 `path` 依赖，方便本地联调开发。

> **不会修改**：仓内（intra-repo）path 依赖、`cargo-mobile2`、`schemars_derive` 等。

## 执行流程

### Step 1: 扫描当前状态

检查以下仓库中是否有 Eulogizethesun git 依赖：

```bash
cd D:/workspace/tauri/<repo>
grep -rn 'Eulogizethesun' --include="Cargo.toml" . | grep -v target/ | grep -v '^\s*#'
```

如果扫描结果中**没有** Eulogizethesun git 引用，输出 "✅ 所有依赖已为 path 模式" 并退出。

### Step 2: 逐文件替换

> ⚠️ **替换顺序很重要**：`openharmony-ability-derive` 必须在 `openharmony-ability` 之前替换，否则 derive 会被错误匹配。
> ⚠️ **插件必须逐个替换**：不能用通配模式，否则所有插件路径会变成同一个。

使用 Edit 工具，按以下映射表逐条替换。将 git 依赖恢复为 path 依赖。

---

#### 2.1 tauri/Cargo.toml — [patch.crates-io]

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../openharmony-ability/crates/ability" }

from: wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git" }
to:   wry = { path = "../wry" }

from: tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git" }
to:   tao = { path = "../tao" }

from: muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }
to:   muda = { path = "../muda" }

from: tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git" }
to:   tray-icon = { path = "../tray-icon" }
```

#### 2.2 tauri/crates/tauri/Cargo.toml — desktop deps

```
from: muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git", default-features = false, features = [
        "serde",
        "gtk",
      ] }
to:   muda = { path = "../../../muda", default-features = false, features = [
        "serde",
        "gtk",
      ] }

from: tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ], optional = true }
to:   tray-icon = { path = "../../../tray-icon", default-features = false, features = [
        "serde",
      ], optional = true }
```

#### 2.3 tauri/crates/tauri/Cargo.toml — ohos deps

```
from: muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ] }
to:   muda = { path = "../../../muda", default-features = false, features = [
        "serde",
      ] }

from: tray-icon = { git = "https://github.com/Eulogizethesun/tray-icon", branch = "ohdev-git", default-features = false, features = [
        "serde",
      ], optional = true }
to:   tray-icon = { path = "../../../tray-icon", default-features = false, features = [
        "serde",
      ], optional = true }

# derive 必须在 ability 之前！
from: openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability-derive = { path = "../../../openharmony-ability/crates/derive" }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["webview", "menu"] }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["webview", "menu"] }
```

#### 2.4 tauri/crates/tauri-runtime/Cargo.toml

```
from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
```

#### 2.5 tauri/crates/tauri-runtime-wry/Cargo.toml

```
from: wry = { git = "https://github.com/Eulogizethesun/wry", branch = "ohdev-git", default-features = false, features = [
        "protocol",
        "os-webview",
        "linux-body",
      ] }
to:   wry = { path = "../../../wry", default-features = false, features = [
        "protocol",
        "os-webview",
        "linux-body",
      ] }

from: tao = { git = "https://github.com/Eulogizethesun/tao", branch = "ohdev-git",  default-features = false, features = ["rwh_06"] }
to:   tao = { path = "../../../tao",  default-features = false, features = ["rwh_06"] }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
```

#### 2.6 tauri/examples/api/src-tauri/Cargo.toml — plugins (逐个替换)

> ⚠️ 每个插件必须单独替换，不能用批量模式。

```
from: tauri-plugin-http = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-http = { path = "../../../../plugins-workspace/plugins/http" }

from: tauri-plugin-os = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-os = { path = "../../../../plugins-workspace/plugins/os" }

from: tauri-plugin-fs = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-fs = { path = "../../../../plugins-workspace/plugins/fs" }

from: tauri-plugin-shell = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-shell = { path = "../../../../plugins-workspace/plugins/shell" }

from: tauri-plugin-clipboard-manager = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-clipboard-manager = { path = "../../../../plugins-workspace/plugins/clipboard-manager" }

from: tauri-plugin-process = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-process = { path = "../../../../plugins-workspace/plugins/process" }

from: tauri-plugin-updater = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-updater = { path = "../../../../plugins-workspace/plugins/updater" }

from: tauri-plugin-autostart = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-autostart = { path = "../../../../plugins-workspace/plugins/autostart" }

from: tauri-plugin-log = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-log = { path = "../../../../plugins-workspace/plugins/log" }

from: tauri-plugin-notification = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-notification = { path = "../../../../plugins-workspace/plugins/notification" }

from: tauri-plugin-dialog = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-dialog = { path = "../../../../plugins-workspace/plugins/dialog" }
（注：dialog 出现两次，使用 replace_all: true）

from: tauri-plugin-single-instance = { git = "https://github.com/Eulogizethesun/plugins-workspace", branch = "ohdev-git" }
to:   tauri-plugin-single-instance = { path = "../../../../plugins-workspace/plugins/single-instance" }
```

#### 2.7 tao/Cargo.toml

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../openharmony-ability/crates/ability" }
```

#### 2.8 wry/Cargo.toml

同 2.7 的映射模式。

#### 2.9 muda/Cargo.toml

```
from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["menu"] }
to:   openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu"] }
```

#### 2.10 tray-icon/Cargo.toml

```
from: muda = { git = "https://github.com/Eulogizethesun/muda", branch = "ohdev-git" }   (所有出现位置)
to:   muda = { path = "../muda" }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["menu", "statusbar"] }
to:   openharmony-ability = { path = "../openharmony-ability/crates/ability", features = ["menu", "statusbar"] }
```

#### 2.11 plugins-workspace/Cargo.toml — [patch.crates-io]

```
# derive 必须在 ability 之前！
from: openharmony-ability-derive = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability-derive = { path = "../openharmony-ability/crates/derive" }

from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../openharmony-ability/crates/ability" }
```

> ⚠️ 不要修改 `tauri-plugin` 行（git 依赖保持不变）。

#### 2.12 plugins-workspace/plugins/*/Cargo.toml

```
autostart:
from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }

clipboard-manager:
from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git", features = ["clipboard"] }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability", features = ["clipboard"] }

single-instance:
from: openharmony-ability = { git = "https://github.com/Eulogizethesun/openharmony-ability", branch = "ohdev-git" }
to:   openharmony-ability = { path = "../../../openharmony-ability/crates/ability" }
```

### Step 3: 验证

```bash
cd D:/workspace/tauri/tauri
cargo check --workspace 2>&1 | head -50
```

### Step 4: 报告

```
✅ 已恢复 N 处本地 path 依赖
- tauri: X 处
- tao: X 处
- ...

⚠️ 提醒：path 模式下 Cargo.lock 会产生差异，提交代码前请运行 dep-switch-to-git 切换回 git 模式。
```

## 下游用户注意

如果你使用 Eulogizethesun fork 开发自己的项目，需要在你的 `Cargo.toml` 中添加 `[patch.crates-io]`。详见 [下游 patch 指南](references/downstream-patch-guide.md)。

## 参考文档

- [完整映射表](references/mapping-table.md) — crate 名称、git URL、本地路径对照
- [下游 patch 指南](references/downstream-patch-guide.md) — 下游用户需要的 [patch.crates-io] 配置
