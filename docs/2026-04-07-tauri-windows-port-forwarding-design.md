# Tauri Windows 端口转发工具设计方案

## 1. 背景与目标

本项目目标是实现一个基于 Tauri 的 Windows 桌面端口转发小工具，
第一阶段先交付 MVP，支持以下能力：

- 支持 `TCP` 和 `UDP`
- 支持监听 `0.0.0.0` 或 `::`，也支持指定 IPv4 / IPv6 地址
- 支持将监听端口转发到目标 IP 的端口
- 支持 `IPv4 -> IPv4`、`IPv6 -> IPv6`
- 支持 `IPv4 -> IPv6`、`IPv6 -> IPv4`
- 支持单条规则启动 / 停止
- 支持一键启动全部 / 停止全部
- 支持桌面界面配置规则和查看日志
- 支持本地持久化配置

第一阶段只实现 MVP 工具版。后续第二阶段在该基础上增强托盘、自启、
导入导出等桌面工具能力。

## 实现状态

- 实现计划：
  [2026-04-07-tauri-windows-port-forwarding-implementation-plan.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)
- 验证报告：
  [2026-04-07-tauri-windows-port-forwarding-verification-report.md](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-verification-report.md)

当前进度：

- Task 1 - Task 7 已完成
- Task 8 自动化验证已部分完成
- Windows 防火墙规则自动同步已实现
- 单实例应用行为已实现，重复启动会唤起已有主窗口
- Windows 手工验证矩阵与完整 `tauri build` 退出结果仍待收口


## 2. 范围界定

### 2.1 MVP 范围

- 单窗口桌面应用
- 规则增删改查
- TCP / UDP 端口转发
- IPv4 / IPv6 地址输入与转发
- 使用 `gost.exe` 作为 sidecar 执行转发
- 使用 `TOML` 保存规则
- 支持运行日志展示
- 支持单条规则与全局启停

### 2.2 明确不做

- Rust 原生实现 TCP / UDP 转发内核
- Windows Service
- 规则分组、批量编辑、拖拽排序
- 实时连接统计、流量图表、限流、黑白名单
- 自动更新
- 多窗口
- 高级代理链、负载均衡、多目标转发


## 3. 总体架构

推荐采用 `Tauri + Rust App Core + gost.exe sidecar` 架构。

```text
┌──────────────────────┐
│   Tauri Frontend UI  │
└──────────┬───────────┘
           │ Command / Event
┌──────────▼───────────┐
│    Rust App Core     │
│  ├─ Rule Store       │
│  ├─ Rule Validator   │
│  ├─ Config Renderer  │
│  ├─ Process Manager  │
│  ├─ Runtime State    │
│  └─ Log Bridge       │
└──────────┬───────────┘
           │ spawn / stop / reload
┌──────────▼───────────┐
│    gost.exe sidecar  │
│ TCP / UDP forwarding │
└──────────────────────┘
```

### 3.1 角色分工

#### Tauri 前端

- 提供规则列表和编辑界面
- 触发新建、编辑、删除、启动、停止等操作
- 展示规则状态与日志

#### Rust App Core

- 维护规则持久化
- 校验规则合法性
- 将内部规则模型渲染为 `gost` 配置
- 启动、停止、重载 `gost.exe`
- 维护 `gost` 本地 API 健康检查与状态探测
- 采集 stdout / stderr 并回传前端
- 管理运行态与错误状态

#### gost.exe

- 真正监听端口并执行 TCP / UDP 转发

### 3.2 选择 sidecar 的原因

- MVP 落地更快，避免在第一版手写网络转发栈
- `gost` 已支持本地 TCP / UDP 端口转发
- sidecar 模式故障隔离更好
- 后续增强桌面能力时无需重做转发内核


## 4. 配置分层

建议将用户配置与运行配置拆成两层：

```text
rules.toml   -> 用户可理解、稳定的产品配置
gost.yaml    -> Rust 渲染后给 gost 实际使用的执行配置
```

### 4.1 这样拆分的价值

- 前端与 Rust 不直接依赖 `gost` 原始配置结构
- 将来如果更换转发内核，用户配置层尽量不变
- 错误提示和校验逻辑可以围绕业务模型设计


## 5. 规则模型设计

每条规则建议使用如下字段：

- `id`: 唯一标识
- `name`: 规则名称
- `enabled`: 是否默认参与“启动全部”
- `protocol`: `tcp` 或 `udp`
- `listen_host`: 监听主机地址
- `listen_port`: 监听端口
- `target_host`: 目标主机地址
- `target_port`: 目标端口
- `remark`: 备注，可选
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 5.1 地址存储原则

内部模型统一使用：

```text
host + port
```

不建议把地址直接存成一个原始字符串。

原因：

- 表单校验更自然
- IPv6 方括号处理更可控
- 序列化与显示逻辑更清晰

### 5.2 IPv6 存储规范

- 存储层不带 `[]`
- 渲染到 socket 地址字符串时再补 `[]`

示例：

- 存储 `::1`，渲染为 `[::1]:8080`
- 存储 `::`，渲染为 `[::]:5353`


## 6. `rules.toml` 设计

建议采用 `TOML` 作为持久化格式。

理由：

- 结构稳定，适合“应用设置 + 规则列表”
- 人工查看和修改友好
- Rust 使用 `serde + toml` 处理成本低
- 比 YAML 更少出现缩进和类型推断问题

示例：

```toml
version = 1

[app]
auto_start_enabled_rules = false
log_level = "info"

[[rules]]
id = "9f0f1d7d"
name = "tcp-8080"
enabled = true
protocol = "tcp"
listen_host = "0.0.0.0"
listen_port = 8080
target_host = "192.168.1.10"
target_port = 80
remark = "web forward"
created_at = "2026-04-07T10:00:00Z"
updated_at = "2026-04-07T10:00:00Z"

[[rules]]
id = "81f3db92"
name = "udp-5353"
enabled = false
protocol = "udp"
listen_host = "::"
listen_port = 5353
target_host = "127.0.0.1"
target_port = 5353
remark = "mdns relay"
created_at = "2026-04-07T10:05:00Z"
updated_at = "2026-04-07T10:05:00Z"
```


## 7. `gost.yaml` 渲染策略

Rust 根据当前运行集合生成 `gost.yaml`。

MVP 中建议：

- 一个应用实例只维护一个 `gost.exe`
- 当前运行中的全部规则统一渲染进同一个 `gost.yaml`
- 单条规则启停通过“修改运行集合 + 重新渲染 + 受控重启”实现

TCP 规则映射为一个 service，方向如下：

```yaml
services:
  - name: rule-9f0f1d7d
    addr: ":8080"
    handler:
      type: tcp
    listener:
      type: tcp
    forwarder:
      nodes:
        - name: target-0
          addr: 192.168.1.10:80
```

UDP 规则对应 `udp` 监听与转发组合，MVP 示例：

```yaml
services:
  - name: rule-81f3db92
    addr: ":5353"
    handler:
      type: udp
    listener:
      type: udp
    forwarder:
      nodes:
        - name: target-0
          addr: 127.0.0.1:5353
```

MVP 对 UDP 的策略是：

- UI 不暴露 `keepAlive`、`ttl`、`readBufferSize` 等高级参数
- 渲染器默认不写入这些 `metadata` 字段，直接使用 `gost` 默认值
- 若第二阶段需要做高级 UDP 调优，再在规则模型中显式增加高级字段

按 `gost` UDP listener 文档，默认值至少包括：

- `backlog = 128`
- `keepAlive = false`
- `ttl = 5s`
- `readBufferSize = 1500`
- `readQueueSize = 128`

> 具体字段命名与结构应以项目接入时锁定的 `gost` 版本文档为准。
> 建议在项目初始化时锁定 `gost v3.x` 的具体小版本，避免配置结构漂移。


## 8. 运行模型设计

### 8.1 单 `gost.exe` 进程

第一阶段推荐只维护单个 sidecar 进程。

优点：

- 进程模型简单
- 日志集中
- 资源占用低
- 排障容易

不建议 MVP 做“一条规则一个进程”，因为会增加：

- 进程数量
- 状态管理复杂度
- 日志分散度

### 8.1.1 已知代价：重载窗口

MVP 采用“单进程 + 全量重载配置”时，单条规则启停会触发：

```text
停止当前 gost -> 写入新配置 -> 启动新 gost
```

这意味着：

- 其他正在运行的规则会有一个短暂中断窗口
- 该窗口在规则变更频繁时会更明显

这是 MVP 的已知设计代价，需要在产品说明和实现中明确。

第二阶段升级路径：

- 启用 `gost` Web API 的动态 service 增删能力
- 对“新增规则/删除规则”直接走 API，而不是整进程重启
- 仅在全局配置变化或 API 不可用时退回全量重载

### 8.2 配置态与运行态分离

建议区分两个概念：

- `enabled`
  - 规则默认是否参与“启动全部”
- `running`
  - 规则当前是否已经被下发进运行中的 `gost` 配置

这两个字段语义不同，不能混用。

### 8.3 当前运行集合

Rust App Core 维护一个“当前运行集合”。

操作行为：

- 单条启动：加入运行集合
- 单条停止：移出运行集合
- 启动全部：将全部 `enabled = true` 规则加入运行集合
- 停止全部：清空运行集合并停止 `gost`

### 8.4 进程生命周期保护

需要显式处理应用异常退出后的 sidecar 残留问题。

推荐策略：

1. **主方案：Windows Job Object**
   - Rust 在启动 `gost.exe` 后立即将其绑定到 Job Object
   - 当父进程退出、崩溃或被杀死时，由系统自动终止 `gost.exe`

2. **兜底方案：PID 文件清理**
   - 启动成功后写入 PID 文件
   - 应用下次启动时检查 PID 文件
   - 若发现残留 `gost` 进程且属于本应用路径，则先清理再继续启动

这两层结合，能显著降低“重启应用后端口仍被占用”的概率。


## 9. 启动 / 停止 / 重载流程

### 9.1 全局启动全部

1. 读取 `rules.toml`
2. 过滤出 `enabled = true` 的规则
3. 执行启动前校验
4. 生成 `gost.yaml`
5. 启动 `gost.exe`
6. 持续采集日志
7. 更新前端运行状态

### 9.2 单条启动

1. 将规则加入运行集合
2. 重新生成 `gost.yaml`
3. 受控重启 `gost.exe`
4. 刷新规则状态和日志

### 9.3 单条停止

1. 将规则移出运行集合
2. 重新生成 `gost.yaml`
3. 受控重启 `gost.exe`
4. 刷新规则状态和日志

### 9.4 全局停止

1. 停止 `gost.exe`
2. 清空运行集合
3. 广播状态变化

### 9.5 受控重启流程

建议所有变更走统一重载路径，并加互斥锁：

```text
加锁
  ↓
停止旧 gost
  ↓
等待退出（短超时）
  ↓
写入新 gost.yaml
  ↓
启动新 gost
  ↓
检测启动结果
  ↓
同步运行状态
  ↓
释放锁
```

这样可以避免用户快速连点按钮导致状态错乱。

### 9.6 启动结果检测

“检测启动结果”必须有明确机制，MVP 建议采用 **本地 API 探活**，
而不是依赖日志关键词。

推荐做法：

1. `gost` 启动时同时开启仅绑定本机回环地址的 API 服务
2. Rust 侧在启动后轮询本地 API
3. 当子进程仍存活，且 API 在超时前返回成功响应时，判定启动成功
4. 若子进程提前退出，或 API 超时不可达，则判定启动失败

建议参数：

- API 仅监听 `127.0.0.1:<port>` 或 `[::1]:<port>`
- 轮询端点：`/api/config/services`
- 轮询间隔：`100ms`
- 启动超时：`2s`

推荐原因：

- 不依赖 `stdout` 文本格式
- 同时适用于 TCP / UDP 规则
- 后续第二阶段改用动态配置 API 时可以直接复用

### 9.7 API 服务用途边界

MVP 中启用 `gost` API 服务，主要用于：

- 启动成功探测
- 本地运行状态探测
- 为第二阶段动态 service 管理预留基础能力

MVP 的规则变更主路径仍保持为：

- `rules.toml`
- 渲染 `gost.yaml`
- 受控重启

不在第一阶段直接把规则增删改全部切到 API。


## 10. 日志与状态设计

### 10.1 日志来源

日志分两类：

- 应用日志
  - 规则校验失败
  - 配置文件读写失败
  - `gost` 启停失败
  - 重载超时
- `gost` 运行日志
  - 来自 sidecar 的 stdout / stderr

### 10.2 日志结构

前端日志面板建议统一展示：

- 时间戳
- 来源：`app` / `gost`
- 级别：`info` / `warn` / `error`
- 内容

### 10.3 状态枚举

每条规则建议展示以下运行状态：

- `stopped`
- `starting`
- `running`
- `stopping`
- `error`

全局状态建议展示：

- `gost` 进程状态
- 当前生效规则数
- 最近一次错误摘要

### 10.4 MVP 状态说明

MVP 中“规则运行中”的语义建议定义为：

- 该规则已经被写入当前运行配置
- 且 `gost.exe` 当前进程处于运行状态

第一阶段不承诺以下能力：

- 精确探测目标端口是否实时可达
- 精确统计每条规则的实时连接数
- 精确确认每条报文已成功转发

### 10.5 状态同步方式

前后端状态同步建议采用：

- **事件推送为主**
- **命令查询为辅**

具体做法：

- Rust 侧在规则状态变化、`gost` 启停、`gost` 异常退出时，主动
  `emit` 事件给前端
- 前端在页面初始化时调用 `get_runtime_status()` 获取一次快照
- 后续主要依赖事件更新界面

这样能避免仅靠轮询导致的崩溃感知延迟问题。


## 11. 协议与地址支持策略

### 11.1 MVP 支持矩阵

协议：

- `TCP`
- `UDP`

地址族组合：

- `IPv4 -> IPv4`
- `IPv6 -> IPv6`
- `IPv4 -> IPv6`
- `IPv6 -> IPv4`

监听地址：

- `0.0.0.0`
- `::`
- 指定 IPv4
- 指定 IPv6

目标地址：

- IPv4
- IPv6
- 域名可兼容，但 MVP 不作为主宣传能力

### 11.2 地址格式化函数

建议在 Rust 内部提供统一函数：

```text
format_socket_addr(host, port) -> String
```

规则：

- IPv4: `127.0.0.1 + 8080 -> 127.0.0.1:8080`
- IPv6: `::1 + 8080 -> [::1]:8080`
- 通配 IPv6: `:: + 5353 -> [::]:5353`

### 11.3 双栈行为说明

不要把 `0.0.0.0` 和 `::` 视为同一概念。

Windows 下双栈监听行为可能受系统 socket 选项和底层实现影响，
因此：

- 前端保持地址语义透明
- 系统只做格式规范化
- 不偷偷把 `::` 解释成“自动双栈监听”


## 12. 校验策略

建议分三层校验。

### 12.1 前端即时校验

- 名称非空
- 协议必须为 `tcp` / `udp`
- 端口范围 `1..=65535`
- 监听地址不能为空
- 目标地址不能为空
- IP 或域名格式初步合法

### 12.2 保存前业务校验

由 Rust 执行：

- `id` 必须唯一
- 名称允许重复，`id` 不允许重复
- 去除首尾空白
- 地址字段规范化
- TOML 序列化完整
- 同协议下监听冲突检查

### 12.3 启动前运行校验

- 当前运行集合非空
- `gost.exe` 文件存在
- `gost.yaml` 可成功写入
- 渲染后的配置结构合法
- 同协议监听端口冲突

MVP 建议不单独做“端口抢占式预探测”，而是以 `gost` 实际启动结果
和错误日志为准。

### 12.4 Windows 防火墙策略

监听 `0.0.0.0` 或 `::` 时，Windows 防火墙可能导致：

- 首次运行弹出放行提示
- 局域网访问被静默阻断

因此设计中必须显式包含防火墙策略。

当前实现策略：

1. 仅在 Windows 下处理防火墙规则
2. 仅对监听非回环地址的规则生效
3. 按“协议 + 本地端口”精确创建入站规则，不按整个 `gost.exe` 程序放行
4. 启动规则成功后自动同步对应防火墙规则
5. 停止规则、停止全部、删除规则时自动删除对应防火墙规则
6. 若当前进程不是管理员，则在同步时触发 UAC 提权
7. 若用户取消 UAC 或命令失败，端口转发本身继续运行，但日志面板会追加失败信息
8. 同步前会先查询现有 `Porthole-*` 规则，仅在确有增删变化时才执行 PowerShell 与提权流程，避免每次启动都重复改写防火墙

实现约束：

- 自动同步会修改用户系统防火墙配置
- 首次同步时可能弹出 UAC 确认框
- 当前实现依赖 PowerShell `New-NetFirewallRule` / `Remove-NetFirewallRule`
- Windows 手工验证矩阵仍需补做，重点确认提权、规则增删和局域网访问链路

### 12.5 启动恢复策略

当前实现已增加“恢复上次运行中的规则”能力：

1. 每次运行集合成功变更后，将 `active_rule_ids` 持久化到本地 `runtime-state.toml`
2. 应用启动后在后台优先读取该文件，不阻塞主窗口展示
3. 仅恢复其中仍然存在于 `rules.toml` 的规则
4. 若恢复失败，只追加 `app` 日志，不阻断应用主界面启动

这意味着当前行为是“恢复上次实际在运行的规则”，而不是“自动启动所有 `enabled = true` 的规则”。


## 13. UI 信息架构

MVP 保持单窗口，三段式布局：

```text
┌──────────────────────────────────────────────┐
│ 顶部工具栏                                  │
│ [新增规则] [启动全部] [停止全部] [刷新状态] │
├──────────────────────────────────────────────┤
│ 规则列表区                                  │
│ 名称 | 协议 | 监听 | 目标 | 启用 | 状态 | 操作 │
├──────────────────────────────────────────────┤
│ 底部日志区                                  │
│ 时间 | 来源 | 级别 | 内容                    │
└──────────────────────────────────────────────┘
```

### 13.1 规则列表字段

- 名称
- 协议
- 监听地址
- 目标地址
- 默认启用
- 当前状态
- 操作按钮

建议操作按钮：

- 启动
- 停止
- 编辑
- 删除

### 13.2 新建 / 编辑弹窗

表单字段：

- 名称
- 协议
- 监听地址
- 监听端口
- 目标地址
- 目标端口
- 默认启用
- 备注

建议按钮：

- 保存
- 保存并启动
- 取消

### 13.3 预览字段

建议在表单中增加只读预览：

- 监听预览
- 目标预览

这样用户在填写 IPv6 时能立刻看到最终地址渲染效果。

### 13.4 MVP 暂不实现的 UI 能力

- 规则分组
- 批量编辑
- 托盘菜单
- 图表统计
- 高级筛选
- 主题系统


## 14. 打包与目录布局建议

### 14.1 Sidecar 打包

将 `gost.exe` 作为 Tauri sidecar 打包进应用。

建议目录方向：

```text
src-tauri/
├── binaries/
│   └── gost-x86_64-pc-windows-msvc.exe
└── tauri.conf.json
```

Tauri v2 对 sidecar 命名有明确要求：

- `externalBin` 中配置的是无三元组后缀的基础路径
- 实际文件名必须带 `-$TARGET_TRIPLE` 后缀

Windows 例子：

```json
{
  "bundle": {
    "externalBin": ["binaries/gost"]
  }
}
```

对应文件：

```text
src-tauri/binaries/gost-x86_64-pc-windows-msvc.exe
```

如果未来支持其他 Windows 架构，还需要准备对应三元组文件，例如：

- `gost-aarch64-pc-windows-msvc.exe`

### 14.2 运行时文件

应用运行时建议至少维护：

- `rules.toml`
- `gost.yaml`
- 可选运行日志文件

建议放在应用数据目录，而不是项目安装目录。


## 15. Rust 模块拆分建议

建议尽量按职责拆模块，避免一个文件做所有事情。

```text
src-tauri/src/
├── app_state.rs
├── commands/
│   ├── rules.rs
│   └── runtime.rs
├── model/
│   ├── rule.rs
│   └── runtime.rs
├── service/
│   ├── rule_store.rs
│   ├── validator.rs
│   ├── gost_renderer.rs
│   ├── gost_process.rs
│   └── log_bridge.rs
└── main.rs
```

说明：

- `model`: 领域模型与枚举
- `service/rule_store`: 持久化读写
- `service/validator`: 校验逻辑
- `service/gost_renderer`: 生成 `gost.yaml`
- `service/gost_process`: sidecar 生命周期管理
- `commands`: 暴露给前端的 Tauri Command

实现时建议：

- `RuleStore` 层增加 `Mutex` 或由 `app_state` 统一串行化写入
- 所有写 `rules.toml` 的路径都走单入口，避免并发写交叉覆盖


## 16. 核心接口建议

### 16.1 前端到 Rust 的命令

- `list_rules()`
- `create_rule(input)`
- `update_rule(id, input)`
- `delete_rule(id)`
- `start_rule(id)`
- `stop_rule(id)`
- `start_all_enabled_rules()`
- `stop_all_rules()`
- `get_runtime_status()`
- `clear_logs()`

### 16.2 Rust 内部服务接口方向

- `RuleStore`
  - `load()`
  - `save(rules)`

- `RuleValidator`
  - `validate_for_save(rule, existing_rules)`
  - `validate_for_run(rules)`

- `GostRenderer`
  - `render(running_rules) -> gost.yaml`

- `GostProcessManager`
  - `start(config_path)`
  - `stop()`
  - `reload(config_path)`
  - `is_running()`

- `RuntimeEventEmitter`
  - `emit_runtime_changed(snapshot)`
  - `emit_process_exited(reason)`


## 17. MVP 测试策略

### 17.1 单元测试重点

- 地址格式化
- IPv6 方括号渲染
- 规则校验
- TOML 读写
- `gost.yaml` 渲染输出
- 运行集合变更逻辑

### 17.2 集成测试重点

- 新建规则后写入配置文件
- 单条启停触发 `gost` 重载
- 启动全部 / 停止全部行为正确
- 日志桥接可读取 stdout / stderr

### 17.3 PoC 验证矩阵

Windows 上建议优先做以下组合验证：

1. `TCP 0.0.0.0:8080 -> 127.0.0.1:80`
2. `TCP [::]:8080 -> [::1]:80`
3. `TCP [::]:8081 -> 127.0.0.1:80`
4. `TCP 0.0.0.0:8082 -> [::1]:80`
5. `UDP 0.0.0.0:5353 -> 127.0.0.1:5353`
6. `UDP [::]:5353 -> [::1]:5353`
7. `UDP [::]:5354 -> 127.0.0.1:5353`
8. `UDP 0.0.0.0:5354 -> [::1]:5353`

验证目标：

- 配置是否成功渲染
- `gost` 是否成功启动
- 端口是否成功监听
- 数据是否成功转发


## 18. 第二阶段扩展位

本设计已为第二阶段预留扩展点。

第二阶段可以在不推翻 MVP 的前提下增加：

- 系统托盘
- 开机自启
- 配置导入 / 导出
- 应用关闭后最小化到托盘
- 更丰富的运行日志
- 更细粒度运行状态展示

之所以能平滑扩展，是因为 MVP 已提前拆清：

- 用户配置层
- 运行配置层
- 进程管理层
- UI 展示层


## 19. 风险与对策

### 风险 1：Windows 下 IPv6 / 双栈行为存在平台差异

对策：

- 将 `0.0.0.0` 与 `::` 语义分开处理
- 不做“自动双栈”假设
- 通过 PoC 矩阵优先验证

### 风险 2：不同版本 `gost` 配置结构存在差异

对策：

- 尽早锁定 `gost` 版本
- 生成器基于锁定版本文档实现
- 渲染层与业务模型解耦

### 风险 2.1：`gost` API 与配置文件能力边界不一致

对策：

- MVP 先以配置文件驱动为主
- API 仅用于健康检查与第二阶段扩展
- 接入动态 service 管理前，先完成 Windows 环境 PoC 验证

### 风险 3：频繁点击启停可能造成状态竞争

对策：

- 所有启停操作走统一互斥重载通道
- 前端状态切换为 `starting` / `stopping`
- 按钮在处理中短暂禁用

### 风险 4：日志量过大影响前端性能

对策：

- 前端仅缓存最近 `500 ~ 1000` 条
- 支持手动清空日志


## 20. 结论

第一阶段最优落地路线是：

- 使用 Tauri 提供桌面 UI
- 使用 Rust 作为规则与运行编排核心
- 使用 `gost.exe` sidecar 承担真正的端口转发
- 使用 `rules.toml` 持久化产品配置
- 使用 `gost.yaml` 驱动运行时执行
- 使用“单进程 + 运行集合 + 受控重载”的方式实现单条与全局启停

这个方案的优点是：

- MVP 开发速度快
- 结构清晰
- 可测试性较好
- 为第二阶段扩展保留了充足空间


## 参考资料

- GOST 项目主页：<https://github.com/go-gost/gost>
- GOST Port Forwarding：<https://gost.run/en/tutorials/port-forwarding/>
- GOST Dynamic Configuration：<https://gost.run/en/tutorials/api/config/>
- GOST Web API Overview：<https://gost.run/en/tutorials/api/overview/>
- GOST UDP Listener：<https://gost.run/en/reference/listeners/udp/>
- GOST UDP Handler：<https://gost.run/en/reference/handlers/udp/>
- Tauri Sidecar 文档：<https://v2.tauri.app/zh-cn/develop/sidecar/>
