# OHOS 多设备 .app 构建 — 代码检视报告

> 检视对象：当前工作区未提交改动（`git diff` + 新增 `entry_mobile/`、`entry_desktop/` 模板目录）
> 对照设计：[`doc/ohos-multi-device-app-build-design.md`](ohos-multi-device-app-build-design.md)（A 方案）
> 检视日期：2026-07-01
> 检视模式：本地未提交改动检视（未执行编译部署、未提交 GitHub）

## 涉及代码

- `tauri-cli/src/mobile/open_harmony/{build,mod,dev,run,plugins,project,init}.rs`
- `templates/mobile/open-harmony/`（删除单 `entry/`，新增 `entry_mobile/` + `entry_desktop/`）
- `Cargo.toml`、`crates/tauri-cli/Cargo.toml`（`cargo-mobile2` 改本地 path 依赖）
- 本地 path 依赖 `cargo-mobile2`（新增 `app.rs`，`target.rs` 的 `--dist` 按形态推目录）

## 总览

| 🔴 Blocker | 🟡 Major | 🔵 Minor | ℹ️ Info |
|---|---|---|---|
| 0 | 2 | 5 | 4 |

整体方向与设计文档一致：双 entry 模板、`forms_for_device_types` 切分、CLI 显式编 .so + `TAURI_OHOS_SKIP_DEVECO_SCRIPT` 守卫、`app::build` 走 `assembleApp`、签名泛化。三条铁律（`openharmony-ability` 桥接 / `cfg(target_env="ohos")` 隔离 / `OHOS_DEVICE_TYPE` 决定形态）未违反，`dev` 正确保留 tauriPlugin 不 skip。无 Blocker，但有 2 处 Major 需确认。

---

## 🟡 Major

### M1. TestTrayAbility 归属倒置 — mobile 声明了 tray 扩展，desktop 反而没声明

- **位置**：`templates/mobile/open-harmony/entry_mobile/src/main/module.json5:47-52` vs `templates/mobile/open-harmony/entry_desktop/src/main/module.json5`
- **现象**：拆分双 entry 时，原 `entry/module.json5` 中的 `TestTrayAbility`（`type: "statusBarView"`）只被带进了 `entry_mobile`，`entry_desktop` 的 `extensionAbilities` 里删除了它。但两侧的 `TestTrayAbility.ets` / `TestTrayPage.ets` 源文件完全相同，都在模板里。
- **问题**：铁律 #3 与设计文档明确 tray/menu bar 属 desktop 形态（`OHOS_DEVICE_TYPE=desktop` 启用 `cfg(desktop)` 含 tray）。现状：
  - mobile HAP 声明了一个 statusBarView tray 扩展（语义错误，mobile 不该有 tray）
  - desktop HAP 没有声明 → desktop 的 `TestTrayAbility.ets` 变成死代码，tray 测试页在 desktop 包里不会被注册
- **建议**：把 `TestTrayAbility` 的 `extensionAbilities` 声明从 `entry_mobile` 移到 `entry_desktop`；或如果该测试页是形态无关的通用测试，则在两侧都声明并说明理由。请确认拆分意图。

### M2. mobile 形态设备类集合偏离设计文档

- **位置**：`crates/tauri-cli/src/mobile/open_harmony/mod.rs:351` `MOBILE = &["phone", "tablet", "car", "wearable", "tv"]`
- **设计文档 §4**：`mobile 类 ← phone, tablet`（仅两值），`desktop 类 ← 2in1`
- **现象**：代码把 `car/wearable/tv` 也归入 mobile 形态。`build --app` 的 `forms_for_device_types`、`device_types_for_form`、`build.rs:189-193` 的 bail 提示（`phone/tablet/car/wearable/tv/2in1`）都按这个更宽的集合工作。
- **问题**：设计与实现不一致（H6 Design-implementation mismatch）。代码行为可能更完整，但 `--app` 切分结果会与设计文档描述不同（如 conf 含 `car` 时，代码产 mobile HAP，设计文档则不会映射）。
- **建议**：二选一——要么收窄代码到 `{phone, tablet}` 与设计对齐；要么更新设计文档 §4 把 `car/wearable/tv` 显式纳入 mobile 类并说明依据。需 team 确认哪种是预期。

---

## 🔵 Minor

### m1. 注释/doc-string 用连字符 `entry-mobile`，实际模块名是下划线 `entry_mobile`

- **位置**：
  - `crates/tauri-cli/src/mobile/open_harmony/plugins.rs:567-569` `write_build_profile_modules` doc：`["entry-mobile"]`、`["entry-mobile", "entry-desktop"]`
  - `crates/tauri-cli/src/mobile/open_harmony/project.rs:53` 注释 `Both entry-mobile and entry-desktop`
  - `crates/tauri-cli/src/mobile/open_harmony/mod.rs:340` `active_entry_module` doc：`entry-mobile / entry-desktop`
  - 本地 `cargo-mobile2/src/open_harmony/target.rs:201` 注释 `entry-{OHOS_DEVICE_TYPE}`
- **问题**：设计文档 §2 明确实测 hvigor 模块名禁连字符（必须下划线），代码生成也确实用下划线，但注释一律写成连字符，会误导后续维护者。
- **建议**：注释统一改为 `entry_mobile` / `entry_desktop`。

### m2. `--app` 与 `--open` 组合时 `--open` 被静默忽略

- **位置**：`crates/tauri-cli/src/mobile/open_harmony/build.rs:185-230` `command`
- **现象**：`--app` 走 `if options.app` 分支，只调 `run_app`，从不读 `options.open`。`--app` 只声明了 `conflicts_with = "device_type"`，没和 `open` 互斥。用户传 `cargo tauri ohos build --app --open` 不会报错也不会打开 DevEco。
- **建议**：给 `app` 加 `conflicts_with = "open"`，或在 `--app` 分支检测到 `open` 时 warn。

### m3. `app::build` 产物 glob 在"残留旧签名包"场景下行为可疑

- **位置**：本地 `cargo-mobile2/src/open_harmony/app.rs` `app_paths` + `crates/tauri-cli/src/mobile/open_harmony/build.rs:262` `sign_if_configured`
- **现象**：`app_paths` 收集 `build/outputs/default/*.app` 全部文件，`reduce(last_modified)` 取最新。两种边角：
  1. 上次构建留下 `*-signed.app`（旧），本次产出 `*-unsigned.app`（新）→ 取到 unsigned，正常签名。✅
  2. 但若 signed 因某些操作 mtime 更新变成最新 → `app_output` 已是 signed；`sign_if_configured` 里 `replace("unsigned","signed")` 无变化 → `signed_path == path`，`sign_hap(path, path)` 同名输入输出，hap-sign-tool 可能拒绝。
  3. 未设签名 env 且目录里有残留 signed 时，`has_signed=true` 不告警，但返回的 `app_output` 可能是 unsigned（取最新那条），用户拿到未签名包却以为有签名。
- **建议**：`app_paths` / `sign_if_configured` 区分 signed/unsigned，优先返回 unsigned 让签名流程幂等；或签名前过滤掉已是 `-signed` 的产物。

### m4. `command` 无条件 `set_var("OHOS_DEVICE_TYPE", &options.device_type)`，`--app` 下也执行

- **位置**：`crates/tauri-cli/src/mobile/open_harmony/build.rs:115`
- **现象**：`--app` 时 `options.device_type` 仍是 clap 默认 `"mobile"`，所以在 `--app` 循环重置前，`inject_plugins`（`build.rs:180`）是在 `OHOS_DEVICE_TYPE=mobile` 下跑的——`update_entry_package` 只写 `entry_mobile`。循环里再对 desktop 显式补写，功能正确，但依赖"循环补写"的隐式顺序，脆弱。
- **建议**：`--app` 分支跳过这行默认 set，或把 `inject_plugins` 移到循环内按形态各跑一次（与 `update_entry_package` 一致）。

### m5. `write_build_profile_modules` 重建 entry 模块时硬编码字段，丢弃用户自定义

- **位置**：`crates/tauri-cli/src/mobile/open_harmony/plugins.rs:589-596`
- **现象**：重建 entry 模块时只写 `name/srcPath/targets` 三字段。模板生成的工程这三字段就够，但如果用户在 entry 的 build-profile 里加了 `buildOption` 等，会被抹掉。
- **建议**：可接受（模板工程通常不自定义 entry build-profile），但建议从原 modules 数组里按 name 匹配保留既有 entry 对象，仅控制"激活/去激活"，而非重建。至少加注释说明会重建。

---

## ℹ️ Info

### i1. 根目录 `devtools-test-cases.patch` 不应入库（H1）

- **位置**：`devtools-test-cases.patch`（untracked，3625 字节）
- **建议**：提交前删除或加入 `.gitignore`，patch 文件不属于源码。

### i2. `doc/manual_tests.md` 未为 `--app` 新增手动用例（H5，边界）

- `build --app` 是新用户可见 CLI 能力，但属构建/打包流程而非运行时 API，H5 的"createPdf/tray/menu"类运行时功能标准对此边界模糊。建议至少补一条 `.app` 打包+签名+安装验证的手动用例。

### i3. 设计文档放在 `doc/` 而非 `openspec/changes/`（H6）

- `ohos-multi-device-app-build-design.md` 在 `doc/`。仓内已有 `openspec/changes/{archive,ohos-plugin-architecture-analysis,webview-transparent-plan}`。若 openspec 是设计归档约定位置，应归档；若 `doc/` 是 OHOS 设计文档约定位置（看已有多个 `doc/ohos-*.md`），则可忽略。请按仓约定处理。

### i4. `entry_mobile/oh-package-lock.json5` / `entry_desktop/oh-package-lock.json5` 入库（H1，沿袭旧模板）

- checklist H1 列 `oh-package-lock.json5` 为不应提交的 lock 文件。但旧 `entry/oh-package-lock.json5` 本就被跟踪，新模板沿袭。属模板脚手架，可接受；若要严格遵守 H1，可考虑从模板移除（生成时由 ohpm 重建）。

---

## 已核对正确的设计要点（无 finding）

- ✅ 模块名统一用下划线 `entry_mobile`/`entry_desktop`（`module.json5:3`、`oh-package.json5:2`），符合 hvigor 命名约束
- ✅ `TAURI_OHOS_SKIP_DEVECO_SCRIPT` 守卫：`build`/`--app` 置位、`dev` 不置位、`--open` 不置位——与设计 §3 / 阶段 1 完全一致（`build.rs:329-341`、`hvigorfile.ts:24`）
- ✅ tauriPlugin 烘焙 `OHOS_DEVICE_TYPE={{form}}`，仅 `--open`/IDE 路径生效（`hvigorfile.ts:28`）
- ✅ `compile_lib` 的 `--dist` 按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`（本地 `cargo-mobile2/target.rs:205-209`）
- ✅ `app::build` 走 `assembleApp`（project 级，不传 `--mode module`），glob `build/outputs/default/*.app`
- ✅ 签名泛化为 HAP/.app 通用，`sign-app` 子命令对两者通用（`signing.rs:160`）
- ✅ `dev`/`run` 正确激活单 entry 模块（`dev.rs:328-330`、`run.rs` `app: false`）
- ✅ 注释全英文（H7 合规），无中文注释混入
- ✅ `cfg` 互补不变，每个 entry 是独立单形态 cargo build，不触碰 both-true 雷区

---

## 建议处理顺序

1. **M1（TestTrayAbility 倒置）** — 最可能影响实际打包结果，优先确认意图并修正
2. **M2（mobile 设备类集合）** — 与设计文档对齐，影响 AGC 分发语义
3. **i1（patch 文件）** — 提交前清理
4. m1–m5 可一并打磨

---

## 复核回复（2026-07-01，对照当前工作区实测）

> 以下结论均以当前工作区实际代码为准重新核对（非基于检视当时快照）。

### M1（TestTrayAbility 倒置）— ❌ 检视失准，无需修改

实测与检视描述**相反**：
- `entry_mobile/src/main/module.json5` 中 `TestTrayAbility` 计数 = **0**
- `entry_desktop/src/main/module.json5` 中 `TestTrayAbility` 计数 = **2**（`name` + `srcEntry`）

代码现状正是设计预期（tray 属 desktop 形态，mobile 不声明）。检视者可能看了早期拆分前的快照或读反了。**不修改。**

### M2（mobile 设备类集合）— 🟡 改文档，不改代码

- 代码 `mod.rs:354` `MOBILE = ["phone","tablet","car","wearable","tv"]`
- 设计 §4 写 `mobile 类 ← phone, tablet`

代码与 `demo3signature` 实测一致（demo3 entry 的 `deviceTypes = [phone,tablet,car,wearable,tv]`），语义合理（car/wearable/tv 均为非 desktop 形态）。**结论：代码正确，更新设计 §4 把 mobile 类扩到 `{phone,tablet,car,wearable,tv}`，并说明依据（与 demo3 一致）。**

### m1（注释用连字符 `entry-mobile`）— 🔵 真，需改

实测仍存在连字符注释：
- `mod.rs:340,342`（`active_entry_module` doc）
- `plugins.rs:566,567`（`write_build_profile_modules` doc）
- `project.rs:53,55,68,103`
- `cargo-mobile2/src/open_harmony/target.rs:201`（`entry-{OHOS_DEVICE_TYPE}`）

代码生成用下划线（hvigor 模块名禁连字符），注释却用连字符，误导维护者。**统一改为下划线 `entry_mobile`/`entry_desktop`/`entry_{OHOS_DEVICE_TYPE}`。**

### m2（`--app` 与 `--open` 不互斥）— 🔵 真，需改

`build.rs:78` `--app` 仅 `conflicts_with = "device_type"`，未与 `open` 互斥。`build --app --open` 静默忽略 `--open`。**给 `app` 字段加 `conflicts_with = "open"`。**

### m3（app_paths glob 残留 signed 包）— 🔵 低风险，可选加固

正常流程下 `assembleApp` 每次产新 `*-unsigned.app`（mtime 最新），`reduce(last_modified)` 取到 unsigned，签名正确。仅当 signed 包被外部 touch 成最新时才出问题（极罕见）。`sign_if_configured` 的 `has_signed` 检查的是传入的 outputs（非扫目录），检视 scenario 3 实际不成立。**可选加固：`sign_if_configured` 跳过已是 `-signed` 的路径，使签名幂等。**

### m4（`--app` 下仍 `set_var` mobile）— 🔵 功能正确，不改

`build.rs:115` 无条件 `set_var("OHOS_DEVICE_TYPE", "mobile")` → `inject_plugins`（line 180）在 mobile 下跑，写 entry_mobile 的 dialog dep → `--app` 循环按形态各 `set_var` + `update_entry_package` 补写 desktop。功能正确（两 entry 都拿到 dialog dep），"脆弱"是 code smell 不是 bug。重构（`inject_plugins` 进循环）会重复 detect/copy HAR，更差。**不修改，最多加注释。**

### m5（write_build_profile_modules 重建 entry 丢自定义）— 🔵 可接受，加注释

重建 entry 模块仅写 `name/srcPath/targets`。模板工程 entry build-profile 无自定义字段，不丢东西。**加注释说明会重建即可。**

### i1（`devtools-test-cases.patch`）— ℹ️ 真，提交前清理

根目录存在该 untracked patch 文件。**提交前删除或加入 `.gitignore`。**

### i2（manual_tests.md 补 --app 用例）— ℹ️ 可选

新增 CLI 能力，补一条 `.app` 打包+签名+安装手动用例更好，非必须。

### i3（设计文档位置）— ℹ️ 跳过

`doc/` 下已有多个 `ohos-*.md`，是 OHOS 设计文档约定位置，不迁移。

### i4（oh-package-lock.json5 入库）— ℹ️ 跳过

旧模板沿袭，属脚手架，可接受。

---

### 处置汇总

| 优先级 | 项 | 动作 |
|---|---|---|
| 必改 | m1 | 注释连字符 → 下划线 |
| 必改 | m2 | `--app` 加 `conflicts_with = "open"` |
| 必改（文档） | M2 | 设计 §4 mobile 类扩到 `{phone,tablet,car,wearable,tv}` |
| 必改 | i1 | 提交前删除 `devtools-test-cases.patch` |
| 可选 | m3 | `sign_if_configured` 跳过已签名路径 |
| 可选 | m5 | `write_build_profile_modules` 加重建注释 |
| 跳过 | M1 | 检视失准（实际已正确） |
| 跳过 | m4 | 功能正确，code smell 可接受 |
| 跳过 | i2, i3, i4 | 非必须 / 仓约定 / 沿袭 |

> 其中 **M1 经实测为误报**（实际 entry_desktop 有 TestTrayAbility、entry_mobile 无），建议检视者复核。

