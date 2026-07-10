# OHOS HAP 签名与部署 — tauri-cli 内置方案

## 概述

将 OHOS HAP 签名和设备部署从外部脚本迁移到 `tauri ohos` CLI 内置命令。签名采用 **环境变量注入** 方式（对标 iOS），使用 DevEco SDK 的 **hap-sign-tool.jar** 进行独立签名，不修改 `build-profile.json5`。

## 命令

### `tauri ohos build`

构建 HAP 并自动签名（环境变量控制）。

```
tauri ohos build [OPTIONS]
```

产物目录：`gen/ohos/entry/build/default/outputs/default/`

### `tauri ohos run`

构建 + 签名 + 安装到设备 + 启动应用。

```
tauri ohos run [OPTIONS]
```

| 选项 | 说明 |
|------|------|
| `--release` | release 模式 |
| `-f, --features` | cargo features |
| `-c, --config` | 合并配置 |
| `--device <name>` | 指定目标设备 |
| `--device-type` | mobile / desktop |

## 签名环境变量

效仿 iOS 的 `IOS_CERTIFICATE` / `IOS_MOBILE_PROVISION` 模式，签名材料通过环境变量注入：

| 变量名 | 必需 | 说明 |
|--------|------|------|
| `OHOS_KEYSTORE_FILE` | ✅ | .p12 密钥库文件路径 |
| `OHOS_KEYSTORE_PASSWORD` | ✅ | 密钥库密码 |
| `OHOS_KEY_ALIAS` | ✅ | 密钥别名 |
| `OHOS_KEY_PASSWORD` | ✅ | 密钥密码 |
| `OHOS_APP_CERT_FILE` | ✅ | .cer 应用证书路径 |
| `OHOS_PROFILE_FILE` | ✅ | .p7b Provisioning Profile 路径 |
| `OHOS_SIGN_ALG` | ❌ | 签名算法，默认 `SHA256withECDSA` |

### 行为规则

- **全部设置** → 使用 hap-sign-tool.jar 独立签名
- **部分设置** → 警告并跳过，列出缺失的变量
- **全部未设置** → 跳过签名，使用构建产出的 HAP

## 签名流程

```
tauri ohos build / run
       │
       ▼
┌─────────────────────────────┐
│ 1. 构建 HAP                  │  → entry-default-unsigned.hap
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 2. 读取环境变量               │
│    全部存在？                  │
│    YES → hap-sign-tool.jar   │  → entry-default-signed.hap
│    NO  → 跳过签名             │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 3. 输出构建产物               │
│    Finished 2 HAPs at:       │
│    - entry-default-unsigned  │
│    - entry-default-signed    │
└─────────────────────────────┘
```

## hap-sign-tool.jar

签名工具位于 OHOS SDK 的 `toolchains/lib/hap-sign-tool.jar`。

自动查找路径：
1. `env.toolchains_path()/lib/hap-sign-tool.jar`（cargo-mobile2 环境）
2. `$OHOS_SDK_HOME/toolchains/lib/hap-sign-tool.jar`

签名命令：
```
java -jar hap-sign-tool.jar sign-app \
  -keyAlias <OHOS_KEY_ALIAS> \
  -signAlg <OHOS_SIGN_ALG> \
  -mode localSign \
  -appCertFile <OHOS_APP_CERT_FILE> \
  -profileFile <OHOS_PROFILE_FILE> \
  -inFile entry-default-unsigned.hap \
  -keystoreFile <OHOS_KEYSTORE_FILE> \
  -outFile entry-default-signed.hap \
  -keyPwd <OHOS_KEY_PASSWORD> \
  -keystorePwd <OHOS_KEYSTORE_PASSWORD>
```

- 成功时静默（不输出 Java 日志）
- 失败时输出 stdout/stderr 供调试

### 安全说明

`keyPwd` 和 `keystorePwd` 作为命令行参数传递给 `hap-sign-tool.jar`，在进程存活期间可通过 `ps` 或 `/proc/<pid>/cmdline` 查看。

这与 tauri 中 iOS/macOS 签名的处理方式一致 — Apple 的 `security import -P <password>` 同样将证书密码作为 CLI 参数传递，无更安全替代方案。`hap-sign-tool.jar` 不支持从 stdin、环境变量或配置文件读取密码。

**实际风险**：
- 签名进程存活时间短（通常 < 5 秒），暴露窗口有限
- CI 环境中进程隔离，其他用户无法访问
- 本地开发通常为单用户机器

**建议**：CI 环境中使用 secrets 管理密码，避免明文写入仓库。但需注意：无论密码来源如何，运行时仍会以 CLI 参数传递给 Java 进程，这是 `hap-sign-tool.jar` 的限制，无更安全的替代方式。

## 设备部署（run 命令）

| 步骤 | 命令 | 说明 |
|------|------|------|
| 安装 | `hdc -t <id> install -r <hap>` | 覆盖安装 |
| 失败重试 | `hdc -t <id> shell bm uninstall -n <bundle>` + 重安装 | fallback |
| 启动 | `hdc -t <id> shell aa start -b <bundle> -a EntryAbility` | — |

- 成功时 hdc 输出静默
- 失败时输出 hdc 日志供调试

## 各平台签名对比

| | Android | iOS | OHOS |
|---|---------|-----|------|
| 签名方式 | Gradle signingConfigs | 环境变量 | 环境变量 |
| 签名工具 | Gradle 内置 | codesign / xcrun | hap-sign-tool.jar |
| 环境变量前缀 | N/A | `IOS_` | `OHOS_` |
| 签名时机 | 构建时 | 构建后 export | 构建后独立签名 |
| CI 友好 | ❌ 需改 Gradle | ✅ | ✅ |

## 各平台构建产物目录

| | Android | iOS | OHOS |
|---|---------|-----|------|
| 根目录 | `gen/android` | `gen/apple` | `gen/ohos` |
| 产物路径 | `app/build/outputs/apk/{arch}/{profile}/` | `build/{arch}/` | `entry/build/default/outputs/default/` |
| 格式 | `.apk` / `.aab` | `.ipa` | `.hap` |

## CI/CD 示例

```bash
# 设置签名环境变量（从 CI secrets）
export OHOS_KEYSTORE_FILE="./signing/release.p12"
export OHOS_KEYSTORE_PASSWORD="${{ secrets.OHOS_KEYSTORE_PWD }}"
export OHOS_KEY_ALIAS="release"
export OHOS_KEY_PASSWORD="${{ secrets.OHOS_KEY_PWD }}"
export OHOS_APP_CERT_FILE="./signing/release.cer"
export OHOS_PROFILE_FILE="./signing/release.p7b"

# 构建 + 签名
cargo tauri ohos build --release
```

## 本地开发示例（PowerShell）

```powershell
# ohos-signing.ps1
$SignDir = "C:\myproject\signing"
$env:OHOS_KEYSTORE_FILE     = "$SignDir\debug.p12"
$env:OHOS_KEYSTORE_PASSWORD = "password"
$env:OHOS_KEY_ALIAS         = "debug"
$env:OHOS_KEY_PASSWORD      = "password"
$env:OHOS_APP_CERT_FILE     = "$SignDir\debug.cer"
$env:OHOS_PROFILE_FILE      = "$SignDir\debug.p7b"

# 使用
. .\ohos-signing.ps1
cargo tauri ohos build     # 构建 + 签名
cargo tauri ohos run       # 构建 + 签名 + 安装 + 启动
```

## 实现文件

| 文件 | 说明 |
|------|------|
| `crates/tauri-cli/src/mobile/open_harmony/signing.rs` | 环境变量读取 + hap-sign-tool.jar 签名 |
| `crates/tauri-cli/src/mobile/open_harmony/run.rs` | `tauri ohos run` 子命令 |
| `crates/tauri-cli/src/mobile/open_harmony/build.rs` | build 命令集成签名 |
| `crates/tauri-cli/src/mobile/open_harmony/mod.rs` | 模块注册 |
