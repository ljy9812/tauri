---
name: tauri-ohos-code-review
description: Tauri OHOS 代码检视。使用场景：(1) committer 审查 GitHub PR，(2) submit skill 在 push 前检视本地 commit。
---

# Tauri OHOS 代码检视

本技能引导完成代码检视流程：获取 diff → 多轮代码审计 → 提交/输出 findings → 编译部署 → 清理。

> **适用场景**：
> - **PR 检视**：committer 收到 PR 链接，需要系统性审查 OHOS 适配代码质量
> - **本地 commit 检视**：submit skill 在 push 前检视本地 commit（由 submit skill 调用）

> **前提条件**：
> - PR 检视：已安装并认证 `gh` CLI，本地已有所有仓库代码
> - 本地 commit 检视：本地已有 commit（尚未 push）

## 状态追踪

使用 Claude TaskList 追踪每个 Step 的执行状态。

### Guard: 启动时初始化

**每次 skill 被调用时，首先检查 TaskList**：
- 如果 TaskList 非空 → 找到当前 `in_progress` 的 task，从该 step 继续
- 如果 TaskList 为空 → 立即创建以下 task（不可跳过）：

```
TaskCreate: "Step 1: 获取 diff 来源"
TaskCreate: "Step 2: 代码检视（多轮迭代）"
TaskCreate: "Step 3: 提交/输出 findings"
TaskCreate: "Step 4: 编译部署"
TaskCreate: "Step 5: 清理"
TaskCreate: "Step 6: Checklist 演进 — 提取通用规则"
```

创建后 TaskUpdate 第一个为 `in_progress`，开始执行。

## 步骤

### Step 1: 获取 diff 来源

根据检视对象获取 diff 和变更文件列表。

#### PR 检视

1. **解析 PR 链接**：从用户输入提取 PR 信息（完整 URL / `tauri#25` 简写 / 多个 PR）

2. **检查 gh CLI**：`gh auth status`，未认证则提示用户 `gh auth login`

3. **获取 PR 元信息**：
   ```bash
   gh pr view <N> --repo Eulogizethesun/<repo> --json title,body,headRefName,files
   ```

4. **解析关联 PR**：检查 PR body 是否包含其他仓的 PR 链接，发现未列出的关联 PR 则提示用户补充

5. **Checkout PR 分支**：对每个仓库：
   ```bash
   cd D:\workspace\tauri\<repo>
   git stash -u                                    # 保存 uncommitted 改动
   git fetch origin pull/<N>/head:review/pr-<N>    # fetch PR 分支
   git checkout review/pr-<N>                       # checkout
   ```
   记录原始分支名（通常 `ohdev`），用于 Step 5 清理。

6. **获取 diff**：
   ```bash
   gh pr diff <N> --repo Eulogizethesun/<repo>
   ```

#### 本地 commit 检视

1. **确定 base branch**：默认 `upstream/ohdev`（如 upstream 不存在则 fallback 到 `origin/ohdev`），也可由调用方指定

2. **获取 diff**：
   ```bash
   git diff <base-branch>...HEAD
   ```
   包含本地所有未推送的 commit 变更。无需 checkout，已在目标分支。

#### 输出

两种模式都输出相同的 diff 内容，供 Step 2 使用：
```
## Diff Source
✅ tauri#25: review/pr-25 (12 files changed)
✅ tao#8: review/pr-8 (3 files changed)
```
或：
```
## Diff Source
✅ openharmony-ability: upstream/ohdev...HEAD (8 files changed)
✅ tray-icon: upstream/ohdev...HEAD (1 file changed)
```

### Step 2: 代码检视（多轮迭代）

检视分为多轮，每轮侧重不同，**直到连续 2 轮无新发现为止**（loop-until-dry）。两种模式执行相同的检视逻辑。

#### 2a. Round 1: Diff 扫描 + Checklist 快速检查

**目标**：快速扫描 diff，发现明显违规。

1. 按文件分组扫描：
   - **代码文件**：`.rs` / `.ets` / `.ts` / `Cargo.toml`（A-G 类检查）
   - **仓库配置文件**：`.gitattributes` / `.gitignore` / `.env.local` / `.env`（H 类检查）
   - **文档/openspec**：`openspec/` / `doc/` 下的文件（H3/H5/H6 检查）

2. 对照 `references/review-checklist.md` 逐项快速检查：
   - A: cfg 隔离 — OHOS 代码是否有正确的 cfg gate
   - B: 平台隔离 — 其他平台代码是否受影响
   - C: NAPI/TSFN — callee_handled、FnArgs、camelCase
   - D: 线程模型 — 阻塞模式、Mutex 跨越
   - E: ArkTS 框架 — @Builder、onLoadIntercept
   - F: openharmony-ability 桥接 — 是否唯一桥接
   - G: 代码质量 — unwrap、硬编码、注释语言
   - H: 仓库级规范 — 不应提交的文件、gitattributes、openspec 归档、注释语言、手动用例归档

3. 记录 Round 1 findings。

#### 2b. Round 2: 源码深读 + Openspec 对照（使用 Subagent 并行）

**目标**：阅读变更文件的完整源码（不是 diff），对照 openspec 设计文档验证实现完整性。

**执行方式**：使用 `Agent` 工具并行派发 subagent，每个 subagent 负责一个文件的深度审查。

1. **先读取 openspec 文档**（如果涉及 tauri 仓）：
   ```bash
   ls openspec/changes/
   cat openspec/changes/<change-name>/proposal.md
   cat openspec/changes/<change-name>/design.md
   cat openspec/changes/<change-name>/tasks.md
   cat openspec/changes/<change-name>/specs/<capability>/spec.md  # 如有
   ```

2. **派发 Subagent 并行深读源码**：

   对 diff 中修改的每个关键文件，派发一个 subagent 做深度审查：

   ```
   Agent("深度审查 <file_path>"):
     - Read 完整文件源码
     - 理解上下文：函数调用链、模块边界、cfg gate 组合
     - 检查 diff 未修改但相关的代码（是否需要同步更新）
     - 对照 openspec design.md 检查该文件的实现是否完整
     - 输出该文件的 findings 列表
   ```

   **Subagent prompt 模板**：
   ```
   深度审查文件 `<file_path>`，完整阅读源码后检查：
   1. 该文件的 OHOS 相关代码是否有完整的 cfg gate
   2. 函数/方法的错误处理是否完整（无 unwrap、无 callback 丢失）
   3. 是否有 diff 未修改但需要同步更新的关联代码
   4. 对照 openspec 中 <feature_name> 的设计，该文件的实现是否完整
   5. 线程安全：Mutex/Arc 使用是否合理

   输出格式：每个 finding 列出 file:line, severity, category, description, suggestion。
   如果没有发现新问题，输出 "No new findings"。
   ```

3. **Openspec 合规性审计**（主 agent 执行）：
   - 逐条核对 design.md 中定义的每个功能点是否在代码中实现
   - 逐条核对 spec.md 中定义的每个 requirement 是否被满足
   - 检查 tasks.md 中 `[x]` 标记的任务是否真正完成
   - 未实现的需求 → 🟡 Major [Spec合规] Requirement X not implemented
   - 设计与实现不一致 → 🟡 Major [Spec合规] Design-implementation mismatch

4. **跨仓一致性检查**（多仓场景）：
   - wry 层 API 与 tauri 层调用方是否匹配（参数类型、错误处理）
   - openharmony-ability 的 NAPI 接口与 Rust 侧调用是否一致
   - 新增的公共 API 在所有仓中签名是否对齐

5. **仓库级检查（仅 tauri 仓）**：
   - `git diff <base-branch> -- doc/manual_tests.md`：是否新增了与功能对应的手动用例（H5）
   - `git diff <base-branch> --name-only -- openspec/changes/`：是否归档了对应的设计文档（H6）

6. 记录 Round 2 findings（排除与 Round 1 重复的）。

#### 2c. Round 3+: 专项深挖

**目标**：针对前两轮发现的模式进行定向深挖。

根据前两轮 findings 的模式，选择以下专项检查：

- **错误路径分析**：如果发现 callback 丢失问题，全面扫描所有异步回调路径
- **线程安全分析**：如果发现锁竞争问题，全面检查所有 Mutex/Arc 使用
- **API 兼容性分析**：如果发现 API 签名不一致，全面比对所有跨仓接口
- **cfg 覆盖分析**：如果发现遗漏的 cfg gate，用 grep 扫描所有 OHOS 代码路径

每轮仅保留与 Round 1/2 不重复的新 findings。

#### 2d. Loop-until-dry 退出条件

```
Round N findings 与 Round N-1 findings 去重比较：
  - 如果 Round N 有 0 个新 finding → dry_count++
  - 如果 Round N 有 ≥1 个新 finding → dry_count = 0

退出条件：dry_count >= 2（连续 2 轮无新发现）
最大轮次：5（防止无限循环）
```

每轮结束输出进度：
```
## Review Progress
Round 1 (Diff 扫描): 5 findings (1 🔴, 2 🟡, 2 🔵)
Round 2 (源码深读): 3 new findings (0 🔴, 2 🟡, 1 🔵)
Round 3 (专项深挖): 1 new finding (0 🔴, 0 🟡, 1 🔵)
Round 4 (专项深挖): 0 new findings → dry_count = 1
Round 5 (专项深挖): 0 new findings → dry_count = 2 → EXIT
Total: 9 unique findings
Adversarial Verify: 9 findings → 7 survived, 2 refuted
```

#### 2e. 生成最终 Findings

合并所有轮次的 findings，去重后按仓库分组。每个 finding 包含：

```
Finding 结构:
  repo: <仓库名>
  file: <文件路径>
  line: <行号>
  severity: 🔴 Blocker / 🟡 Major / 🔵 Minor / ℹ️ Suggestion
  category: OHOS约束 / Spec合规 / 平台隔离 / 代码质量 / 测试回归 / 仓库规范
  description: <问题描述>
  suggestion: <修复建议>
  round: <发现轮次，标注来源>
```

输出最终审计进度：
```
## Audit Complete (4 rounds)
✅ tauri#25: 5 findings (1 Blocker, 2 Major, 2 Minor)
✅ tao#8: 2 findings (0 Blocker, 1 Major, 1 Minor)
```

#### 2f. 对抗性自检 (Adversarial Self-Verify)

**目的**：减少 false positive，提升检视质量。对每个 finding 派发独立 subagent 尝试反驳，只有通过验证的 finding 才会进入 Step 3。误报会损害审查者信任。

**执行方式**：对 2e 产出的每个 finding，使用 `Agent` 工具派发一个质疑者 subagent：

```
Agent("尝试反驳 finding"):
  prompt: |
    你是一个代码检视质疑者。以下是 Claude 对 PR 的检视发现：

    ## Finding
    - 描述: <description>
    - 文件: <file>:<line>
    - 严重级别: <severity>
    - 分类: <category>

    ## 你的任务
    1. 阅读该文件的完整源码（Read 工具）
    2. 阅读 PR diff（gh pr diff）
    3. 尝试证明这个 finding 是误报：
       - 代码是否真的有问题？还是审查者理解有误？
       - 是否有上下文信息（注释、文档、调用方）使其合理？
       - 建议的修复是否可行？会不会引入新问题？

    4. 给出判断：
       - refuted: true — 误报，理由: ...
       - refuted: false — 确认真阳性，理由: ...

    ## 偏向规则
    如果不确定，默认 refuted: true。宁可漏掉一个真阳性，也不提交一个误报。
```

**过滤规则**：
- `refuted: true` → 丢弃该 finding（误报）
- `refuted: false` → 保留该 finding（确认真阳性）
- 不确定 → 默认丢弃

**可并行执行**：多个 finding 的质疑者 subagent 可以同时派发（互不依赖）。

输出：
```
## Adversarial Verify
tauri#25: 5 findings → 3 survived, 2 refuted
  Refuted: F3 (cfg gate already in parent module), F5 (Mutex::lock unwrap allowed per G2)
tao#8: 2 findings → 2 survived, 0 refuted
```

过滤后的 findings 列表传给 Step 3 提交。

### Step 3: 提交/输出 findings

#### PR 检视：提交到 GitHub

**使用 `gh api` 而非 `gh pr review`**，因为 `gh pr review --body` 只能提交总结评论，无法标注到具体代码行。

1. **获取 Head Commit SHA**：
   ```bash
   gh pr view <N> --repo Eulogizethesun/<repo> --json headRefOid --jq '.headRefOid'
   ```

2. **判断 review 类型**：
   - 有 🔴 Blocker → `event: "REQUEST_CHANGES"`
   - 无 Blocker → `event: "COMMENT"`

3. **提交 API 调用**（每个 PR 独立一个调用）：
   ```bash
   gh api repos/Eulogizethesun/<repo>/pulls/<N>/reviews \
     -X POST \
     --input - <<'ENDJSON'
   {
     "commit_id": "<head_commit_sha>",
     "event": "COMMENT",
     "body": "## OHOS Code Review — <repo>#<N>\n\n| 🔴 | 🟡 | 🔵 | ℹ️ |\n|---|---|---|---|\n| 0 | 2 | 1 | 1 |\n\n详细 inline comments 见下方各文件标注。",
     "comments": [
       {
         "path": "crates/tauri/src/ohos_plugin.rs",
         "line": 79,
         "side": "RIGHT",
         "body": "🟡 **[NAPI]** `unwrap()` 在序列化失败时会 panic。Fix: ..."
       }
     ]
   }
   ENDJSON
   ```

4. **行号定位**：`comments[].line` 是文件中的实际行号，不是 diff 行号。

5. **输出提交结果**：
   ```
   ## Review Submitted
   ✅ tauri#25: https://github.com/.../pull/25#pullrequestreview-xxx (4 inline comments)
   ✅ tao#8: https://github.com/.../pull/8#pullrequestreview-xxx (1 inline comment)
   ```

> 报告格式详见 `references/review-report-template.md`
> API 用法详见 `references/github-review-api.md`

#### 本地 commit 检视：输出到终端

**跳过 GitHub 提交**。将 findings 直接输出到终端，返回给调用方（submit skill）：

```
## Local Review Complete (3 rounds)
openharmony-ability: 5 findings (1 🔴 Blocker, 2 🟡 Major, 1 🔵 Minor, 1 ℹ️ Info)
tray-icon: 0 findings (clean)
```

调用方根据 findings 严重级别决定是否修复并 amend commit。

### Step 4: 编译部署

> **本地 commit 检视模式**：跳过此步骤（编译部署由 submit skill 自行决定）。

调用 `ohos-build` skill 执行完整编译+部署+autotest。

#### 4a. Source 环境

```bash
source D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/env.sh
```

#### 4b. 运行构建+测试

```bash
bash D:/workspace/tauri/tauri/.claude/skills/ohos-build/scripts/run-tests.sh "" desktop
```

脚本自动完成：
1. 检测 `openharmony-ability/` 源码变更，自动重建 HAR 包
2. 前端构建（pnpm + vite，VITE_AUTOTEST=true）
3. Rust 交叉编译（aarch64-unknown-linux-ohos，release，--features prod）
4. 拷贝 .so → hvigorw assembleHap（自动签名）
5. 卸载旧版 → 安装 HAP → 启动
6. 等待 30s → 拉取 test-report.md → 分析结果

#### 4c. 解析测试结果

读取 `examples/api/test-report.md`，提取：
- 总测试数 / 通过 / 失败 / 跳过
- 失败的测试名称和错误信息

如果有测试失败，生成 findings：
- 新增失败 → 🟡 Major [测试回归] New test failure: <test_name>
- 回归（之前通过现在失败）→ 🔴 Blocker [测试回归] Regression: <test_name>

#### 4d. 处理构建失败

如果 ohos-build 脚本报错（编译失败、签名失败等）：
- 标记为 🔴 Blocker [代码质量] Build failed: <错误摘要>
- 继续执行 Step 5（findings 仍有效）

输出：
```
## Build & Test Complete
✅ Build: success
✅ Autotest: 42/42 passed, 0 failed
✅ No regressions
```

### Step 5: 清理（需用户确认）

> **本地 commit 检视模式**：跳过此步骤（未 checkout 分支，无需清理）。

**在开始清理前，先输出当前状态，然后用 AskUserQuestion 询问用户是否要清理：**

```
## 当前状态（清理前）
tauri → review/pr-25
tao → review/pr-8
wry → review/pr-12
```

> **注意**：不要自动执行清理。等待用户确认后再执行以下 5a/5b/5c。如果用户选择跳过清理，直接跳到最终汇总输出。

#### 5a. 切回原始分支并删除 review 分支

对每个涉及的仓库：

```bash
cd D:\workspace\tauri\<repo>
git checkout ohdev           # 切回原始分支
git stash pop                # 恢复 uncommitted 改动（如有）
git branch -D review/pr-<N>  # 删除临时 review 分支
```

#### 5b. 清理本地改动

恢复因 review 产生的本地修改（如 build 产物、test-report 等）：

```bash
git checkout -- <被修改的文件>  # 恢复被修改的文件
git clean -f <untracked 文件>   # 清理 untracked 文件
```

#### 5c. 验证清理结果

对每个仓库执行 `git status --short`，确认无 dirty files。

输出：
```
## Cleanup Complete
✅ tauri → ohdev (clean)
✅ tao → ohdev (stash restored)
✅ wry → ohdev (clean)
```

### 最终汇总（Step 5 完成后输出）

```
## Code Review Complete

Reviewed PRs:
  - tauri#25: 1 Blocker, 2 Major (Request Changes)
  - tao#8: 0 Blocker, 1 Major (Comment)
  - wry#12: Clean (Comment)

Build: ✅ Success
Autotest: ✅ 42/42 passed

All reviews submitted to GitHub.
```

### Step 6: Checklist 演进 — 提取通用规则

检视完成后，审视本次产生的所有 findings（含被对抗性验证 refuted 的），判断是否有可提取的通用规则应补充到 checklist。

#### 6a. 审视 findings

回顾本次检视的全部 findings，逐条评估：

- 这个 finding 对应的问题是否是**反复出现的模式**？（同类问题在多个文件/多个 PR 中出现）
- 这个 finding 对应的检查项是否**已经被 checklist 覆盖**？
- 如果未覆盖，是否可以抽象为一个**通用的检查规则**？

**适合提取为 checklist 项的特征**：
- 同类问题在本次检视中出现 ≥2 次
- 该问题属于 OHOS 适配的特有陷阱，开发者容易忽略
- 可以用一句简洁的规则描述

**不适合提取的情况**：
- 过于具体的代码问题（仅某个函数特有的 bug）
- 已经被现有 checklist 项覆盖（检查是否遗漏而非规则缺失）

#### 6b. 更新 checklist

如果有新的通用规则，读取当前 checklist：

```bash
cat D:\workspace\tauri\tauri\.claude\skills\tauri-ohos-code-review\references\review-checklist.md
```

按以下格式追加新项到对应分类下（A-H）：

```markdown
- [ ] <分类编号><序号>: <规则描述>
```

示例：
```markdown
## G — 代码质量

- [ ] G5: OHOS 平台的 `log` 宏使用 `hilog` 而非 `println!`（`println!` 在 OHOS 上无输出）
```

如果新规则不属于任何现有分类，可新增分类（如 `## I — xxx`）。

#### 6c. Commit + Push + PR

如果 checklist 有变更：

```bash
cd D:\workspace\tauri\tauri

# 确认 remote 配置
git remote -v
# origin = 用户 fork, upstream = Eulogizethesun/tauri

# 确保在 ohdev 分支且是最新
git checkout ohdev
git fetch upstream
git rebase upstream/ohdev

# 提交变更
git add .claude/skills/tauri-ohos-code-review/references/review-checklist.md
git commit -m "chore(review): update review checklist with new items from PR review"

# Push 到用户 fork
git push origin ohdev

# 创建 PR 到 upstream
gh pr create \
  --repo Eulogizethesun/tauri \
  --base ohdev \
  --head <your-username>:ohdev \
  --title "chore(review): update review checklist with new items" \
  --body "## Changes\n- 新增 checklist 项，来源于本次 PR 检视中发现的通用模式\n\n## New Items\n- <列出新增的 checklist 项>"
```

#### 6d. 输出

```
## Step 6: Checklist Evolution
New items added: 2
  - G5: OHOS log 宏使用 hilog 而非 println!
  - H8: OHOS 权限声明需在 module.json5 的 requestPermissions 中

PR created: https://github.com/Eulogizethesun/tauri/pull/XX
```

如果没有新增项：
```
## Step 6: Checklist Evolution
No new items — all findings covered by existing checklist.
```

## 参考文档

- [检视清单](references/review-checklist.md) — OHOS 约束检查清单
- [报告模板](references/review-report-template.md) — 检视报告格式模板
- [GitHub Review API](references/github-review-api.md) — gh api + inline comments 用法

## 错误处理

| 错误场景 | 处理方式 |
|---------|---------|
| gh CLI 未安装 | Step 1 检测，提示用户安装 |
| gh 未认证 | Step 1 检测，提示用户 `gh auth login` |
| PR 不存在或已关闭 | 标记为 "❌ PR Not Found"，跳过该仓库 |
| git stash 失败 | 提示用户手动 commit 或清理 |
| git checkout 冲突 | 提示用户手动解决 |
| 编译失败 | 标记为 🔴 Blocker，Review 已在 Step 3 提交，继续到 Step 5 清理 |
| 测试超时 | 标记为 🟡 Major，继续到 Step 5 清理 |
| gh api review 失败 | 提示用户手动提交，保留报告内容；常见原因：pending review 冲突、PR 已合并 |
