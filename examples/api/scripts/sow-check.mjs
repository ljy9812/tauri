#!/usr/bin/env node
// sow-check.mjs — SOW 能力清单 × 套件用例账目机检门（Phase A2，plan: sprightly-frolicking-stroustrup）
//
// 用法：
//   node scripts/sow-check.mjs            # 全量校验（exit 0=绿，1=红，2=降级跑非全绿口径）；
//                                        #   默认即严格档（三件套/plan-gap 由 WARN 升 FAIL）
//   node scripts/sow-check.mjs --selftest # 变异红臂：5 个内置变异逐一必须令 verify 报错
//   node scripts/sow-check.mjs --json out.json   # 附带 JSON 账目落盘
//   node scripts/sow-check.mjs --loose    # 宽档（三件套/plan-gap 只 WARN；旧默认档）
//   node scripts/sow-check.mjs --allow-degraded  # 显式接受降级跑（降级档且无 catalog 错误时才 exit 0）
//
// 求值主路径：glob 解析 esbuild（tauri\node_modules\.pnpm\esbuild@*）→ bundle 两个入口
// （src/lib/tests/index.ts 契件 barrel + src/lib/tests/sow-registry.ts 注册表）→ CJS 内存求值。
// esbuild 不可达时降级为正则提取（degraded:true，仅存在性检查；name 重复或数量=0 仍硬 FAIL）。
// 降级档默认 fail-closed：exit 2（≠全绿口径），--allow-degraded 显式 opt-in 才 exit 0。
//
// 判定六档（推导，不手填）：✅=有 std 非 manual 锚 或 有非拒答文档行；⏭contract=条目登记
// contract:'reject'（PASS 判据=平台显式拒绝/契约性 no-op/定义性缺席）；⏭docSkip=文档锚全是
// 拒答行（平台限制 SKIP / 当前无法验证）；🟡=仅 cov；⚠️=仅 manual 锚无文档行；❌=全空；
// plan.kind='excluded' 单列。对账目标 = SOW_BASELINE_1127（1127 诚实分档口径）。

import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const API_DIR = path.resolve(__dirname, '..');
const TAURI_ROOT = path.resolve(API_DIR, '..', '..');
const TESTS_DIR = path.join(API_DIR, 'src', 'lib', 'tests');
const MTD_PATH = path.join(TAURI_ROOT, 'doc', 'manual_tests.md');
const TR_PATH = path.join(API_DIR, 'src', 'views', 'TestRunner.svelte');

const STD_ARRAYS = [
  'coreTests', 'pluginTests', 'dpiTests', 'windowDpiTests', 'imageTests', 'menuTests', 'trayTests',
  'ohosAdapterTests', 'ohosInitTests', 'ohosGapTests', 'ohosMobilePluginTests', 'ohosScreenshotTests',
  'ohosContinuationTests', 'windowOpsTests', 'riskSupplementTests', 'strongholdTests', 'zombieCloseTests'
];
const COV_ARRAYS = [
  'driverTests', 'sideReplayTests', 'badInputTests', 'faultInjectionTests', 'windowOpsExtraTests', 'apiGapTests'
];
const ALL_ARRAYS = [...STD_ARRAYS, ...COV_ARRAYS];

// ─── 1. esbuild 解析（pnpm 版本目录漂移用 glob 兜住） ────────────────────────
export function resolveEsbuild() {
  const req = createRequire(path.join(TAURI_ROOT, 'package.json'));
  try {
    return req.resolve('esbuild');
  } catch { /* fall through to .pnpm glob */ }
  const pnpmDir = path.join(TAURI_ROOT, 'node_modules', '.pnpm');
  if (!fs.existsSync(pnpmDir)) return null;
  const candidates = fs.readdirSync(pnpmDir).filter((d) => /^esbuild@/.test(d));
  if (!candidates.length) return null;
  candidates.sort(); // 取版本字典序最大
  for (let i = candidates.length - 1; i >= 0; i--) {
    const p = path.join(pnpmDir, candidates[i], 'node_modules', 'esbuild', 'package.json');
    if (fs.existsSync(p)) {
      const req2 = createRequire(p);
      return req2.resolve('esbuild');
    }
  }
  return null;
}

export async function bundleEval(entryPath, esbuildPath) {
  const esbuild = createRequire(esbuildPath)(esbuildPath);
  const result = await esbuild.build({
    entryPoints: [entryPath],
    bundle: true,
    format: 'cjs',
    platform: 'browser',
    write: false,
    logLevel: 'silent',
    // import.meta.env 只出现在函数体内（driver-generated.ts:962），CJS 下被 esbuild 摦平，不求值不触发
  });
  const code = result.outputFiles.map((f) => f.text).join('\n');
  const mod = { exports: {} };
  const requireShim = (p) => { throw new Error(`[sow-check] unexpected require('${p}') in bundled output (should be fully bundled)`); };
  new Function('module', 'exports', 'require', code)(mod, mod.exports, requireShim);
  return mod.exports;
}

// ─── 2. 正则降级提取（esbuild 不可达时；仅 names，无 category → 降级账目） ───
// 文件名→数组名的驼峰派生对 4 个文件失配（index.ts 真源导出名与此不同）：
// driver-generated.ts→driverTests、fault-injection-generated.ts→faultInjectionTests、
// plugins.ts→pluginTests、ohos-mobile-plugins.ts→ohosMobilePluginTests。
// 降级档的 array 仅作元数据展示，但错名会让降级账目对不上 cov 数组名，逐个显式映射。
const ARRAY_NAME_OVERRIDES = {
  'driver-generated.ts': 'driverTests',
  'fault-injection-generated.ts': 'faultInjectionTests',
  'plugins.ts': 'pluginTests',
  'ohos-mobile-plugins.ts': 'ohosMobilePluginTests',
};

function regexExtractTests() {
  const catalog = [];
  for (const file of fs.readdirSync(TESTS_DIR).filter((f) => f.endsWith('.ts') && !f.startsWith('sow-registry'))) {
    const text = fs.readFileSync(path.join(TESTS_DIR, file), 'utf8');
    const layer = /driver-generated|fault-injection|window-ops-extra|api-gap/.test(file) ? 'cov' : 'std';
    const arrayName = ARRAY_NAME_OVERRIDES[file]
      ?? file.replace(/\.ts$/, '').replace(/-([a-z])/g, (m, c) => c.toUpperCase()) + 'Tests';
    // 顶格 name: '...'（demo-case-counts 计数坑#3：以此行为准）
    const re = /^ {4}name: '((?:[^'\\]|\\.)*)'/gm;
    let m;
    let n = 0;
    while ((m = re.exec(text)) !== null) {
      catalog.push({ name: m[1].replace(/\\'/g, "'"), category: 'unknown', array: arrayName, layer });
      n++;
    }
    if (n === 0) {
      const re2 = /name: '((?:[^'\\]|\\.)*)'/g;
      while ((m = re2.exec(text)) !== null) {
        catalog.push({ name: m[1].replace(/\\'/g, "'"), category: 'unknown', array: arrayName, layer });
        n++;
      }
    }
  }
  return catalog;
}

// ─── 3. manual_tests.md 解析器 ───────────────────────────────────────────────
// 分节 ^## 中文序号、 + ^### N.N 子节（两者都可被注册表锚引用）；
// 表格按列头名定位（含「用例名称」才是用例表 → 9/10/8 列变体自动兼容，统计小表天然跳过）；
// 分裂前处理 \| 转义；✅ 归一只作用于用例名称/级别两 cell。
// 行 status 分类（review tauri#262 item 11）：行内自带处置的「拒答行」不计入 ✅ 依据——
// 只认两个精确标记（/平台限制\s*SKIP/ 与 /当前无法(人工)?验证/，当前各恰命中 1 行）。
// 注意不能用 /占位/ 之类宽标记：§36.6 NativeIcon PASS 行含「占位符」、§7 Dialog 行含
// 「占位用例」，全是误报。
export function parseManualDoc(text) {
  const lines = text.split(/\r?\n/);
  const rows = [];
  const sectionKeys = [];
  let curKey = null;
  let curSubKey = null; // ### N.N 子节键：传导给其下表格行（锚可引用 §7.1/§36.2 形态）
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const m = /^(#{2,3})\s+(.+)$/.exec(line);
    if (m) {
      const isH2 = m[1] === '##';
      const raw = m[2].trim();
      const key = raw.split(/[、\s（(]/)[0];
      if (isH2) { curKey = key; curSubKey = null; sectionKeys.push({ key, level: 2, title: raw }); }
      else if (curKey) { curSubKey = key; sectionKeys.push({ key, level: 3, title: raw, parent: curKey }); }
      i++;
      continue;
    }
    if (line.startsWith('|')) {
      // 表头候选：拆 cell（先转义 \|）
      const headerCells = splitRow(line);
      const nameIdx = headerCells.findIndex((c) => c.includes('用例名称'));
      if (nameIdx >= 0 && headerCells.length >= 3) {
        const levelIdx = headerCells.findIndex((c) => c.includes('用例级别') || c.includes('级别'));
        let j = i + 1;
        // 跳过分隔行 |---|
        if (j < lines.length && /^\|[\s:|-]+\|?\s*$/.test(lines[j].replace(/\\\|/g, 'X'))) j++;
        while (j < lines.length && lines[j].startsWith('|')) {
          let cells = splitRow(lines[j]);
          if (cells.length !== headerCells.length) {
            // 裸管道容错：L188/L697 两行步骤/预期列含未转义 `|`（#[cfg(any(..)) 等）→
            // 超长合并尾列（名称/级别列不受影响）、短行补尾，不再整行丢弃。
            while (cells.length > headerCells.length) {
              const tail = cells.pop();
              cells[cells.length - 1] += '|' + tail;
            }
            while (cells.length < headerCells.length) cells.push('');
          }
          {
            // ✅ 归一：剥前导/尾随 ✅（§十八~§三十四大量行名尾部带 ✅ 状态标；⏳ 等其他标记保留原样，锚须逐字带）
            const nameCell = (cells[nameIdx] || '').replace(/^✅\s*/, '').replace(/\s*✅\s*$/, '').trim();
            const levelCell = levelIdx >= 0 ? (cells[levelIdx] || '') : '';
            const levels = [...levelCell.matchAll(/T([01])/g)].map((mm) => 'T' + mm[1]);
            // 拒答行分类：整行文本命中精确标记才判 skip（见函数头注释的误报清单）
            const rowText = cells.join(' ');
            const isSkip = /平台限制\s*SKIP/.test(rowText) || /当前无法(人工)?验证/.test(rowText);
            const status = isSkip ? 'skip' : 'pass';
            const rowKey = curSubKey || curKey;
            if (levels.length && nameCell) {
              rows.push({ key: rowKey, name: nameCell, levels, hasMarker: true, lineNo: j + 1, status });
            } else if (nameCell) {
              rows.push({ key: rowKey, name: nameCell, levels: [], hasMarker: false, lineNo: j + 1, status });
            }
          }
          j++;
        }
        i = j;
        continue;
      }
    }
    i++;
  }
  return { rows, sectionKeys };
}

function splitRow(line) {
  return line
    .replace(/\\\|/g, '\x00')
    .split('|')
    .slice(1, -1)
    .map((c) => c.replace(/\x00/g, '|').trim());
}

// ─── 4. TestRunner wrapManual 字面量提取 ─────────────────────────────────────
function extractWrapManual() {
  const text = fs.readFileSync(TR_PATH, 'utf8');
  const names = new Set();
  const re = /wrapManual\(\s*'((?:[^'\\]|\\.)*)'/g;
  let m;
  while ((m = re.exec(text)) !== null) names.add(m[1].replace(/\\'/g, "'"));
  return names;
}

// ─── 5. 核心校验（纯函数：selftest 变异的就是这里的输入） ──────────────────────
export function verify({ registry, expectedCounts, baseline, catalog, docRows, wrapManualNames, strictThreePiece = false }) {
  const errors = [];
  const warnings = [];
  const byName = new Map();
  for (const t of catalog) {
    if (byName.has(t.name)) errors.push(`[catalog] 重复 name: '${t.name}'（${byName.get(t.name)} 与 ${t.array}）`);
    byName.set(t.name, t);
  }
  if (catalog.length === 0) errors.push('[catalog] 求值条目数=0（bundle/求值主路径失败）');

  const docBySec = new Map();
  const docByName = new Map();
  for (const r of docRows) {
    if (!r.hasMarker) continue;
    if (!docBySec.has(r.key)) docBySec.set(r.key, []);
    docBySec.get(r.key).push(r);
    docByName.set(r.key + '|' + r.name, r);
  }

  // 5.1 注册表卫生
  const ids = new Set();
  for (const it of registry) {
    if (ids.has(it.id)) errors.push(`[registry] id 重复: ${it.id}`);
    ids.add(it.id);
    if (!it.name || !it.group) errors.push(`[registry] ${it.id} name/group 缺失`);
    if (it.plan) {
      if (!['auto', 'manual', 'excluded'].includes(it.plan.kind)) errors.push(`[registry] ${it.id} plan.kind 非法: ${it.plan.kind}`);
      if (!['B', 'C1a', 'C1b', 'C2', 'C3', 'C4', 'C5', 'S4'].includes(it.plan.batch)) errors.push(`[registry] ${it.id} plan.batch 非法: ${it.plan.batch}`);
      if (it.plan.kind === 'excluded' && !it.plan.reason) errors.push(`[registry] ${it.id} excluded 缺 reason`);
    }
    for (const a of it.manual || []) {
      if (!/^§[^|]+\|.+/.test(a)) errors.push(`[registry] ${it.id} manual 锚格式非法: ${a}`);
    }
  }
  for (const [sec, want] of Object.entries(expectedCounts)) {
    const got = registry.filter((it) => it.section === sec).length;
    if (got !== want) errors.push(`[registry] §${sec} 条目数 ${got} ≠ SOW 口径 ${want}`);
  }

  // 5.2 引用存在性（转录质量机器关卡）
  for (const it of registry) {
    for (const n of it.std || []) {
      const t = byName.get(n);
      if (!t) errors.push(`[ref] ${it.id} std 引用不存在: '${n}'`);
      else if (t.layer !== 'std') errors.push(`[ref] ${it.id} std 引用落在 cov 层: '${n}'`);
    }
    for (const n of it.cov || []) {
      const t = byName.get(n);
      if (!t) errors.push(`[ref] ${it.id} cov 引用不存在: '${n}'`);
      else if (t.layer !== 'cov') errors.push(`[ref] ${it.id} cov 引用落在 std 层: '${n}'`);
    }
    for (const a of it.manual || []) {
      const key = a.slice(1, a.indexOf('|'));
      const name = a.slice(a.indexOf('|') + 1);
      if (docByName.has(a.slice(1))) continue; // a.slice(1) === '节号|用例名称' 即 docByName 键形态
      if (docBySec.has(key)) {
        // 节存在但行名不匹配 → 先全局找同名行（节号错挂=warn）
        const global = [...docByName.values()].find((r) => r.name === name);
        if (global) warnings.push(`[ref] ${it.id} manual 锚节号错挂: '${a}'（同名行实际在 §${global.key}）`);
        else errors.push(`[ref] ${it.id} manual 引用行不存在: '${a}'`);
      } else {
        errors.push(`[ref] ${it.id} manual 引用节不存在: §${key}（行: '${name}'）`);
      }
    }
  }

  // 5.3 六档推导 + 账目（review tauri#262 item 11：ok 拆出两支诚实档——
  //   contract=条目登记 contract:'reject'（PASS 判据=平台显式拒绝/契约性 no-op/定义性缺席），
  //   docSkip=文档锚全是拒答行。判据不变，变的是「拒答也折算 ✅」的注水口径被拆出来单列。
  //   两档均输出 warning 行（常显，不静默），且不触发 [orphan]/[missing]——它们自带处置。）
  const tally = { ok: 0, contract: 0, docSkip: 0, covOnly: 0, manualOnly: 0, missing: 0, excluded: 0 };
  const perSection = {};
  const orphanCandidates = [];
  for (const it of registry) {
    const stdRefs = (it.std || []).map((n) => byName.get(n)).filter(Boolean);
    const hasRunnableStd = stdRefs.some((t) => t.category !== 'manual');
    const hasManualAnchor = stdRefs.some((t) => t.category === 'manual');
    // 文档行存在性（三件套第三腿用）与「通过行/拒答行」分档分开看
    const docRowsFor = (it.manual || []).map((a) => docByName.get(a.slice(1))).filter(Boolean);
    const hasDoc = docRowsFor.length > 0;
    const hasDocPass = docRowsFor.some((r) => r.status !== 'skip');
    const hasDocSkipOnly = hasDoc && docRowsFor.every((r) => r.status === 'skip');
    const hasCov = (it.cov || []).some((n) => byName.get(n)?.layer === 'cov');
    const isExcluded = it.plan?.kind === 'excluded';
    const isContract = it.contract === 'reject';
    let verdict;
    if (isExcluded) verdict = 'excluded';
    else if (isContract) verdict = 'contract';
    else if (hasRunnableStd || hasDocPass) verdict = 'ok';
    else if (hasDocSkipOnly) verdict = 'docSkip';
    else if (hasCov) verdict = 'covOnly';
    else if (hasManualAnchor) verdict = 'manualOnly';
    else verdict = 'missing';
    tally[verdict]++;
    perSection[it.section] = perSection[it.section] || { ok: 0, contract: 0, docSkip: 0, covOnly: 0, manualOnly: 0, missing: 0, excluded: 0 };
    perSection[it.section][verdict]++;
    if (verdict === 'docSkip') {
      warnings.push(`[doc-skip] ${it.id}（${it.name}）⏭ 文档行为拒答口径（平台限制 SKIP / 当前无法验证），不折算 ✅`);
    }
    if (verdict === 'contract') {
      warnings.push(`[contract-reject] ${it.id}（${it.name}）⏭ contract:'reject' 登记：PASS 判据=平台显式拒绝/契约性 no-op/定义性缺席`);
    }
    if (verdict === 'manualOnly' && !it.plan) {
      orphanCandidates.push(it.id);
      errors.push(`[orphan] ⚠️ 条目无处置: ${it.id}（${it.group}/${it.name}）——std manual 锚无文档行且未登记 plan`);
    }
    if (verdict === 'missing' && !it.plan) {
      errors.push(`[missing] ❌ 条目无处置: ${it.id}（${it.group}/${it.name}）`);
    }
    // 🟡 未登记处置批次 = Phase B/C 待办清单（首跑 WARN，--final 升 ERROR）
    if (verdict === 'covOnly' && !it.plan) {
      const msg = `[plan-gap] ${it.id}（${it.name}）🟡 仅 cov 且未登记 plan 批次`;
      if (strictThreePiece) errors.push(msg); else warnings.push(msg);
    }
    // 5.4 三件套完整性（plan.kind='manual'：std 锚 + TR 按钮 + 文档行）
    if (it.plan?.kind === 'manual') {
      const legs = [];
      if (!(it.std || []).length) legs.push('std锚');
      const stdAnchorInTr = (it.std || []).some((n) => wrapManualNames.has(n));
      if (!stdAnchorInTr) legs.push('TR按钮');
      if (!hasDoc) legs.push('文档行');
      if (legs.length) {
        const msg = `[three-piece] ${it.id}（${it.name}）缺 ${legs.join('+')}`;
        if (strictThreePiece) errors.push(msg); else warnings.push(msg);
      }
    }
  }

  // 5.5 基线对账（冻结基线=504 口径：全量 514 − §四 10 条流程级豁免；差额逐条归因——禁止改判据凑数）
  const s4Excluded = registry.filter((it) => it.section === '四' && it.plan?.kind === 'excluded').length;
  const tallyNonS4 = { ok: tally.ok, contract: tally.contract, docSkip: tally.docSkip, covOnly: tally.covOnly, manualOnly: tally.manualOnly, missing: tally.missing, excluded: tally.excluded - s4Excluded };
  for (const k of ['ok', 'contract', 'docSkip', 'covOnly', 'manualOnly', 'missing', 'excluded']) {
    if (tallyNonS4[k] !== baseline[k]) {
      errors.push(`[baseline] ${k} = ${tallyNonS4[k]}（504 口径；全量含 §四 ${tally[k]}）≠ 冻结基线 ${baseline[k]}`);
    }
  }

  // 5.6 反向孤儿（文档行未被注册表引用 → 信息级）
  const referencedRows = new Set();
  for (const it of registry) {
    for (const a of it.manual || []) {
      const key = a.slice(1, a.indexOf('|'));
      const name = a.slice(a.indexOf('|') + 1);
      const r = docByName.get(key + '|' + name);
      if (r) referencedRows.add(r);
    }
  }
  const unreferenced = docRows.filter((r) => r.hasMarker && !referencedRows.has(r));
  if (unreferenced.length) {
    warnings.push(`[reverse-orphan] ${unreferenced.length} 条文档行未被注册表引用（按钮驱动/系统交互域=预期，首条: §${unreferenced[0].key} ${unreferenced[0].name}）`);
  }

  return { errors, warnings, tally, perSection, byName, docBySec };
}

// ─── 6. P0-2 计数对账（std 实跑口径 / cov 分项 / 文档行口普查） ───────────────
function p02Counts(catalog, docRows, mtdText) {
  const out = { errors: [], info: [] };
  const std = catalog.filter((t) => t.layer === 'std');
  const cov = catalog.filter((t) => t.layer === 'cov');
  const manualMarks = std.filter((t) => t.category === 'manual').length;
  out.info.push(`std: ${std.length} 条目 / ${manualMarks} manual 标记 / 实跑口径 ${std.length - manualMarks}`);
  for (const a of COV_ARRAYS) {
    out.info.push(`cov ${a}: ${cov.filter((t) => t.array === a).length}`);
  }
  out.info.push(`cov 合计: ${cov.length}`);
  // 文档行口普查：解析行数 vs 原文 T 标记行数（229 纯行 / 231 标记口径含 2 双标记行）
  const markerLines = mtdText.split(/\r?\n/).filter((l) => /\*\*T[01]\*\*/.test(l)).length;
  const parsed = docRows.filter((r) => r.hasMarker).length;
  const parsedAll = docRows.length;
  out.info.push(`MTD: 解析 ${parsed} 条 T 标记行（含无标记候选行 ${parsedAll - parsed}）/ 原文 T 标记行 ${markerLines}`);
  if (markerLines !== parsed) out.errors.push(`[p0-2] MTD 行口普查不一致: 原文 T 标记行 ${markerLines} ≠ 解析 ${parsed}（先逐条归因，禁止硬凑）`);
  const noMarker = docRows.filter((r) => !r.hasMarker);
  if (noMarker.length) out.info.push(`无 T 标记候选行: ${noMarker.map((r) => `§${r.key}:${r.name}`).slice(0, 12).join('；')}${noMarker.length > 12 ? ' …' : ''}`);
  return out;
}

// ─── 7. 变异红臂（--selftest：5 个变异逐一必须令 verify 报错） ─────────────────
function mutate(registry, kind) {
  const reg = structuredClone(registry);
  if (kind === 'drop-docrow') {
    // 优先挑「仅文档行承载」的条目（无 std/cov）——删掉 manual 后翻 ❌ missing，门必红；
    // 若注册表里不存在此类条目则退化为任一带 manual 的条目（此时依赖基线对账臂翻红）。
    const it = reg.find((x) => (x.manual || []).length > 0 && !(x.cov || []).length && !(x.std || []).length)
      || reg.find((x) => (x.manual || []).length > 0);
    it.manual = [];
    return { desc: '删一条文档行映射', registry: reg };
  }
  if (kind === 'corrupt-std') {
    const it = reg.find((x) => (x.std || []).length > 0);
    it.std = [it.std[0] + '_CORRUPTED'];
    return { desc: '改坏一个 std 锚', registry: reg };
  }
  if (kind === 'dup-id') {
    // 动态取一对相邻且 id 不同的条目，不再硬编码下标（注册表缩到 <2 条或相邻
    // id 全同时显式抛错——main().catch 也会接住标红，但 TypeError 的诊断信息
    // 指不出「注册表缩水」这个真实原因）。
    const idx = reg.findIndex((x, i) => i > 0 && x.id !== reg[i - 1].id);
    if (idx <= 0) throw new Error('dup-id mutation needs >=2 registry entries with distinct adjacent ids');
    reg[idx].id = reg[idx - 1].id;
    return { desc: '复制 id', registry: reg };
  }
  if (kind === 'null-reason') {
    const it = reg.find((x) => x.plan?.kind === 'excluded');
    it.plan.reason = undefined;
    return { desc: '置空 excluded reason', registry: reg };
  }
  if (kind === 'make-orphan') {
    // 挑「manual+std、无 cov、非 contract 登记」的条目：变异后（去 plan+去文档行+
    // std 换 manual 类锚）必落 ⚠️ manualOnly 且无 plan → [orphan] 臂翻红。旧版裸挑
    // 第一个 manual+std 条目（S1-001，带 cov）——变异后先落 [plan-gap]，[orphan]
    // 分支从未被真正走到（review tauri#262 item 13）。
    const it = reg.find((x) => (x.manual || []).length > 0 && x.std?.length && !(x.cov || []).length && x.contract !== 'reject')
      || reg.find((x) => (x.manual || []).length > 0 && x.std?.length);
    it.manual = [];
    it.plan = undefined;
    it.std = it.std.map(() => 'n/a');
    // 直接构造 manual 锚条目：std 引用指向 catalog 中 category='manual' 的条目
    return { desc: '制造孤儿锚（去 plan+去文档行）', registry: reg, needManualAnchor: it.id };
  }
  throw new Error('unknown mutation ' + kind);
}

// ─── main ────────────────────────────────────────────────────────────────────
export async function main() {
const args = process.argv.slice(2);
const SELFTEST = args.includes('--selftest');
// 严格档默认开（review tauri#262 item 13：原 --final 旗标仓库里无任何 script/workflow
// 调用——门是否生效取决于跑的人记不记得带旗标，等于没设门。现默认严格；--loose 显式
// 退回旧宽档（三件套/plan-gap 只 WARN）。--final 保留为兼容别名（默认即其行为）。
const FINAL = !args.includes('--loose');
const ALLOW_DEGRADED = args.includes('--allow-degraded');
const jsonIdx = args.indexOf('--json');
const jsonEq = args.find((a) => /^--json=/.test(a));
const jsonOut = jsonEq
  ? jsonEq.slice('--json='.length)
  : jsonIdx >= 0 && args[jsonIdx + 1] && !args[jsonIdx + 1].startsWith('--') ? args[jsonIdx + 1] : null;

let degraded = false;
let catalog;
let registryModule;
try {
  const esbuildPath = resolveEsbuild();
  if (!esbuildPath) throw new Error('esbuild 不可达');
  const [testsExports, regExports] = await Promise.all([
    bundleEval(path.join(TESTS_DIR, 'index.ts'), esbuildPath),
    bundleEval(path.join(TESTS_DIR, 'sow-registry.ts'), esbuildPath)
  ]);
  registryModule = regExports;
  catalog = [];
  for (const arr of ALL_ARRAYS) {
    const list = testsExports[arr];
    if (!Array.isArray(list)) throw new Error(`bundle 求值缺数组 ${arr}（exports keys: ${Object.keys(testsExports).join(',')}）`);
    for (const t of list) catalog.push({ name: t.name, category: t.category, array: arr, layer: STD_ARRAYS.includes(arr) ? 'std' : 'cov' });
  }
} catch (e) {
  console.error(`[sow-check] DEGRADED 主路径失败，降级正则模式: ${e.message}`);
  degraded = true;
  catalog = regexExtractTests();
  // 降级下注册表用正则抽不出（TS 字面量嵌套结构复杂）→ 只做 catalog 自身完整性
  const dupCheck = verify({ registry: [], expectedCounts: {}, baseline: { ok: 0, covOnly: 0, manualOnly: 0, missing: 0, excluded: 0 }, catalog, docRows: [], wrapManualNames: new Set() });
  // 结构化降级信号（稳定键值行；错误按 [catalog] 结构前缀计数，不耦合错误文案）
  const catalogErrors = dupCheck.errors.filter((x) => x.startsWith('[catalog]'));
  console.log(`DEGRADED=true catalog=${catalog.length} catalogErrors=${catalogErrors.length}`);
  for (const err of catalogErrors) console.log('  ' + err);
  if (SELFTEST) {
    // 变异红臂必须走 bundle 主路径，降级档根本没跑——不接受 --allow-degraded 假绿
    console.error('[sow-check] --selftest 需要主路径（bundle 求值），降级档无法执行变异红臂 → exit 1');
    process.exit(1);
  }
  if (catalog.length === 0 || catalogErrors.length > 0) process.exit(1);
  if (ALLOW_DEGRADED) {
    console.log('DEGRADED accepted via --allow-degraded → exit 0（仅 catalog 存在性检查；注册表/引用/基线/四档均未校验）');
    process.exit(0);
  }
  console.error('[sow-check] 降级跑非全绿口径（仅 catalog 存在性检查；注册表/引用/基线/四档均未校验）→ exit 2。确需接受降级跑请加 --allow-degraded。');
  process.exit(2);
}

const mtdText = fs.readFileSync(MTD_PATH, 'utf8');
const { rows: docRows } = parseManualDoc(mtdText);
const wrapManualNames = extractWrapManual();

const input = {
  registry: registryModule.sowRegistry,
  expectedCounts: registryModule.SOW_EXPECTED_COUNTS,
  // 基线 1127（review tauri#262 item 11）：ok 拆 contract/docSkip 两支诚实档后的口径；
  // 1109 基线在注册表里继续导出留档（历史账目，判据口径演进记录）。
  baseline: registryModule.SOW_BASELINE_1127,
  catalog,
  docRows,
  wrapManualNames,
  strictThreePiece: FINAL
};

if (SELFTEST) {
  // 变异红臂：对「registry 输入」做 5 个变异，逐一必须令 verify 产生 ≥1 error
  const mutations = ['drop-docrow', 'corrupt-std', 'dup-id', 'null-reason', 'make-orphan'];
  let pass = 0;
  for (const kind of mutations) {
    const m = mutate(input.registry, kind);
    let reg = m.registry;
    if (m.needManualAnchor) {
      // make-orphan：把该条目 std 改为指向 catalog 中真实存在的 manual 类别条目（孤儿锚条件）
      const anchor = catalog.find((t) => t.layer === 'std' && t.category === 'manual');
      if (anchor) {
        const it = reg.find((x) => x.id === m.needManualAnchor);
        it.std = [anchor.name];
      }
    }
    const r = verify({ ...input, registry: reg, strictThreePiece: true });
    if (r.errors.length > 0) { pass++; console.log(`  [selftest] PASS  ${m.desc} → ${r.errors.length} 条错误（首条: ${r.errors[0]}）`); }
    else console.log(`  [selftest] FAIL  ${m.desc} → 零错误（门没红！）`);
  }
  console.log(`--selftest ${pass}/${mutations.length}`);
  process.exit(pass === mutations.length ? 0 : 1);
}

const result = verify(input);
const p02 = p02Counts(catalog, docRows, mtdText);
result.errors.push(...p02.errors);

console.log('═'.repeat(72));
console.log(`SOW 账目机检（degraded=${degraded}，final=${FINAL}）`);
console.log('═'.repeat(72));
for (const line of p02.info) console.log('  ' + line);
console.log('  注册表条目: ' + input.registry.length + `（${Object.entries(input.expectedCounts).map(([k, v]) => `§${k}=${input.registry.filter((x) => x.section === k).length}/${v}`).join(' ')}）`);
console.log(`  六档: ✅${result.tally.ok} ⏭contract ${result.tally.contract} ⏭docSkip ${result.tally.docSkip} 🟡${result.tally.covOnly} ⚠️${result.tally.manualOnly} ❌${result.tally.missing} excluded ${result.tally.excluded}`);
for (const [sec, t] of Object.entries(result.perSection)) {
  console.log(`    §${sec}: ✅${t.ok} ⏭c${t.contract} ⏭d${t.docSkip} 🟡${t.covOnly} ⚠️${t.manualOnly} ❌${t.missing} excluded${t.excluded}`);
}
console.log('  wrapManual 按钮字面量: ' + wrapManualNames.size + ' 处');
if (result.warnings.length) {
  console.log(`── WARNINGS (${result.warnings.length}) ──`);
  for (const w of result.warnings) console.log('  ' + w);
}
if (result.errors.length) {
  console.log(`── ERRORS (${result.errors.length}) ──`);
  for (const e of result.errors.slice(0, 60)) console.log('  ' + e);
  if (result.errors.length > 60) console.log(`  …另有 ${result.errors.length - 60} 条`);
}
console.log('═'.repeat(72));
console.log(result.errors.length ? `✗ FAIL（${result.errors.length} 条）` : '✓ PASS');

if (jsonOut) {
  const payload = { degraded, final: FINAL, tally: result.tally, perSection: result.perSection, errors: result.errors, warnings: result.warnings, p02: p02.info, catalogSize: catalog.length, docRows: docRows.filter((r) => r.hasMarker).length };
  fs.writeFileSync(jsonOut, JSON.stringify(payload, null, 2), 'utf8');
  console.log(`JSON 账目 → ${jsonOut}`);
}
process.exit(result.errors.length ? 1 : 0);
}

// 直接运行守卫：被 import 时不执行 main（parseManualDoc/verify 可被单测/探针复用）
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => { console.error('[sow-check] fatal:', e); process.exit(1); });
}
