# OHOS 多设备 .app 一次性构建方案（A 方案）

## 目标

支持开发者用一条命令 `cargo tauri ohos build --app` 一次性编译出包含 mobile / desktop 两种设备形态 Entry HAP 的 `.app` 包，用于 AppGallery 统一上架；AppGallery 按用户设备类型自动分发匹配的 HAP。

同时保留现有 `cargo tauri ohos build --device-type mobile|desktop` 单 HAP 构建路径用于开发调试。

## 背景与约束

### 三个原本脱节的机制

| 机制 | 作用 | 时机 | 来源 |
|------|------|------|------|
| `--device-type mobile\|desktop` | 选 Tauri 编译期 cfg（`cfg(mobile)`/`cfg(desktop)`），决定 Runtime / window 后端 / tray 等结构级代码路径 | 编译期 | CLI flag → `OHOS_DEVICE_TYPE` env |
| `tauri.conf.json` `bundle.openHarmony.deviceTypes` | AGC 上架发布设备集（`.app` 内所有 HAP 的 deviceTypes 并集） | 分发期 | 配置文件 |
| module.json5 `deviceTypes` | 单个 HAP 的安装期设备路由 | 安装期 | 模板渲染 |

### 关键约束

1. **`cfg(mobile)` / `cfg(desktop)` 是结构级、编译期、互斥的**。它们选的是不同的 Runtime / window 后端 / 类型与 trait impl，不是"建不建 tray"这种运行期开关。同一个二进制无法同时包含两套形态代码（both-true 会在同名 item 上编译冲突）。因此 **mobile 和 desktop 必须是两个独立编译的 .so，无法合并成单 HAP**。这决定了多设备分发必须是"两个 Entry HAP"。
2. **多 Entry HAP 合法**（已确认）：一个 `.app` 可含多个 Entry HAP，各自 module.json5 声明互斥的 `deviceTypes`，系统按设备只装匹配的那个。
3. **`deviceTypes` 字段在 module.json5 里，是按模块声明的**。所以"按设备形态分发"要靠**多个 entry 模块**，而不是单 entry 模块挂多个 hvigor product。
4. AGC 规则：AGC 勾选的"支持设备" ⊆ `.app` 内所有 Module `deviceTypes` 的并集。即 conf `deviceTypes` = 两个 HAP deviceTypes 的并集。
5. 三条铁律：OHOS 代码经 `openharmony-ability` 桥接；`cfg(target_env = "ohos")` 隔离，不影响其他平台；`OHOS_DEVICE_TYPE` 决定形态。

## 现状构建流程（已读源码确认）

`cargo tauri ohos build --device-type X` 当前链路：

```
1. set_var("OHOS_DEVICE_TYPE", X)                         build.rs:110
2. first_target.build() → ohrs build --arch aarch64       target.rs:201
       --dist entry/libs -- <cargo args>                  (.so 编译，OHOS_DEVICE_TYPE 经 env→cargo→cfg)
3. hap::build() → hvigorw --mode module assembleHap       hap.rs:48-56
       --parallel --incremental -p buildMode=...          (无 product/module 参数)
   └─ hvigor 期间 tauriPlugin 钩 default@ConfigureCMake   hvigorfile.ts:25
       └─ cargo tauri ohos dev-eco-studio-script          → 又一次 Target::build → 又一次 ohrs
4. sign_hap_if_configured() → hap-sign-tool.jar sign-app  build.rs:252
```

关键事实：

- **.so 当前被编两次**：步骤 2 `first_target.build` + 步骤 3 tauriPlugin 回调，都走 `ohrs`。跨平台通用 wart（见"已确认事实"第 5 条），本设计用 skip 守卫在 CLI 路径消除。
- **cargo-mobile2 只有 `assembleHap`，没有 `assembleApp`**；HAP 产物路径硬编码 `entry/build/default/outputs/default/entry-default-*.hap`（[hap.rs:29-35](../../cargo-mobile2/src/open_harmony/hap.rs)）。
- 模板是**单 entry 模块 + 单 product "default"**；module.json5 `deviceTypes` 由 conf 原样灌入（[init.rs:236-238](../crates/tauri-cli/src/mobile/init.rs)）。
- tauriPlugin 钩 `<product>@ConfigureCMake` afterRun，只读 `properties.target`（arch），**不读 product、不设 OHOS_DEVICE_TYPE**。
- 模板结构：`entry/`（Entry HAP）+ `tauri/`（HAR，共享 ArkTS 桥）+ `dialog/`。

## 总体方案：A — hvigor `assembleApp` 一步到位

OHOS 原生提供 `hvigorw assembleApp`：一次 hvigor 调用把所有模块的 HAP 组装并打包成 `.app`（`MakeProjectPackInfo` 生成 pack.info → `PackageApp` 打包 → `SignApp` 签名）。两个形态的 .so 由 CLI 按形态各编一次（`first_target.build` × N，两个不同二进制省不掉），assembleApp 只负责把它们打进 HAP + `.app`，无需 Tauri 手工拼装。

为此需要：

1. **模板**从单 entry 改为**双 entry 模块**（entry_mobile / entry_desktop，所有 OHOS 构建统一），各自 module.json5 `deviceTypes` = 该形态的切分子集，各自 hvigorfile.ts 的 tauriPlugin **按形态烘焙** `OHOS_DEVICE_TYPE`（compile_lib 据此推 `entry_{form}/libs`，保 IDE/`--open` 直构正确），并加 **skip 守卫**。
2. **cargo-mobile2** 新增 `app::build`，调 `hvigorw assembleApp`，返回 `.app` 产物路径；`compile_lib` 的 `--dist` 改为按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`（原硬编码 `entry/libs` 弃用）。
3. **Tauri CLI** 新增 `build --app` 子命令：按 conf `deviceTypes` 切分 → 对齐生成的 OHOS 工程 → **按形态显式编 .so**（`first_target.build` × N，设 `OHOS_DEVICE_TYPE` 各 entry，compile_lib 推 `entry_{form}/libs`）→ 置 `TAURI_OHOS_SKIP_DEVECO_SCRIPT`（CLI 跳过 tauriPlugin）→ 调 `app::build` → 签 `.app`。
4. **tauriPlugin 加 skip 守卫**：CLI **一次性构建**路径（`build` / `--app`）置 `TAURI_OHOS_SKIP_DEVECO_SCRIPT`，tauriPlugin 检测到即 no-op；`--open`/DevEco 直构不置，tauriPlugin 正常编 .so。这样 CLI 一次性构建 .so 只由 `first_target.build` 编一次（消除双编译），且不再调 `dev-eco-studio-script`→`read_options`（消除 WS 回调的 CI 挂起隐患）。**`dev` 不 skip**——dev 的 watch 热重载链（`run`→`device.run`→`hap::build`→tauriPlugin）依赖 tauriPlugin 重建 .so，闭包本身不再调 `target.build`；skip 掉会导致改 Rust 代码后不重编 .so。dev 保留双编译现状，后续若要消除需重构闭包改为每次重建显式调 `target.build`（独立工作，不在本设计内）。

## 已确认事实（本地 SDK 实测）

环境：`OHOS_HOME=C:\myprogram\DevEcoStudio\sdk\default\openharmony`，`hvigorw` 在 `DevEcoStudio/tools/hvigor/bin`，`hap-sign-tool.jar` 在 `$OHOS_HOME/toolchains/lib/`。在生成工程 `examples/api/src-tauri/gen/ohos/` 上实测 `hvigorw tasks` 得到 project（`ohos`）节点任务图：

```
assembleApp   — Assemble the task for the packaged app.   (编排)
├─ PackageApp — Build the app package in the stage model. (打包 .app)
├─ SignApp    — Sign the app package.                     (签名)
├─ MakeProjectPackInfo — Generate project pack.info       (APP 级清单)
└─ GeneratePackRes — Build the pack.res file
```

确认结论：

1. **`hvigorw assembleApp` 是 project 级任务**（`ohos` 节点，非 module 级），编排上图 `PackageApp`/`MakeProjectPackInfo`/`SignApp`。调用 `hvigorw assembleApp --parallel --incremental -p buildMode=...`，**不传 `--mode module`**（那是单 HAP 的 module 级模式，见 [hap.rs:49-51](../../cargo-mobile2/src/open_harmony/hap.rs)）。
2. **`.app` 产物路径**（demo3signature 双 entry 工程实测）：**项目级** `build/outputs/default/`（注意：不是 HAP 的 `entry/build/default/outputs/default/`），文件名 `<projectName>-default-unsigned.app` / `<projectName>-default-signed.app`（demo3 为 `demo3signature-default-signed.app`）。前缀随工程名，故 `app::build` 宜 glob `build/outputs/default/*.app` 取最新，不硬编码前缀。
3. **pack.info 自动生成**：`MakeProjectPackInfo` 从 build-profile.json5 的 `modules` 列表 + 各模块 module.json5 的 `deviceTypes` 生成。实测单 entry 的 pack.info：
   ```json
   "packages":[{"deviceType":["phone","tablet","2in1"],"moduleType":"entry","deliveryWithInstall":true,"name":"entry-default"}]
   ```
   多 entry 时 `packages[]` 自动列出多个 HAP（各自 deviceType + name）。**无需手写 pack.info**，只要在 build-profile 声明两个 entry 模块 + 各自 module.json5 deviceTypes。
4. **`sign-app` 可直接签 `.app`**（[华为博客](https://developer.huawei.com/consumer/cn/blog/topic/03202586123166309)实证）：参数与签 HAP 完全相同，无需 `-inForm`/`-compatibleVersion`（`-h` 文本只列 `.hap/.bin/.elf` 是不完整）。故 `--app` 与单 HAP 签名统一——assembleApp 出未签名 `.app` → 事后 `sign-app` 签，**无需** build-profile `signingConfigs`，模板维持 `signingConfigs: []`。命令见第 7 节。

> 调试路径（本机 hdc 安装）：`.app` 无法直接 hdc 安装，需 `app_unpacking_tool.jar` 拆包成 HAP 再用**调试证书**逐个签 HAP 安装（见上述博客）。`--app` 面向发布上架；本机调试仍用单 HAP `build --device-type` + hdc install。

5. **tauriPlugin（cargo↔hvigor 桥）**：模板 [hvigorfile.ts](../crates/tauri-cli/templates/mobile/open-harmony/entry/hvigorfile.ts) 的 tauriPlugin 钩 `<product>@ConfigureCMake` afterRun，同步 `execFileSync(cargo tauri ohos dev-eco-studio-script)` → 经 WebSocket 向父进程取 `CliOptions`（`write_options`/`read_options`，[mod.rs:341/379](../crates/tauri-cli/src/mobile/mod.rs)）→ `ohrs` 编 .so。它是**跨平台既定设计**（Android Gradle 回调 / iOS Xcode Build Rust Code 同构，共用 WS 机器），CLI 路径与 `first_target.build` 双编译是跨平台通用 wart。本设计加 `TAURI_OHOS_SKIP_DEVECO_SCRIPT` 守卫在 OHOS CLI 路径跳过它（OHOS 特有偏离，治 WS 回调 CI 挂起），`--open`/IDE 直构仍由它编 .so。`taskTree` 实测 `:ohos:assembleApp` 只依赖 `::SignApp`、不依赖 entry 的 `ConfigureCmake`（仅 `:entry:assembleHap` 才依赖），故 `--app` 走 CLI 显式编 + skip 后此问题 moot。直接在 DevEco 打开工程编译（非 `--open`）会因无 CLI 父进程、`read_options` 连不上 WS 而 panic——现状已知限制，本设计不处理。
6. **cargo-mobile2 为本地 path 依赖**：[Cargo.toml:75](../Cargo.toml) 已是 `path = "../cargo-mobile2"`，本地仓 `c:\myprogram\code\tauri\cargo-mobile2`（feat/ohos，HEAD 5060cae）。当前无 `app.rs` / `assembleApp`，`compile_lib` 的 `--dist` 仍硬编码 `entry/libs`，本设计的 cargo-mobile2 改动直接落本地仓，无需上游 PR。
7. **双 entry + 共享 HAR 实测可行**（`C:\myprogram\code\tauri\demo3signature`）：`entry`(entry, mobile 类) + `desktop`(entry, `["2in1"]`) + `ohability`(har) 三个模块，`assembleApp` 产 `demo3signature-default-signed.app`，pack.info 自动列两个 entry 互斥分发。印证本设计 mobile/desktop 双 entry 切分（desktop=`2in1` 一致）+ 共享 HAR（Tauri 的 `tauri` HAR 同理）。

## 详细设计

### 1. 命令模型

| 命令 | 产物 | 形态 | 用途 |
|------|------|------|------|
| `build --device-type mobile\|desktop`（现有，保留） | 单 HAP | 显式单值 | 开发/调试/单设备 |
| `build --app`（新增） | `.app`（1~2 个 Entry HAP） | 从 conf `deviceTypes` 推导 | 整包上架 |

`--app` 产出的 `.app` 含几个 HAP 由 conf 决定：
- conf = `["phone","tablet","2in1"]` → mobile HAP（phone/tablet）+ desktop HAP（2in1）
- conf = `["phone","tablet"]` → 仅 mobile HAP 的 `.app`
- conf = `["2in1"]` → 仅 desktop HAP 的 `.app`

### 2. 模板结构：双 entry 模块

```
templates/mobile/open-harmony/
├── build-profile.json5        # modules 列 entry_mobile + entry_desktop + tauri + dialog
├── AppScope/app.json5
├── entry_mobile/              # Entry HAP, cfg(mobile)
│   ├── build-profile.json5
│   ├── hvigorfile.ts          # tauriPlugin: 烘焙 OHOS_DEVICE_TYPE=mobile + skip 守卫
│   └── src/main/module.json5  # deviceTypes = mobile 子集
├── entry_desktop/             # Entry HAP, cfg(desktop)
│   ├── build-profile.json5
│   ├── hvigorfile.ts          # tauriPlugin: 烘焙 OHOS_DEVICE_TYPE=desktop + skip 守卫
│   └── src/main/module.json5  # deviceTypes = desktop 子集
├── tauri/                     # HAR, 共享 ArkTS 桥（不变）
└── dialog/                    # 共享（不变）
```

两个 entry 模块的 `module.json5` 结构相同（type=entry、mainElement=EntryAbility、abilities 等），仅 `deviceTypes` 与所依赖的 .so 不同。两者都依赖共享的 `tauri` HAR。

**此双 entry 模板是所有 OHOS 构建的统一模板**：单 HAP `build --device-type X` 时 build-profile `modules` 只列 `entry_{form}`（激活一个），`--app` 时列两个。模块名固定 `entry_mobile`/`entry_desktop`，`compile_lib` 据此按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`。

> **OHOS 模块名约束（实测）**：hvigor 模块名（module.json5 `module.name` + build-profile `modules[].name`）必须匹配 `^[a-zA-Z][0-9a-zA-Z_.]*$`，**禁连字符**。故用下划线 `entry_mobile`/`entry_desktop`（目录名/模块名/oh-package name/hap 名 `entry_{form}-default-*.hap` 四者一致），不用 `entry-mobile`。

### 3. cfg 传播：CLI 显式编 + tauriPlugin skip 守卫

形态 cfg 的传播分两条路径：

**CLI 一次性构建路径（`build` / `--app`）——显式编，tauriPlugin 跳过：**
- Tauri CLI 设 `OHOS_DEVICE_TYPE=<form>` 进程 env（已有机制，[build.rs:110](../crates/tauri-cli/src/mobile/open_harmony/build.rs)）。
- 调 `first_target.build`（`ohrs`）；`compile_lib` 按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`，.so 落到对应 entry 模块。
- 设 `TAURI_OHOS_SKIP_DEVECO_SCRIPT=1`，tauriPlugin 检测到即 no-op，不调 `dev-eco-studio-script`、不触发 WS 回调。.so 完全由 CLI 这一次编译决定。
- **`dev` 不在此路径**：dev 的 watch 热重载依赖 tauriPlugin 重建 .so（见上"tauriPlugin 处理"），skip 会破坏热重载。dev 保留 `target.build` 初始编 + tauriPlugin 重建的双编译现状。

**IDE / `--open` 路径——tauriPlugin 编，按形态烘焙：**
- `--open` 不置 skip env；用户在 DevEco 里构建时 hvigor 触发 tauriPlugin，由它编 .so。
- 每个 entry 模块的 hvigorfile.ts 烘焙本模块形态，保证 IDE 直构也编对形态：

```typescript
function tauriPlugin(): HvigorPlugin {
  return {
    pluginId: 'tauri',
    apply(node: HvigorNode) {
      const buildRustCode = () => {
        if (process.env.TAURI_OHOS_SKIP_DEVECO_SCRIPT) return;   // CLI 路径已显式编过，跳过
        const properties = hvigor.getParameter().getProperties();
        const target = properties.target || "aarch64";
        process.env.OHOS_DEVICE_TYPE = "{{form}}";            // mobile/desktop，按 entry 模块烘焙
        execFileSync(`{{tauri-binary}}`,
          [{{quote-and-join tauri-binary-args}}, "--target", target.toString()], {
            cwd: resolve(__dirname, "{{root-dir-rel}}"),
            stdio: "inherit",
          });
      }
      node.getTaskByName('{{product}}@ConfigureCMake')!.afterRun(buildRustCode);
    }
  }
}
```

要点：
- `TAURI_OHOS_SKIP_DEVECO_SCRIPT` 经 hvigorw 子进程继承（与 `OHOS_DEVICE_TYPE` 同通道）。CLI **一次性构建**（`build`/`--app`）置位，`--open`/`dev` 不置（dev 需 tauriPlugin 重建 .so）。`{{form}}` 模板渲染时按 entry 模块烘焙为 `mobile`/`desktop`，仅 IDE/`--open` 路径生效。
- **`compile_lib` 的 `--dist` 按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`**：[target.rs:199](../../cargo-mobile2/src/open_harmony/target.rs) 原硬编码 `entry/libs`，改为 `project_dir.join(format!("entry_{OHOS_DEVICE_TYPE}")).join("libs")`。模板统一 `entry_mobile`/`entry_desktop` 命名，故目录由形态确定，**无需额外 env**；ohrs 已支持 `--dist`，不改 ohrs。
- `--app` 因 skip 了 tauriPlugin，不调 `dev-eco-studio-script`、不经 WS，**无需** `write_options`/`OptionsHandle`（WS 机器仅 `--open`/IDE 路径需要）。不改 `read_options`——IDE 直构（非 `--open`）panic 是现状已知限制，走 `--open`；skip 守卫使 CLI/CI 不受影响。

### 4. deviceTypes 配置（per-form 子字段）

`tauri.conf.json` 的 `bundle.openHarmony.deviceTypes` 拆为两个子字段，各自直接对应一个 entry HAP：

```json
"openHarmony": {
  "deviceTypes": {
    "mobile": ["phone", "tablet"],
    "desktop": ["2in1"]
  }
}
```

- `mobile`（默认 `["phone","tablet"]`）→ `entry_mobile` 的 module.json5 `deviceTypes` + `cfg(mobile)` 编译。
- `desktop`（默认 `["2in1"]`）→ `entry_desktop` 的 module.json5 `deviceTypes` + `cfg(desktop)` 编译。

构建期 `device_types_for_form(conf, form)` 直接取对应子字段（无交集/映射表——形态分类由配置显式给出）。`forms_for_device_types(conf)` 返回非空子字段对应的形态。单 HAP `--device-type X` 若该子字段为空 → bail（config 错误，不写空 `deviceTypes`）。`--app` 跳过空子字段的形态。

AGC 发布集 = `mobile ∪ desktop`。一致性由构造保证（形态从 conf 推导）。

### 5. cargo-mobile2 改动

新增 `src/open_harmony/app.rs`（或扩展 `hap.rs`）：

```rust
pub fn app_paths(config: &Config) -> Vec<PathBuf> {
  // 已确认（demo3signature 实测）：项目级 build/outputs/default/，前缀随工程名
  // 故 glob *.app 取最新，不硬编码前缀
  let output_dir = prefix_path(config.project_dir(), "build/outputs/default");
  // 实现期用 std::fs::read_dir 匹配 *-signed.app / *-unsigned.app，reduce(last_modified)
  // 伪代码：
  vec![output_dir.join("*-signed.app"), output_dir.join("*-unsigned.app")]
}

pub fn build(config: &Config, env: &Env, noise_level: NoiseLevel, profile: Profile)
  -> Result<PathBuf, AppError>
{
  ohpm::install(config, env)?;
  let build_mode = profile.as_str().to_lowercase();
  // 已确认：assembleApp 是 project 级任务，不传 --mode module
  let hvigor_args = vec![
    "assembleApp".to_string(),
    "--parallel".to_string(),
    "--incremental".to_string(),
    "-p".to_string(),
    format!("buildMode={build_mode}"),
  ];
  hvigorw(config, env)
    .before_spawn(move |cmd| {
      cmd.args(&hvigor_args).arg(match noise_level {
        NoiseLevel::Polite => "--info",
        NoiseLevel::LoudAndProud | NoiseLevel::FranklyQuitePedantic => "--debug",
      });
      Ok(())
    })
    .start()?.wait()?;
  // glob *.app 取最新（signed 优先于 unsigned），逻辑同 hap.rs 的 reduce(last_modified)
  Ok(app_paths(config).into_iter().reduce(last_modified).unwrap())
}
```

并导出 `open_harmony::app` 模块。`hap.rs` 保持不变（单 HAP 路径仍用）。

### 6. Tauri CLI `build --app` 流程

`crates/tauri-cli/src/mobile/open_harmony/build.rs`：

1. 解析新增 `--app` flag；`--app` 与 `--device-type` 互斥（`--app` 时形态从 conf 推导）。
2. 读 conf `deviceTypes`，按映射表切分为 mobile/desktop 子集；确定激活的 entry 模块集合（子集非空者）。
3. **对齐生成的 OHOS 工程**（关键，处理 conf 变更无需重新 init）：
   - 改写 `build-profile.json5` 的 `modules` 列表：只列激活的 entry 模块 + tauri + dialog。
   - 改写每个激活 entry 模块的 `module.json5` `deviceTypes` = 该形态子集。
4. `ensure_init` + `inject_plugins` + `inject_resources`（与现有 `build` 一致）。
5. **按形态显式编 .so**：对每个激活形态，set `OHOS_DEVICE_TYPE=<form>`，调 `first_target.build`（`ohrs`）→ `compile_lib` 按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`，.so 落到对应 entry 模块。
6. set `TAURI_OHOS_SKIP_DEVECO_SCRIPT=1`（CLI 跳过 tauriPlugin，避免双编译与 WS 回调）。
7. 调 `cargo_mobile2::open_harmony::app::build(...)` → `hvigorw assembleApp`：组装各 entry 的 HAP（用步骤 5 已编好的 .so）→ `MakeProjectPackInfo` 生成 pack.info → `PackageApp` 打包未签名 `.app`。（tauriPlugin 被 skip，不重编 .so、不经 WS。）
8. `sign_if_configured`：事后用 `hap-sign-tool sign-app` 签 `.app`（`sign-app` 子命令对 `.hap`/`.app` 通用，故单 HAP 与 `--app` 共用同一签名函数；见第 7 节）。
9. `log_finished` 输出 `.app` 路径。

> 单 HAP `build --device-type X` 路径（统一模板下）：激活 `entry_{form}`（build-profile `modules` 只列 `entry_{form}` + tauri + dialog，其余 entry 不列），assembleHap 产 `entry_{form}-default-*.hap`（**原 `entry-default-*.hap` 改名**，现有 hdc 脚本/记忆引用处需同步）。其 module.json5 `deviceTypes` = `conf.deviceTypes.<X>`（该形态子字段）；子字段为空则 bail（config 错误）。

### 6.1 编译流程命令示意

下面以脚本形式示意整个编译过程实际调用的命令（`#` 为说明，命令为示意，非逐字复制）。

```bash
# ===== 一次性：生成 OHOS 工程（模板已含双 entry）=====
cargo tauri ohos init --ci --skip-targets-install
#   渲染 templates/mobile/open-harmony → gen/ohos/
#   产物：entry_mobile/ + entry_desktop/ + tauri/ (+ dialog/) + build-profile.json5
#   各 entry module.json5 deviceTypes = conf ∩ 形态；hvigorfile.ts 烘焙 OHOS_DEVICE_TYPE

# 签名材料走环境变量（不入 build-profile，事后签名）：
export OHOS_KEYSTORE_FILE=.../release.p12   OHOS_KEYSTORE_PASSWORD=a12345678
export OHOS_KEY_ALIAS=release               OHOS_KEY_PASSWORD=a12345678
export OHOS_APP_CERT_FILE=.../release.cer   OHOS_PROFILE_FILE=.../releaseRelease.p7b
export OHOS_SIGN_ALG=SHA256withECDSA

# ===== 单 HAP：cargo tauri ohos build --device-type mobile =====
#   （CLI 内部依次执行：）
pnpm build                              # beforeBuildCommand（若配）
ohrs build --arch aarch64 --dist entry_mobile/libs -- \
      cargo build --target aarch64-unknown-linux-ohos --release   # 编 .so（cfg mobile）
hvigorw --mode module assembleHap --parallel -p buildMode=release # 打 entry_mobile-default-*.hap
java -jar hap-sign-tool.jar sign-app ... \
      -inFile entry_mobile-default-unsigned.hap -outFile entry_mobile-default-signed.hap
#   tauriPlugin 被 TAURI_OHOS_SKIP_DEVECO_SCRIPT=1 跳过，不重编 .so、不经 WS

# ===== 多设备 .app：cargo tauri ohos build --app =====
#   （形态从 conf deviceTypes 推导；此处手机+PC → mobile + desktop）
pnpm build                              # beforeBuildCommand（若配）
# 按形态各编一次 .so（设 OHOS_DEVICE_TYPE 决定 cfg 与 --dist 目录）：
OHOS_DEVICE_TYPE=mobile  ohrs build --arch aarch64 --dist entry_mobile/libs  -- cargo build ... --release
OHOS_DEVICE_TYPE=desktop ohrs build --arch aarch64 --dist entry_desktop/libs -- cargo build ... --release
# 对齐工程：build-profile modules=[entry_mobile,entry_desktop,tauri,dialog]；各 entry module.json5 deviceTypes=conf∩形态
export TAURI_OHOS_SKIP_DEVECO_SCRIPT=1
hvigorw assembleApp --parallel --incremental -p buildMode=release
#   编排 :entry_mobile:assembleHap + :entry_desktop:assembleHap + ::PackageApp + ::SignApp → 项目级 *.app
java -jar hap-sign-tool.jar sign-app ... \
      -inFile ohos-default-unsigned.app -outFile ohos-default-signed.app
#   产物：build/outputs/default/ohos-default-signed.app（含两 HAP，pack.info 互斥 deviceTypes）
```

要点：`--app` 与单 HAP 的差别仅在"按形态各编一次 .so + `assembleApp`（project 级）替换单 HAP 的 `--mode module assembleHap`"；签名、skip 守卫、build-profile 对齐两者一致。

### 7. 签名（与单 HAP 路径统一）

已确认 `hap-sign-tool.jar sign-app` 可直接签 `.app`（inFile=.app，参数同 HAP）。因此 `--app` 复用现有事后签名模式，不引入 build-profile signingConfigs：

1. assembleApp 产出未签名 `<projectName>-default-unsigned.app`（项目级 `build/outputs/default/`，模板维持 `signingConfigs: []`，hvigor 不签名）。
2. `sign_app_if_configured`（把现有 `sign_hap_if_configured` 泛化为 HAP/.app 通用）：若 `OhosSigningConfig::from_env()` 存在，调
   ```
   hap-sign-tool.jar sign-app -mode localSign -keyAlias <OHOS_KEY_ALIAS>
     -keyPwd <OHOS_KEY_PASSWORD> -appCertFile <OHOS_APP_CERT_FILE>
     -profileFile <OHOS_PROFILE_FILE> -inFile <projectName>-default-unsigned.app
     -signAlg <OHOS_SIGN_ALG> -keystoreFile <OHOS_KEYSTORE_FILE>
     -keystorePwd <OHOS_KEYSTORE_PASSWORD> -outFile <projectName>-default-signed.app
     -signCode 1
   ```
   产出 `<projectName>-default-signed.app`。无签名环境变量时警告并返回未签名 `.app`（同单 HAP 现有语义）。

现有 `sign_hap_if_configured` 已按 "unsigned→signed" 文件名替换逻辑工作（[build.rs:340-349](../crates/tauri-cli/src/mobile/open_harmony/build.rs)），`.app` 路径同构，泛化为 `sign_app_if_configured` 即可。

### 8. 一致性校验

- `--app`：形态集合从 conf 推导，天然一致；若某形态子集为空，对应 entry 模块不激活、不出现在 `.app`（合法）。
- `build --device-type X`：若 `conf.deviceTypes` 含 X 形态覆盖不到的设备（如 `--device-type mobile` 但 conf 含 `2in1`），警告"声明设备无对应 HAP 覆盖，AGC 可能无法分发"。

## 影响面与铁律合规

| 改动位置 | 内容 | 铁律合规 |
|----------|------|----------|
| `cargo-mobile2`（本地 path 依赖 `../cargo-mobile2`，feat/ohos） | 新增 `app::build`（assembleApp）；`compile_lib` 的 `--dist` 按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs` | 仅 OHOS 路径，不影响其他平台 |
| `tauri-cli` 模板 `open-harmony/` | 单 entry → 双 entry；hvigorfile.ts 按形态烘焙 + skip 守卫 | OHOS 隔离 |
| `tauri-cli` `open_harmony/build.rs` | 新增 `--app`；按形态显式编 .so + 置 skip env；deviceTypes 切分；单 HAP 路径 deviceTypes 修正 | OHOS 隔离 |
| `tauri-utils` `config.rs` | `OpenHarmonyConfig.device_types` 改为 per-form struct `OpenHarmonyDeviceTypes { mobile, desktop }`（各默认 `["phone","tablet"]` / `["2in1"]`）；`config.schema.json` 同步 | 无影响 |
| `tauri` crate `build.rs` 等 cfg 别名 | **不改**——单次 cargo build 仍单形态，`desktop = !mobile` 互补不变 | 铁律 #3 不变 |

`cfg` 互补不变是本方案的关键：每个 entry 模块是一次独立的单形态 cargo build，永远只有一个形态为真，不触碰 both-true 的雷区。

## 待确认 / 风险

已坐实（见"已确认事实"）：assembleApp 调用/任务编排/产物路径、pack.info 自动生成、`sign-app` 可签 `.app`、tauriPlugin 跨平台同构、双 entry + 共享 HAR 可行（demo3 实测）。已定：desktop 类 `2in1`；cargo-mobile2 本地 path 依赖；skip 守卫 `TAURI_OHOS_SKIP_DEVECO_SCRIPT`；不改 `read_options`（IDE 直构 panic 走 `--open`）。剩余待实跑确认：

1. **skip 守卫实效**：CLI 路径置 `TAURI_OHOS_SKIP_DEVECO_SCRIPT` 后 tauriPlugin 是否确为 no-op、`cargo tauri ohos build` 是否不再双编译 .so、不再触发 `dev-eco-studio-script`/WS（CI 不挂）。实跑验证。
2. **`--open` 不回归**：`--open` 不置 skip，tauriPlugin 经 WS 取 `CliOptions`，确认与现有 `--open` 行为一致。
3. **Tauri 双 entry 模板**：demo3 是纯 ArkTS 工程；Tauri 的双 entry（每个带 .so + tauriPlugin 烘焙形态）能否同样跑通 assembleApp，实跑确认。

## 分阶段实施（skip 守卫最优先，独立可落地）

1. **阶段 1（最高优先）— skip 守卫**（独立于 `--app`，先落地）：
   - 模板 `entry/hvigorfile.ts`：`buildRustCode` 开头 `if (process.env.TAURI_OHOS_SKIP_DEVECO_SCRIPT) return;`。
   - Tauri CLI `build.rs`：在 `first_target.build` 之后、调 hvigor（`hap::build` / `app::build`）之前 `set_var("TAURI_OHOS_SKIP_DEVECO_SCRIPT", "1")`，**仅非 `--open` 路径**（`--open` 不置）。**不改 `dev.rs`**——dev watch 热重载依赖 tauriPlugin 重建 .so（见"tauriPlugin 处理"）。
   - 验证：`cargo tauri ohos build` 不再双编译 .so、不再调 `dev-eco-studio-script`/WS；`--open` 仍正常。
2. **阶段 2 — cargo-mobile2 `app::build`**：新增 `app::build`（assembleApp），`app_paths` glob `build/outputs/default/*.app`；`compile_lib` 的 `--dist` 改为按 `OHOS_DEVICE_TYPE` 推 `entry_{form}/libs`。
3. **阶段 3 — 模板双 entry**：单 entry → entry_mobile / entry_desktop（仿 demo3 的 entry+desktop），共享 `tauri` HAR；各 hvigorfile.ts 按形态烘焙 `OHOS_DEVICE_TYPE`（保留 skip 守卫）。单 HAP `build --device-type` 同步改为激活单个 `entry_{form}`、产 `entry_{form}-default-*.hap`。
4. **阶段 4 — Tauri CLI `build --app`**：`--app` flag、conf 切分、工程对齐（build-profile modules + 各 entry module.json5 deviceTypes）、按形态显式编 .so + 置 skip、调 `app::build`、签 `.app`；同步修正单 HAP 路径 deviceTypes。
5. **阶段 5 — 验证**：`--app` 产出双 entry `.app`，本地 hdc 安装到 phone/2in1 验证路由；`--open` 不回归；AGC 上架测试分发。
