#!/bin/bash
# Tauri OpenHarmony Build Script
# 编译 Rust + 前端，生成已签名 HAP（hvigorw 使用 build-profile.json5 中的证书自动签名）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"

API_DIR="$PROJECT_ROOT/examples/api"
SRC_TAURI="$API_DIR/src-tauri"
OHOS_PROJECT="$SRC_TAURI/gen/ohos"
if [ "$OHOS_DEVICE_TYPE" = "desktop" ]; then
    ENTRY_DIR="entry_desktop"
else
    ENTRY_DIR="entry_mobile"
fi
SIGNED_HAP="$OHOS_PROJECT/$ENTRY_DIR/build/default/outputs/default/$ENTRY_DIR-default-signed.hap"
SO_FILE="$PROJECT_ROOT/target/aarch64-unknown-linux-ohos/release/libapi_lib.so"
HVIGORFILE="$OHOS_PROJECT/$ENTRY_DIR/hvigorfile.ts"

echo "=== Tauri OpenHarmony Build ==="
echo "DEVECO_HOME=$DEVECO_HOME"
echo "PROJECT_ROOT=$PROJECT_ROOT"
echo "OHOS_DEVICE_TYPE=$OHOS_DEVICE_TYPE"
echo ""

# ─── Step 0: Detect template changes and re-run `tauri ohos init` ───
TEMPLATE_DIR="$PROJECT_ROOT/crates/tauri-cli/templates/mobile/open-harmony"
ENTRY_ETS="$OHOS_PROJECT/$ENTRY_DIR/src/main/ets/entryability/EntryAbility.ets"
NEED_INIT=false

if [ ! -f "$ENTRY_ETS" ]; then
    NEED_INIT=true
    echo ">>> Step 0: gen/ohos project not found, will run tauri ohos init."
elif [ -d "$TEMPLATE_DIR" ]; then
    # Check if any template file is newer than the generated EntryAbility.ets
    ETS_MTIME=$(stat -c %Y "$ENTRY_ETS" 2>/dev/null || stat -f %m "$ENTRY_ETS" 2>/dev/null || echo 0)
    NEWER_TEMPLATE=$(find "$TEMPLATE_DIR" -newer "$ENTRY_ETS" -type f 2>/dev/null | head -1)
    if [ -n "$NEWER_TEMPLATE" ]; then
        NEED_INIT=true
        echo ">>> Step 0: Template files changed (e.g. $NEWER_TEMPLATE), will re-run tauri ohos init."
    fi
fi

if [ "$NEED_INIT" = true ]; then
    echo "    Running tauri ohos init to regenerate gen/ohos project..."
    (cd "$SRC_TAURI" && cargo run --manifest-path "$PROJECT_ROOT/crates/tauri-cli/Cargo.toml" -- ohos init --skip-targets-install --ci 2>&1) || {
        echo "ERROR: tauri ohos init failed"
        exit 1
    }
    echo "    gen/ohos project regenerated."
else
    echo ">>> Step 0: gen/ohos project is up-to-date with templates, skipping init."
fi

# ─── Step 1: 安装前端依赖 ───
if [ ! -d "$API_DIR/node_modules" ]; then
    echo ""
    echo ">>> Step 1: Installing frontend dependencies..."
    (cd "$API_DIR" && pnpm install)
fi

# ─── Step 2: 构建 @tauri-apps/api ───
if [ ! -d "$PROJECT_ROOT/packages/api/dist" ] || [ ! -f "$PROJECT_ROOT/crates/tauri/scripts/bundle.global.js" ]; then
    echo ""
    echo ">>> Step 2: Building @tauri-apps/api..."
    (cd "$PROJECT_ROOT" && pnpm build:api)
fi

# ─── Step 2.5: 构建插件 dist-js（防止过期产物导致测试失败）───
# plugins-workspace is at the project root level (parent of tauri repo)
PLUGINS_DIR="$(dirname "$PROJECT_ROOT")/plugins-workspace"
if [ -d "$PLUGINS_DIR" ]; then
    echo ""
    echo ">>> Step 2.5: Building plugins dist-js..."
    (cd "$PLUGINS_DIR" && pnpm build 2>&1 | tail -3) || echo "    WARNING: plugins build failed, using existing dist-js"
fi

# ─── Step 3: 前端构建 ───
echo ""
echo ">>> Step 3: Building frontend (VITE_AUTOTEST=${VITE_AUTOTEST:-false})..."
export VITE_AUTOTEST="${VITE_AUTOTEST:-false}"
(cd "$API_DIR" && pnpm build)

# ─── Step 3.5: ACL permission consistency check ───
echo ""
echo ">>> Step 3.5: Checking ACL permission consistency..."
bash "$SCRIPT_DIR/../../acl-check/scripts/clean-stale-acl.sh" "$SRC_TAURI"

# ─── Step 4: Rust 编译 ───
echo ""
echo ">>> Step 4: Compiling Rust (aarch64-unknown-linux-ohos release, device_type=$OHOS_DEVICE_TYPE)..."
rm -f "$SO_FILE"
(cd "$SRC_TAURI" && OHOS_DEVICE_TYPE="$OHOS_DEVICE_TYPE" cargo build --target aarch64-unknown-linux-ohos --release --features prod)

if [ ! -f "$SO_FILE" ]; then
    echo "ERROR: Rust compilation failed - .so not found"
    exit 1
fi
echo "    Generated: $SO_FILE"

# ─── Step 5: 拷贝 .so 到 ohos 项目 ───
echo ""
echo ">>> Step 5: Copying .so to ohos project..."
mkdir -p "$OHOS_PROJECT/$ENTRY_DIR/libs/arm64-v8a"
cp "$SO_FILE" "$OHOS_PROJECT/$ENTRY_DIR/libs/arm64-v8a/libapi_lib.so"

# ─── Step 6: hvigorw 打包（自动禁用/恢复 tauriPlugin）───
echo ""
echo ">>> Step 6: Running hvigorw assembleHap..."

# 禁用 tauriPlugin（独立构建时不需要 TCP 回调 tauri CLI）
if grep -q 'plugins:\[tauriPlugin()\]' "$HVIGORFILE"; then
    sed -i 's/plugins:\[tauriPlugin()\]/plugins:[]/' "$HVIGORFILE"
    RESTORE_PLUGIN=true
else
    RESTORE_PLUGIN=false
fi

rm -f "$SIGNED_HAP"
(cd "$OHOS_PROJECT" && hvigorw --no-daemon -p product=default -p module=$ENTRY_DIR@default assembleHap --analyze=normal --parallel --incremental) || HVIGOR_EXIT=$?
HVIGOR_EXIT=${HVIGOR_EXIT:-0}

# 恢复 tauriPlugin
if [ "$RESTORE_PLUGIN" = true ]; then
    sed -i 's/plugins:\[\]/plugins:[tauriPlugin()]/' "$HVIGORFILE"
fi

if [ $HVIGOR_EXIT -ne 0 ]; then
    echo "ERROR: hvigorw assembleHap failed"
    exit 1
fi

# ─── 验证产物 ───
if [ ! -f "$SIGNED_HAP" ]; then
    echo "ERROR: Build failed - signed HAP not found at:"
    echo "  $SIGNED_HAP"
    exit 1
fi

echo ""
echo "=== Build Complete ==="
echo "HAP: $SIGNED_HAP"
