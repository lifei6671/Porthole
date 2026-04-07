# Tauri Windows 端口转发工具 MVP 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个基于 Tauri 的 Windows 桌面端口转发工具 MVP，支持 TCP/UDP、IPv4/IPv6、规则持久化、单条/全局启停、日志展示，并使用 `gost.exe` 作为 sidecar 执行真实转发。

**Architecture:** 前端采用 Tauri + Web UI，Rust 负责规则持久化、校验、配置渲染、`gost.exe` 生命周期管理与事件推送，`gost.exe` 负责真实端口转发。运行时以单 sidecar 进程为主，规则变更通过“更新运行集合 -> 重渲染 `gost.yaml` -> 受控重载”实现。

**Tech Stack:** Tauri v2、Rust、Serde、TOML、YAML、TypeScript、前端状态管理、`gost v3.x` sidecar

---

## 0. 使用说明

### 0.1 文档用途

这份文档是 **执行计划 + 验收清单 + 进度记录**。

- 开发时按任务顺序执行
- 每完成一个任务，必须回写本文档
- 每个任务都有明确的验证方式
- 未通过验证，不得勾选完成

### 0.2 完成状态规则

任务状态只允许使用：

- `- [ ]` 未完成
- `- [x]` 已完成

### 0.3 完成后回写规则

每完成一个任务，必须同步更新以下 3 处：

1. 将对应任务标题下的 `任务状态` 从 `未完成` 改为 `已完成`
2. 勾选该任务下所有已完成步骤
3. 在文末 **执行记录** 区追加一条记录，格式如下：

```markdown
- 2026-04-07 Task N 完成
  - 验证：`命令或手工验证项`
  - 结果：`PASS / FAIL`
  - 说明：`关键变更摘要`
```

### 0.4 总体验收门槛

全部任务完成后，至少需要满足：

- `cargo test` 通过
- 前端测试通过
- 基础构建通过
- Windows 手工验证矩阵中 MVP 关键场景通过
- 本文档所有任务均已回写为完成

---

## 1. 文件结构规划

以下是推荐的目标目录结构，用于指导后续任务拆分。

```text
/
├── docs/
│   ├── 2026-04-07-tauri-windows-port-forwarding-design.md
│   └── 2026-04-07-tauri-windows-port-forwarding-implementation-plan.md
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── app-toolbar.tsx
│   │   ├── rule-list.tsx
│   │   ├── rule-dialog.tsx
│   │   ├── status-bar.tsx
│   │   └── log-panel.tsx
│   ├── hooks/
│   │   ├── use-rules.ts
│   │   └── use-runtime-events.ts
│   ├── lib/
│   │   ├── api.ts
│   │   ├── types.ts
│   │   └── validators.ts
│   └── styles/
│       └── app.css
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── binaries/
    │   └── gost-x86_64-pc-windows-msvc.exe
    └── src/
        ├── main.rs
        ├── app_state.rs
        ├── commands/
        │   ├── mod.rs
        │   ├── rules.rs
        │   └── runtime.rs
        ├── model/
        │   ├── mod.rs
        │   ├── rule.rs
        │   └── runtime.rs
        ├── service/
        │   ├── mod.rs
        │   ├── rule_store.rs
        │   ├── validator.rs
        │   ├── gost_renderer.rs
        │   ├── gost_process.rs
        │   ├── runtime_events.rs
        │   └── firewall_notice.rs
        └── support/
            ├── paths.rs
            ├── job_object.rs
            └── pid_file.rs
```

---

## 2. 任务总览

### 2.1 进度勾选区

- [x] Task 1 完成：项目脚手架与 Sidecar 打包
- [x] Task 2 完成：Rust 领域模型与持久化
- [x] Task 3 完成：校验器与 `gost.yaml` 渲染
- [ ] Task 4 完成：`gost` 进程管理与运行态
- [ ] Task 5 完成：Tauri Command 与事件推送
- [ ] Task 6 完成：前端骨架与规则列表
- [ ] Task 7 完成：规则编辑、日志与状态 UI
- [ ] Task 8 完成：端到端验证与文档收尾

### 2.2 任务表

| Task | 名称 | 目标 | 验证方式 |
|------|------|------|----------|
| 1 | 项目脚手架与 Sidecar 打包 | 建立 Tauri 工程和 `gost` sidecar 基础 | 构建成功、sidecar 可被发现 |
| 2 | Rust 领域模型与持久化 | 实现规则模型、TOML 读写、路径管理 | 单元测试通过 |
| 3 | 校验器与 `gost.yaml` 渲染 | 完成规则校验与 TCP/UDP 配置输出 | 渲染测试通过 |
| 4 | `gost` 进程管理与运行态 | 完成 Job Object、PID 清理、API 探活、重载 | 集成测试通过 |
| 5 | Tauri Command 与事件推送 | 打通前后端命令和状态事件 | 命令测试 + 手工联通验证 |
| 6 | 前端骨架与规则列表 | 实现单窗口 UI 骨架与规则列表 | 前端测试 + 手工验证 |
| 7 | 规则编辑、日志与状态 UI | 完成规则弹窗、日志区、状态条 | 手工操作链路通过 |
| 8 | 端到端验证与文档收尾 | 完成 MVP 验收矩阵和文档回写规范 | 验证矩阵通过 |

---

## 3. 任务明细

### Task 1: 项目脚手架与 Sidecar 打包

**任务状态：** `- [x] 已完成`

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles/app.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/binaries/gost-x86_64-pc-windows-msvc.exe`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [x] Step 1: 初始化 Tauri v2 前端与 Rust 工程骨架，确保项目根目录直接作为应用仓库，不额外再套子目录
- [x] Step 2: 在 `src-tauri/tauri.conf.json` 中配置 `bundle.externalBin = ["binaries/gost"]`
- [x] Step 3: 将 Windows sidecar 文件按目标三元组命名放入 `src-tauri/binaries/`
- [x] Step 4: 建立最小可运行前端页面，页面先只显示应用标题和空白内容区域
- [x] Step 5: 建立最小可运行 Rust 入口，能成功启动 Tauri 窗口
- [x] Step 6: 执行构建验证

**Run:**

```bash
npm install
npm run tauri build
```

**Expected:**

- 依赖安装成功
- Tauri 能找到 sidecar 二进制
- 构建流程不因 `externalBin` 配置报错

- [x] Step 7: 回写本计划文档，标记 Task 1 已完成并记录构建结果

**Verification Criteria:**

- `npm run tauri build` 通过
- 打包日志中不出现 sidecar 文件缺失错误
- 应用启动后能正常打开窗口

---

### Task 2: Rust 领域模型与持久化

**任务状态：** `- [x] 已完成`

**Files:**
- Create: `src-tauri/src/app_state.rs`
- Create: `src-tauri/src/model/mod.rs`
- Create: `src-tauri/src/model/rule.rs`
- Create: `src-tauri/src/model/runtime.rs`
- Create: `src-tauri/src/service/mod.rs`
- Create: `src-tauri/src/service/rule_store.rs`
- Create: `src-tauri/src/support/paths.rs`
- Create: `src-tauri/tests/rule_store_tests.rs`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [x] Step 1: 定义规则领域模型，字段至少包含 `id`、`name`、`enabled`、`protocol`、`listen_host`、`listen_port`、`target_host`、`target_port`、`remark`、`created_at`、`updated_at`
- [x] Step 2: 定义运行态模型，至少包含进程状态、规则运行状态、最近错误摘要、当前运行集合
- [x] Step 3: 实现应用数据目录路径管理，明确 `rules.toml`、`gost.yaml`、PID 文件的保存路径
- [x] Step 4: 实现 `RuleStore` 的读取、原子写入、默认配置初始化逻辑
- [x] Step 5: 为 `RuleStore` 增加并发写保护，使用 `Mutex` 或统一写入口，避免 Tauri 并发命令导致交叉覆盖
- [x] Step 6: 编写 TOML 读写测试，覆盖空文件初始化、正常保存、重新加载、时间字段序列化
- [x] Step 7: 执行 Rust 单元测试

**Run:**

```bash
cd src-tauri
cargo test --test rule_store_tests -- --nocapture
```

**Expected:**

- `rules.toml` 可正常生成和读取
- 并发写保护逻辑通过测试

- [x] Step 8: 回写本计划文档，标记 Task 2 已完成并记录测试结果

**Verification Criteria:**

- `cargo test --test rule_store_tests -- --nocapture` 通过
- 新建默认配置后，文件结构与设计文档一致
- 同一进程内多次快速保存不会产生损坏文件

---

### Task 3: 校验器与 `gost.yaml` 渲染

**任务状态：** `- [x] 已完成`

**Files:**
- Create: `src-tauri/src/service/validator.rs`
- Create: `src-tauri/src/service/gost_renderer.rs`
- Create: `src-tauri/tests/validator_tests.rs`
- Create: `src-tauri/tests/gost_renderer_tests.rs`
- Modify: `src-tauri/src/model/rule.rs`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [x] Step 1: 实现地址格式化逻辑，确保 IPv4 输出 `host:port`，IPv6 输出 `[host]:port`
- [x] Step 2: 实现规则保存前校验，覆盖协议合法性、端口范围、ID 唯一性、监听冲突检测
- [x] Step 3: 实现启动前校验，覆盖运行集合为空、sidecar 缺失、渲染失败、监听冲突等场景
- [x] Step 4: 实现 `gost.yaml` 渲染器，支持 TCP 和 UDP 两类 service 输出
- [x] Step 5: UDP 渲染默认不写高级 metadata 字段，显式依赖 `gost` 默认值
- [x] Step 6: 编写渲染测试，覆盖 `IPv4 -> IPv4`、`IPv6 -> IPv6`、`IPv4 -> IPv6`、`IPv6 -> IPv4`
- [x] Step 7: 执行 Rust 单元测试

**Run:**

```bash
cd src-tauri
cargo test --test validator_tests -- --nocapture
cargo test --test gost_renderer_tests -- --nocapture
```

**Expected:**

- 生成的 `gost.yaml` 与设计约束一致
- TCP / UDP 规则都能输出合法配置

- [x] Step 8: 回写本计划文档，标记 Task 3 已完成并记录测试结果

**Verification Criteria:**

- 地址格式化测试通过
- 冲突校验测试通过
- 渲染结果中 IPv6 地址方括号正确
- UDP 配置未无意写入高级 metadata

---

### Task 4: `gost` 进程管理与运行态

**任务状态：** `- [ ] 未完成`

> 当前切片仅落地 `support/job_object.rs` 与 `support/pid_file.rs`，供后续
> `gost_process.rs` 直接接入；本任务整体仍保持未完成。

**Files:**
- Create: `src-tauri/src/service/gost_process.rs`
- Create: `src-tauri/src/support/job_object.rs`
- Create: `src-tauri/src/support/pid_file.rs`
- Create: `src-tauri/tests/gost_process_tests.rs`
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/support/paths.rs`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [ ] Step 1: 实现 `gost.exe` 启动、停止、重载基础逻辑
- [ ] Step 2: 启动后立即绑定 Windows Job Object，确保父进程退出时 sidecar 自动终止
- [ ] Step 3: 实现 PID 文件写入与启动前残留清理兜底逻辑
- [ ] Step 4: 启动 `gost` 时同时开启本地回环 API 服务，仅监听 `127.0.0.1` 或 `[::1]`
- [ ] Step 5: 实现 API 探活启动检测，采用 `100ms` 轮询、`2s` 超时
- [ ] Step 6: 实现 stdout / stderr 读取与运行态更新
- [ ] Step 7: 实现“运行集合变更 -> 互斥重载”流程，确保快速连续点击不会把状态打乱
- [ ] Step 8: 编写进程管理测试，至少覆盖“子进程退出更新状态”“PID 文件清理”“API 不可达判定启动失败”
- [ ] Step 9: 执行 Rust 测试

**Run:**

```bash
cd src-tauri
cargo test --test gost_process_tests -- --nocapture
```

**Expected:**

- 进程管理测试通过
- 异常退出能被正确识别
- 重载路径可串行执行

- [ ] Step 10: 回写本计划文档，标记 Task 4 已完成并记录测试结果

**Verification Criteria:**

- Job Object 绑定逻辑可工作
- API 探活可区分“进程活着但未就绪”和“已成功启动”
- PID 文件不会长期残留无效状态

---

### Task 5: Tauri Command 与事件推送

**任务状态：** `- [ ] 未完成`

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/rules.rs`
- Create: `src-tauri/src/commands/runtime.rs`
- Create: `src-tauri/src/service/runtime_events.rs`
- Create: `src-tauri/tests/commands_tests.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/app_state.rs`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [ ] Step 1: 暴露 `list_rules`、`create_rule`、`update_rule`、`delete_rule`
- [ ] Step 2: 暴露 `start_rule`、`stop_rule`、`start_all_enabled_rules`、`stop_all_rules`
- [ ] Step 3: 暴露 `get_runtime_status`、`clear_logs`
- [ ] Step 4: 实现事件推送器，至少覆盖“规则状态变化”“进程异常退出”“日志追加”
- [ ] Step 5: 前端初始化时通过 `get_runtime_status()` 拉一次快照，后续主要依赖事件更新
- [ ] Step 6: 编写命令层测试，验证状态变更后能推送正确事件
- [ ] Step 7: 执行 Rust 测试

**Run:**

```bash
cd src-tauri
cargo test --test commands_tests -- --nocapture
```

**Expected:**

- 命令调用成功
- 事件推送包含正确的运行态快照

- [ ] Step 8: 回写本计划文档，标记 Task 5 已完成并记录测试结果

**Verification Criteria:**

- 新建、修改、删除规则命令可成功返回
- 单条启停与全局启停命令能触发状态事件
- `gost` 异常退出时前端可在事件流中及时感知

---

### Task 6: 前端骨架与规则列表

**任务状态：** `- [ ] 未完成`

**Files:**
- Create: `src/components/app-toolbar.tsx`
- Create: `src/components/rule-list.tsx`
- Create: `src/components/status-bar.tsx`
- Create: `src/hooks/use-rules.ts`
- Create: `src/lib/api.ts`
- Create: `src/lib/types.ts`
- Create: `src/styles/app.css`
- Create: `src/__tests__/rule-list.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/main.tsx`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [ ] Step 1: 搭建页面三段式布局，包含顶部工具栏、规则列表、底部状态栏/日志预留区
- [ ] Step 2: 通过 `list_rules()` 拉取规则并在列表展示字段：名称、协议、监听、目标、默认启用、运行状态、操作
- [ ] Step 3: 实现顶部工具栏按钮：新增规则、启动全部、停止全部、刷新状态
- [ ] Step 4: 实现列表级空状态与错误状态展示
- [ ] Step 5: 编写前端测试，验证列表渲染和按钮交互
- [ ] Step 6: 执行前端测试

**Run:**

```bash
npm run test -- rule-list
```

**Expected:**

- 列表测试通过
- 页面可渲染空规则与已有规则两类状态

- [ ] Step 7: 回写本计划文档，标记 Task 6 已完成并记录测试结果

**Verification Criteria:**

- 页面加载后能显示规则列表
- 顶部工具栏按钮可触发对应命令
- 规则状态文案与后端枚举一致

---

### Task 7: 规则编辑、日志与状态 UI

**任务状态：** `- [ ] 未完成`

**Files:**
- Create: `src/components/rule-dialog.tsx`
- Create: `src/components/log-panel.tsx`
- Create: `src/hooks/use-runtime-events.ts`
- Create: `src/lib/validators.ts`
- Create: `src/__tests__/rule-dialog.test.tsx`
- Create: `src/__tests__/log-panel.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/rule-list.tsx`
- Modify: `src/components/status-bar.tsx`
- Modify: `src/styles/app.css`
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)

- [ ] Step 1: 实现新增/编辑规则弹窗，字段覆盖设计文档定义的 MVP 字段
- [ ] Step 2: 实现前端即时校验，覆盖协议、端口、地址非空、基础 IP 合法性
- [ ] Step 3: 在弹窗中增加监听地址与目标地址预览
- [ ] Step 4: 实现单条规则操作按钮：启动、停止、编辑、删除
- [ ] Step 5: 实现底部日志面板，展示时间、来源、级别、内容，并限制内存缓存条数
- [ ] Step 6: 实现状态条，展示 `gost` 进程状态、当前运行规则数、最近错误
- [ ] Step 7: 增加防火墙提示文案，当监听地址不是回环地址时，提示用户可能需要手动放行 Windows 防火墙
- [ ] Step 8: 编写前端测试，覆盖弹窗提交流程、日志展示、状态更新
- [ ] Step 9: 执行前端测试

**Run:**

```bash
npm run test -- rule-dialog
npm run test -- log-panel
```

**Expected:**

- 弹窗与日志测试通过
- 非回环监听地址会出现防火墙提示

- [ ] Step 10: 回写本计划文档，标记 Task 7 已完成并记录测试结果

**Verification Criteria:**

- 可以完成规则新增、编辑、删除、单条启停的全链路交互
- 日志面板能实时看到 `app` / `gost` 两类日志
- 防火墙提示在正确场景下出现

---

### Task 8: 端到端验证与文档收尾

**任务状态：** `- [ ] 未完成`

**Files:**
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-design.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-design.md)
- Modify: [docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)
- Create: `docs/2026-04-07-tauri-windows-port-forwarding-verification-report.md`

- [ ] Step 1: 运行 Rust 全量测试
- [ ] Step 2: 运行前端全量测试
- [ ] Step 3: 运行应用构建验证
- [ ] Step 4: 在 Windows 上执行 MVP 手工验证矩阵
- [ ] Step 5: 输出验证报告，记录通过项、失败项、环境信息、待跟进问题
- [ ] Step 6: 回写设计文档，增加“实现状态”小节，链接实现计划与验证报告
- [ ] Step 7: 回写本计划文档，勾选全部已完成任务并补齐执行记录

**Run:**

```bash
cd src-tauri && cargo test
cd ..
npm run test
npm run tauri build
```

**Manual Verification Matrix:**

- [ ] `TCP 0.0.0.0:8080 -> 127.0.0.1:80`
- [ ] `TCP [::]:8080 -> [::1]:80`
- [ ] `TCP [::]:8081 -> 127.0.0.1:80`
- [ ] `TCP 0.0.0.0:8082 -> [::1]:80`
- [ ] `UDP 0.0.0.0:5353 -> 127.0.0.1:5353`
- [ ] `UDP [::]:5353 -> [::1]:5353`
- [ ] `UDP [::]:5354 -> 127.0.0.1:5353`
- [ ] `UDP 0.0.0.0:5354 -> [::1]:5353`

**Expected:**

- 自动化测试通过
- 构建通过
- MVP 关键转发组合在 Windows 环境中通过验证

**Verification Criteria:**

- `cargo test` 全通过
- `npm run test` 全通过
- `npm run tauri build` 通过
- 验证报告完整，且设计文档与实现计划都已回写状态

---

## 4. 实施顺序建议

建议严格按以下顺序执行：

1. Task 1：先把工程和 sidecar 打通
2. Task 2：再打好 Rust 模型与持久化基础
3. Task 3：完成校验和配置渲染
4. Task 4：把 `gost` 进程管理做稳
5. Task 5：再暴露 Tauri 命令和事件
6. Task 6：搭前端壳与列表
7. Task 7：补编辑弹窗、日志与状态
8. Task 8：做最终验收和文档回写

不要跳步，尤其不要在 Task 4 之前就实现复杂 UI 交互。

---

## 5. 风险提醒

- Tauri v2 sidecar 命名不正确会直接导致打包失败
- Windows Job Object 若处理不当，会留下端口占用的残留进程
- `gost` API 探活依赖本地回环地址，不能暴露到公网接口
- 单进程重载存在短暂中断窗口，这是 MVP 已知代价
- Windows 防火墙可能让局域网访问失败，MVP 只提示不自动提权修改系统配置

---

## 6. 执行记录

- 2026-04-07 Task 1 完成
  - 验证：`npm run tauri build`，`npm run tauri -- build --bundles deb`
  - 结果：`PASS`
  - 说明：补齐并替换为合法 RGBA `src-tauri/icons/icon.png`，`npm run tauri build` 已越过 icon 阻塞并进入打包阶段，`deb` 打包完整通过
- 2026-04-07 Task 2 完成
  - 验证：`cd src-tauri && cargo test --test rule_store_tests -- --nocapture`
  - 结果：`PASS`
  - 说明：完成规则模型、运行态模型、应用数据路径与 `RuleStore` 持久化基础，修正测试模块路径与 `tauri::Manager` 导入后，4 个 `rule_store` 测试全部通过
- 2026-04-07 Task 3 完成
  - 验证：`cd src-tauri && cargo test --test validator_tests -- --nocapture`，`cd src-tauri && cargo test --test gost_renderer_tests -- --nocapture`
  - 结果：`PASS`
  - 说明：补齐规则保存前/启动前校验与 `gost.yaml` 渲染，修复 `Protocol` 缺少 `Hash` 派生导致的编译错误后，2 组 Task 3 单测共 10 条全部通过
