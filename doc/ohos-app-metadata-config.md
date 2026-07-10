# OHOS 应用元信息配置设计方案

## 1. 背景与目标

Tauri 应用通过 `tauri.conf.json` 统一管理应用元信息（名称、图标、版本、标识符等），各平台 CLI 在构建时将这些信息映射到平台原生配置文件中。目前 OHOS 平台的基础映射已实现，但存在多项缺失和硬编码问题，无法像 macOS/Windows/Android 那样完整地从统一配置生成应用元信息。

**目标**：使开发者仅通过 `tauri.conf.json`（和可选的 `tauri.ohos.conf.json`）即可完整配置 OHOS 应用的名称、图标、版本等元信息，无需手动编辑生成的 OHOS 项目文件。

## 2. 其他平台参考

### 2.1 各平台元信息映射一览

| 元信息 | macOS | Windows | Linux | Android | iOS |
|--------|-------|---------|-------|---------|-----|
| **应用名称** | `productName` → `Info.plist` CFBundleDisplayName | `productName` → installer config | `productName` → `.desktop` file | `productName` → `strings.xml` app_name | `productName` → `Info.plist` |
| **标识符** | `identifier` → CFBundleIdentifier | `identifier` → WiX Product ID | `identifier` → package name | `identifier` → applicationId | `identifier` → Bundle Identifier |
| **版本** | `version` → CFBundleShortVersionString | `version` → installer ProductVersion | `version` → `.desktop` file | `version` → versionName, auto-increment versionCode | `version` → CFBundleShortVersionString |
| **图标** | `bundle.icon` → `icon.icns` + `AppIcon.icon` | `bundle.icon` → `icon.ico` + appx tiles | `bundle.icon` → 多尺寸 PNG | `bundle.icon` → adaptive icon 全套 | `bundle.icon` → AppIcon assets |
| **分类** | `bundle.category` → LSApplicationCategoryType | — | `bundle.category` → `.desktop` Categories | — | `bundle.category` → LSApplicationCategoryType |
| **发布者** | — | `bundle.publisher` → Manufacturer | `bundle.publisher` → Maintainer | — | — |
| **版权** | `bundle.copyright` | `bundle.copyright` | — | — | `bundle.copyright` |

### 2.2 Android 的参考价值（与 OHOS 最相似）

Android 平台是 OHOS 最直接的参考，因为两者都使用：
- **自适应图标**（前景层 + 背景层）
- **资源字符串**定义应用名称
- **声明式配置文件**定义权限、设备类型等

Android 的图标生成流程（`icon.rs`）：
1. 读取源图（PNG/SVG）或 manifest JSON（支持 `android_fg`、`android_bg`、`android_monochrome` 分别指定）
2. 生成 5 种 DPI 尺寸的前景/背景/单色图标
3. 生成 `ic_launcher.xml` 自适应图标定义
4. 输出到 `gen/android/` 的 mipmap 目录

### 2.3 tauri.conf.json → OHOS 配置文件完整映射

以下是每个 tauri.conf.json 配置项与 OHOS 原生配置文件的对应关系：

| tauri.conf.json 字段 | OHOS 目标文件 | OHOS 字段 | 说明 |
|---|---|---|---|
| `identifier` | `AppScope/app.json5` | `app.bundleName` | `-` 替换为 `_` |
| `productName` | `AppScope/resources/base/element/string.json` | `string[app_name].value` | 应用显示名称 |
| `productName` | `entry/resources/base/element/string.json` | `string[EntryAbility_label].value` | Ability 标签 |
| `version` | `AppScope/app.json5` | `app.versionName` | 版本字符串 |
| `bundle.publisher` | `AppScope/app.json5` | `app.vendor` | 应用发布者 |
| `bundle.icon` 中 `*-foreground.png` | `*/resources/base/media/foreground.png` | `$media:foreground` | 自适应图标前景层 |
| `bundle.icon` 中 `*-background.png` | `*/resources/base/media/background.png` | `$media:background` | 自适应图标背景层 |
| `bundle.icon` 中 `*-starticon.png` | `entry/resources/base/media/startIcon.png` | `$media:startIcon` | 启动窗口图标（若无 `-starticon` 则使用前景图） |
| （bundler 自动生成） | `*/resources/base/media/layered_image.json` | `$media:layered_image` | 自适应图标定义文件 |
| `bundle.openHarmony.versionCode` | `AppScope/app.json5` | `app.versionCode` | 版本整数 |
| `bundle.openHarmony.deviceTypes` | `entry/src/main/module.json5` | `module.deviceTypes` | 支持的设备类型 |
| `bundle.shortDescription` | `entry/resources/base/element/string.json` | `string[module_desc].value` + `string[EntryAbility_desc].value` | 模块和 Ability 描述（复用已有字段） |

---

## 3. 当前 OHOS 状态与差距分析

### 3.1 已实现

| 配置项 | tauri.conf.json 字段 | OHOS 映射 | 状态 |
|--------|----------------------|-----------|------|
| 包名 | `identifier` | `app.json5` → `bundleName` | ✅ 已实现 |
| 应用名 | `productName` | `string.json` → `app_name` | ✅ 已实现 |
| 发布者 | `bundle.publisher` | `app.json5` → `vendor` | ✅ 已实现 |
| 版本号 | `version` | `app.json5` → `versionName` + `versionCode`（自动计算） | ✅ 已实现 |
| 模块描述 | `bundle.shortDescription` | `string.json` → `module_desc` / `EntryAbility_desc` | ✅ 已实现 |
| 图标 | `bundle.icon` 中 `-foreground`/`-background`/`-starticon` | `media/foreground.png` 等 | ✅ 已实现 |
| OHOS 配置节 | `bundle.openHarmony` | `versionCode` / `deviceTypes` | ✅ 已实现 |
| 设备类型 | `bundle.openHarmony.deviceTypes` | `module.json5` → `deviceTypes` | ✅ 已实现（按 `OHOS_DEVICE_TYPE` 自动调整见 TODO 9.1） |

### 3.2 待办（见第 9 节 TODO）

| # | 待办 | 严重程度 | 说明 |
|---|------|----------|------|
| G6 | **设备类型未按 `OHOS_DEVICE_TYPE` 自动调整** | 🟠 低-中 | `deviceTypes` 默认值固定为 `["phone", "tablet", "2in1"]`，尚未根据 `OHOS_DEVICE_TYPE` 自动收窄（见 TODO 9.1） |
| — | **build app 适配** | 🟠 中 | 当前仅适配 `build hap`，未适配 `build app`（见 TODO 9.2） |
| — | **不同设备形态 HAP 打包** | 🟠 中 | 当前只生成通用 HAP，未按设备形态分别打包（见 TODO 9.3） |

---

## 4. 设计方案

### 4.1 配置映射设计（修复 G2-G4）

#### 4.1.1 模板变量修复

将以下硬编码值改为模板变量：

**`AppScope/app.json5`**：
```json5
{
  "app": {
    "bundleName": "{{app.identifier}}",        // ✅ 已正确
    "vendor": "{{bundle.publisher}}",              // 修复：注册 publisher 变量
    "versionCode": {{bundle.open-harmony.version-code}},       // 新增：从配置读取
    "versionName": "{{version}}",           // 新增：从 tauri.conf.json version 读取
    "icon": "$media:layered_image",             // ✅ 保持不变
    "label": "$string:app_name"                 // ✅ 已正确
  }
}
```

**`entry/src/main/resources/base/element/string.json`**：
```json
{
  "string": [
    { "name": "module_desc", "value": "{{bundle.short-description}}" },
    { "name": "EntryAbility_desc", "value": "{{bundle.short-description}}" },
    { "name": "EntryAbility_label", "value": "{{app.stylized-name}}" }
  ]
}
```

#### 4.1.2 变量来源

| Handlebars 变量 | 数据来源 | 默认值 |
|-----------------|----------|--------|
| `{{app.identifier}}` | `tauri.conf.json > identifier` | （必填） |
| `{{app.stylized-name}}` | `tauri.conf.json > productName` | （必填） |
| `{{version}}` | `tauri.conf.json > version` | `"1.0.0"` |
| `{{bundle.publisher}}` | `tauri.conf.json > bundle.publisher` | identifier 第二段 |
| `{{bundle.short-description}}` | `tauri.conf.json > bundle.shortDescription` | `productName` |
| `{{bundle.open-harmony.version-code}}` | `bundle.openHarmony.versionCode` | 从 version 自动计算：`major×1000000 + minor×1000 + patch` |
| `{{bundle.open-harmony.device-types}}` | `bundle.openHarmony.deviceTypes` | `["phone","tablet","2in1"]` |

### 4.2 OHOS Bundle 配置节设计（修复 G5）

参考 `bundle.android` 的设计模式，在 `bundle` 下新增 `openHarmony` 配置节。

#### 4.2.1 精简原则

| 原则 | 说明 |
|------|------|
| 复用已有字段 | `bundle.shortDescription` 替代 `moduleDescription`/`abilityDescription`，`bundle.publisher` 替代 `vendor`，`bundle.icon` + 文件名约定处理图标 |
| 对齐 Android 模式 | `versionCode` 与 Android 对称 |
| 仅保留 OHOS 独有项 | `deviceTypes` 无跨平台对应，必须保留 |
| 不配置可改文件 | `permissions`、`extensionAbilities`、`startWindowBackground` 等直接改生成的文件，不纳入配置 |

#### 4.2.2 最终配置结构

```jsonc
{
  "bundle": {
    // 复用已有字段
    "shortDescription": "我的 Tauri 应用",    // → module_desc + EntryAbility_desc
    "publisher": "Example Inc.",              // → app.vendor

    "openHarmony": {
      // 版本控制（与 bundle.android 对称）
      "versionCode": 1,

      // OHOS 独有
      "deviceTypes": ["phone", "tablet", "2in1"]
    }
  }
}
```

#### 4.2.3 与 Android 配置对比

```
bundle.android                          bundle.openHarmony
─────────────                           ─────────────────
minSdkVersion: u32                      —
versionCode: Option<u32>                versionCode: Option<u32>
autoIncrementVersionCode: bool          —
                                        deviceTypes: Vec<String>  ← OHOS 独有
```

Android 有 3 个字段，OHOS 只有 2 个。`minSdkVersion` 和 `autoIncrementVersionCode` 对 OHOS 不需要——OHOS 的 SDK 版本由 `build-profile.json5` 的 `compatibleSdkVersion` 控制（模板默认值即可），`versionCode` 无需自动递增。`permissions` 和 `startWindowBackground` 直接编辑生成的 `module.json5` 和资源文件即可。

### 4.3 OHOS 图标生成设计（修复 G1）

#### 4.3.1 设计思路

OHOS 使用自适应图标（Layered Image）系统：

| 对比项 | Android | OHOS |
|--------|---------|------|
| 图标定义格式 | XML (`ic_launcher.xml`) | JSON (`layered_image.json`) |
| 前景图位置 | `mipmap-*/ic_launcher_foreground.png` | `resources/base/media/foreground.png` |
| 背景图位置 | `mipmap-*/ic_launcher_background.png` | `resources/base/media/background.png` |
| 单色图标 | `ic_launcher_monochrome.png` | 不支持 |
| DPI 分桶 | 5 种（mdpi ~ xxxhdpi） | 1 种（`base/media/`），系统自动缩放 |
| 启动图标 | 无 | `startIcon.png`（启动窗口图标） |

#### 4.3.2 源图输入

与所有平台一样，通过 `bundle.icon` 统一配置。bundler 按**文件扩展名 + 文件名约定**区分各平台图标：

```jsonc
{
  "bundle": {
    "icon": [
      // macOS
      "icons/icon.icns",
      // Windows
      "icons/icon.ico",
      // Linux
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      // OHOS — 1024×1024 PNG，文件名以 -foreground / -background 区分前景和背景
      "icons/icon-foreground.png",
      "icons/icon-background.png"
    ]
  }
}
```

**文件名约定**（与 `@2x` 同一思路）：

| 文件名后缀 | 含义 | bundler 识别方式 |
|-----------|------|-----------------|
| `-foreground` | OHOS 自适应图标前景层 | `file_stem.ends_with("-foreground")` |
| `-background` | OHOS 自适应图标背景层 | `file_stem.ends_with("-background")` |
| `-starticon` | OHOS 启动窗口图标 | `file_stem.ends_with("-starticon")` |
| `@2x` | Retina 密度（已有） | `file_stem.ends_with("@2x")` |
| `.icns` | macOS 图标（已有） | `extension == "icns"` |
| `.ico` | Windows 图标（已有） | `extension == "ico"` |

**规则**：
- 前景图（`-foreground`）必须提供
- 背景图（`-background`）必须提供
- `startIcon` 使用 `*-starticon.png` 文件（若无则使用前景图）
- `layered_image.json` 由 bundler 自动生成，无需手动编写

#### 4.3.3 构建时产物

bundler 在 OHOS 构建时读取 `bundle.icon` 中的 `-foreground` / `-background` / `-starticon` 文件，生成以下内容到 OHOS 项目：

```
gen/ohos/AppScope/resources/base/media/
├── layered_image.json      # 模板自带
├── foreground.png          # 从 *-foreground.png 复制
└── background.png          # 从 *-background.png 复制

gen/ohos/entry/src/main/resources/base/media/
├── layered_image.json      # 模板自带
├── foreground.png          # 同上
├── background.png          # 同上
└── startIcon.png           # 从 *-starticon.png 复制（若无则使用 foreground）
```

#### 4.3.4 bundler 处理逻辑

```
构建时遍历 bundle.icon 列表
  │
  ├── 扩展名 .icns → macOS bundler 使用
  ├── 扩展名 .ico  → Windows bundler 使用
  ├── 扩展名 .png 且无 -foreground/-background 后缀 → Linux bundler 使用（按像素尺寸分类）
  │
  └── 扩展名 .png 且含 -foreground / -background 后缀 → OHOS bundler 使用
      │
      ├── foreground.png ← 复制 *-foreground.png
      ├── background.png ← 复制 *-background.png
      ├── startIcon.png  ← 复制 *-starticon.png（若无则使用 foreground）
      └── layered_image.json ← 自动生成
```

**不新增任何 `tauri.conf.json` 配置项，不需要 `tauri icon` manifest**，与其他平台完全一致地通过 `bundle.icon` 文件列表 + 文件名约定来工作。

### 4.4 设备类型适配（修复 G6）

根据 `OHOS_DEVICE_TYPE` 环境变量自动调整 `deviceTypes`：

| OHOS_DEVICE_TYPE | deviceTypes 默认值 | 说明 |
|------------------|-------------------|------|
| `mobile` | `["phone", "tablet"]` | 移动端，不含 2in1 |
| `desktop` | `["2in1"]` | 桌面端，仅 2in1 |
| 未设置 | `["phone", "tablet", "2in1"]` | 通用（默认） |

用户可通过 `bundle.openHarmony.deviceTypes` 显式覆盖。

---

## 5. 实现优先级

| 阶段 | 内容 | 涉及文件 |
|------|------|----------|
| **P0 - 必修** | 修复模板硬编码问题（G2-G4） | `project.rs`, `mod.rs`, `app.json5`, `string.json` 模板 |
| **P0 - 必修** | 实现 OHOS 图标生成（G1） | bundler 读取 `-foreground`/`-background` 文件生成 layered image |
| **P1 - 重要** | 新增 `bundle.openHarmony` 配置节（G5） | `config.schema.json`, `BundleConfig` 结构体 |
| **P2 - 改进** | 设备类型自动适配（G6） | `project.rs` 中根据 `OHOS_DEVICE_TYPE` 调整 |

---

## 6. 完整配置示例

```jsonc
// tauri.conf.json
{
  "productName": "我的应用",
  "version": "2.1.0",
  "identifier": "com.example.myapp",
  "bundle": {
    "active": true,
    "icon": [
      // macOS
      "icons/icon.icns",
      // Windows
      "icons/icon.ico",
      // Linux
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      // OHOS — 文件名约定区分前景/背景/启动图标
      "icons/icon-foreground.png",
      "icons/icon-background.png",
      "icons/icon-starticon.png"
    ],
    "publisher": "Example Inc.",
    "shortDescription": "我的 Tauri 应用",
    "openHarmony": {
      "versionCode": 21,
      "deviceTypes": ["phone", "tablet", "2in1"]
    }
  }
}
```

```jsonc
// tauri.ohos.conf.json（可选，覆盖通用配置）
{
  "bundle": {
    "openHarmony": {
      "versionCode": 22
    }
  }
}
```

---

## 7. 生成的 OHOS 配置文件（预期产物）

### `AppScope/app.json5`
```json5
{
  "app": {
    "bundleName": "com.example.myapp",
    "vendor": "Example Inc.",
    "versionCode": 21,
    "versionName": "2.1.0",
    "icon": "$media:layered_image",
    "label": "$string:app_name"
  }
}
```

### `entry/src/main/module.json5`
```json5
{
  "module": {
    "name": "entry",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": ["phone", "tablet", "2in1"],
    "abilities": [{
      "name": "EntryAbility",
      "icon": "$media:layered_image",
      "label": "$string:EntryAbility_label",
      "startWindowIcon": "$media:startIcon",
      "startWindowBackground": "$color:start_window_background"
    }],
    "requestPermissions": [
      { "name": "ohos.permission.INTERNET" },
      { "name": "ohos.permission.SET_WINDOW_TRANSPARENT" }
    ]
  }
}
```

### `AppScope/resources/base/element/string.json`
```json
{
  "string": [
    { "name": "app_name", "value": "我的应用" }
  ]
}
```

### `entry/src/main/resources/base/element/string.json`
```json
{
  "string": [
    { "name": "module_desc", "value": "我的 Tauri 应用" },
    { "name": "EntryAbility_desc", "value": "我的 Tauri 应用" },
    { "name": "EntryAbility_label", "value": "我的应用" }
  ]
}
```

---

## 8. 兼容性考虑

1. **向后兼容**：所有新增配置项均有默认值，现有项目无需修改 `tauri.conf.json` 即可继续构建
2. **不影响其他平台**：`bundle.openHarmony` 配置节仅 OHOS 构建时读取，其他平台忽略
3. **`tauri ohos init` 重新生成**：模板变更后，已有项目需重新执行 `tauri ohos init` 以更新生成的 OHOS 项目文件（现有 `ensure_init()` 的 bundleName 校验机制会提示用户）
4. **平台特定配置覆盖**：`tauri.ohos.conf.json` 可覆盖 `tauri.conf.json` 中的通用值，便于多平台差异化管理

---

## 9. TODO

### 9.1 设备类型默认值适配 `OHOS_DEVICE_TYPE`

当前 `deviceTypes` 默认值始终为 `["phone", "tablet", "2in1"]`。后续应根据 `OHOS_DEVICE_TYPE` 环境变量自动调整：

| OHOS_DEVICE_TYPE | deviceTypes 默认值 |
|------------------|-------------------|
| `mobile` | `["phone", "tablet"]` |
| `desktop` | `["2in1"]` |
| 未设置 | `["phone", "tablet", "2in1"]` |

### 9.2 build app 适配

当前 OHOS 仅适配了 `build hap`（生成单个 HAP 包），尚未适配 `build app`（生成 APP 包）。APP 包是发布到华为应用市场的最终格式，包含多个 HAP + 签名信息。需要设计：
- `tauri ohos build --target app` 的命令行参数
- APP 包的目录结构和打包流程
- 签名配置与 HAP 签名的关系

### 9.3 不同设备形态的 HAP 打包

当前 OHOS 构建只生成一个通用 HAP。但鸿蒙支持按设备形态（phone/tablet/2in1）分别打包发布。需要设计：
- 如何为不同 `deviceTypes` 生成不同的 HAP
- 是否需要按设备形态拆分 entry module
- 华为应用市场多设备发布流程的对接方式
