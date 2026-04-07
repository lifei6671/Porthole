# Tauri Windows 端口转发工具 MVP 验证报告

## 1. 环境信息

- 日期：`2026-04-07`
- 仓库路径：`/home/lifei6671/src/github.com/lifei6671/Porthole`
- 验证环境：Linux 开发机
- 说明：当前环境可完成 Rust 自动化测试、前端自动化测试和 Linux 侧 Tauri 打包验证；Windows 手工转发矩阵尚未执行

## 2. 自动化验证结果

### 2.1 Rust 自动化测试

- 命令：`cd src-tauri && cargo test`
- 结果：`PASS`
- 摘要：
  - `commands_tests`：6 通过
  - `firewall_tests`：4 通过
  - `gost_process_tests`：7 通过
  - `gost_renderer_tests`：8 通过
  - `rule_store_tests`：7 通过
  - `validator_tests`：8 通过

### 2.2 前端自动化测试

- 命令：`npm run test`
- 结果：`PASS`
- 摘要：
  - `rule-list.test.tsx`：3 通过
  - `rule-dialog.test.tsx`：2 通过
  - `log-panel.test.tsx`：2 通过

### 2.3 前端构建

- 命令：`npm run build`
- 结果：`PASS`

### 2.4 Tauri 构建

- 命令：`npm run tauri build`
- 结果：`PARTIAL`
- 摘要：
  - 应用 release 编译成功
  - `deb` 包生成成功
  - `rpm` 包生成成功
  - 进入 `appimage` 阶段时需要下载 `AppRun-x86_64`
- 当前阻塞：
  - 构建在下载 `https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-x86_64` 时长时间等待，当前未拿到完整成功退出结果

### 2.5 Windows 交叉构建

- 命令：`make build-win`
- 结果：`PASS`
- 摘要：
  - 已生成 `bin/porthole.exe`
  - 已同步 `bin/gost.exe`
  - 已同步 `bin/WebView2Loader.dll`
  - 当前 Windows 产物包含 sidecar 本地复制、隐藏子进程窗口、防火墙幂等同步与启动恢复逻辑

## 3. 功能覆盖情况

当前已完成的 MVP 前端/后端能力：

- 规则增删改查
- 单条规则启动 / 停止
- 启动全部 / 停止全部
- `rules.toml` 持久化
- `gost.yaml` 渲染
- `gost` 进程管理、PID 清理、日志采集、事件推送
- Windows 下 sidecar 本地复制与黑色命令行隐藏
- Windows 下按协议与端口自动添加 / 删除防火墙规则
- 防火墙同步前先比对现有 `Porthole-*` 规则，只有实际变化时才执行 PowerShell / UAC
- 应用启动时自动恢复上次实际运行中的规则
- 前端规则弹窗、即时校验、地址预览、日志面板、状态条、防火墙自动放行提示

## 4. 尚未完成的验证项

### 4.1 Windows 手工验证矩阵

以下场景仍需在 Windows 环境执行：

- `TCP 0.0.0.0:8080 -> 127.0.0.1:80`
- `TCP [::]:8080 -> [::1]:80`
- `TCP [::]:8081 -> 127.0.0.1:80`
- `TCP 0.0.0.0:8082 -> [::1]:80`
- `UDP 0.0.0.0:5353 -> 127.0.0.1:5353`
- `UDP [::]:5353 -> [::1]:5353`
- `UDP [::]:5354 -> 127.0.0.1:5353`
- `UDP 0.0.0.0:5354 -> [::1]:5353`

另外需补充以下 Windows 系统交互验证：

- 启动非回环监听规则时，是否正确弹出 UAC
- 确认 UAC 后，是否成功创建对应 `Porthole-*` 防火墙规则
- 停止 / 删除规则后，防火墙规则是否被清理
- 拒绝 UAC 后，端口转发是否仍继续运行且日志面板出现失败说明

### 4.2 完整 Tauri 打包退出

- 需要确认 `npm run tauri build` 在当前环境是否能完整越过 `appimage` 下载阶段并正常退出
- 如当前开发环境不要求 `appimage`，可改为按目标 bundle 显式构建，避免被额外平台工件阻塞

## 5. 结论

截至 `2026-04-07`：

- 自动化测试整体稳定
- MVP 的核心代码与前端交互链路已经打通
- Task 8 仍未完成，主要缺口是：
  - Windows 手工转发矩阵尚未执行
  - Windows 防火墙自动同步与 UAC 交互尚未手工确认
  - `npm run tauri build` 在 `appimage` 下载阶段未完成最终退出验证
