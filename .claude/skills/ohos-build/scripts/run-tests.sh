#!/bin/bash
# Tauri OpenHarmony 自动化测试脚本
# 编译(autotest) → 签名安装 → 启动 → 等待 → 拉取报告 → 分析

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Read CLI params BEFORE sourcing env.sh (env.sh sets default which would shadow $2)
DEVICE_SN="${DEVICE_SN:-$1}"
# $2 takes priority over inherited OHOS_DEVICE_TYPE (fixes stale parent shell value)
if [ -n "$2" ]; then
    OHOS_DEVICE_TYPE="$2"
fi
export OHOS_DEVICE_TYPE

source "$SCRIPT_DIR/env.sh"
OHOS_PROJECT="$PROJECT_ROOT/examples/api/src-tauri/gen/ohos"
APP_JSON="$OHOS_PROJECT/AppScope/app.json5"
BUNDLE_NAME=$(grep -o '"bundleName"[[:space:]]*:[[:space:]]*"[^"]*"' "$APP_JSON" | head -1 | sed 's/.*"bundleName"[[:space:]]*:[[:space:]]*"//;s/"//')
REPORT_DEVICE_PATH="/data/app/el2/100/base/$BUNDLE_NAME/cache/test-report.md"
REPORT_LOCAL="$PROJECT_ROOT/examples/api/test-report.md"
REPORT_LOCAL_WIN=$(echo "$REPORT_LOCAL" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')
WAIT_SECONDS="${WAIT_SECONDS:-30}"

echo "=== Tauri OpenHarmony Auto Test ==="
echo "Bundle: $BUNDLE_NAME"
echo "Device: ${DEVICE_SN:-auto-detect}"
echo "Device Type: ${OHOS_DEVICE_TYPE:-desktop}"
echo ""

# Step 0: Rebuild openharmony-ability HAR if sources changed
ABILITY_ROOT="$PROJECT_ROOT/../openharmony-ability"
ABILITY_HAR="$ABILITY_ROOT/ability.har"
OHOS_ENTRY="$OHOS_PROJECT/entry"

if [ -d "$ABILITY_ROOT" ]; then
    ABILITY_CHANGED=false
    if [ ! -f "$ABILITY_HAR" ]; then
        ABILITY_CHANGED=true
    else
        HAR_MTIME=$(stat -c %Y "$ABILITY_HAR" 2>/dev/null || stat -f %m "$ABILITY_HAR" 2>/dev/null || echo 0)
        # Check if any source file is newer than the HAR
        NEWER=$(find "$ABILITY_ROOT/native_ability/src" "$ABILITY_ROOT/crates" -newer "$ABILITY_HAR" -type f 2>/dev/null | head -1)
        if [ -n "$NEWER" ]; then
            ABILITY_CHANGED=true
        fi
    fi

    if [ "$ABILITY_CHANGED" = true ]; then
        echo ">>> Step 0: Rebuilding openharmony-ability HAR..."
        pushd "$ABILITY_ROOT" > /dev/null
        ohrs build --arch arm64 --skip-napi-check 2>&1 | tail -5 || true
        bash scripts/pack.sh 2>&1 | tail -3
        tar -czf ability.har package
        popd > /dev/null
        # Windows EPERM: must fully delete oh_modules before reinstalling
        cmd.exe /c "rmdir /s /q $(echo "$OHOS_PROJECT/oh_modules" | sed 's|^/\(.\)/|\U\1:\\|; s|/|\\|g')" 2>/dev/null || true
        (cd "$OHOS_PROJECT" && ohpm install --all) 2>&1 | tail -3
        echo "    HAR rebuilt and installed."
    else
        echo ">>> Step 0: openharmony-ability HAR is up-to-date, skipping."
    fi
fi

# Step 1: Build with VITE_AUTOTEST=true and OHOS_DEVICE_TYPE
echo ">>> Step 1: Building (autotest mode, device_type=${OHOS_DEVICE_TYPE:-desktop})..."
export VITE_AUTOTEST=true
export OHOS_DEVICE_TYPE="${OHOS_DEVICE_TYPE:-desktop}"
bash "$SCRIPT_DIR/build-ohos.sh"

# Step 2: Sign & Install
echo ""
echo ">>> Step 2: Sign & Install..."
if [ -n "$DEVICE_SN" ]; then
    bash "$SCRIPT_DIR/sign-and-install.sh" "$DEVICE_SN"
else
    bash "$SCRIPT_DIR/sign-and-install.sh"
fi

# Step 3: Wait for tests to complete
echo ""
echo ">>> Step 3: Waiting ${WAIT_SECONDS}s for tests to complete..."
sleep "$WAIT_SECONDS"

# Step 4: Pull report (use cmd.exe to avoid Git Bash path mangling)
echo ""
echo ">>> Step 4: Pulling test report..."
rm -f "$REPORT_LOCAL"

if [ -n "$DEVICE_SN" ]; then
    cmd.exe /c "hdc -t $DEVICE_SN file recv $REPORT_DEVICE_PATH $REPORT_LOCAL_WIN" 2>&1 | tr -d '\r'
else
    cmd.exe /c "hdc file recv $REPORT_DEVICE_PATH $REPORT_LOCAL_WIN" 2>&1 | tr -d '\r'
fi

if [ ! -f "$REPORT_LOCAL" ]; then
    echo "ERROR: Failed to pull test report from device."
    echo "Expected at: $REPORT_DEVICE_PATH"
    echo ""
    echo "Try increasing WAIT_SECONDS (current: $WAIT_SECONDS)"
    echo "Or check if the app started correctly on device."
    exit 1
fi

# Step 5: Analyze report
echo ""
echo "=== Test Report ==="
echo ""

cat "$REPORT_LOCAL"
echo ""

# Count pass/fail from markdown table (uses emoji markers)
PASS_COUNT=$(grep -c '✅' "$REPORT_LOCAL" || true)
FAIL_COUNT=$(grep -c '❌' "$REPORT_LOCAL" || true)
: "${PASS_COUNT:=0}"
: "${FAIL_COUNT:=0}"

echo "=================================================="
echo "Summary: $PASS_COUNT passed, $FAIL_COUNT failed"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo ""
    echo "FAILED tests:"
    grep '❌' "$REPORT_LOCAL" | sed 's/|/  /g'
    exit 1
else
    echo "ALL TESTS PASSED!"
fi
