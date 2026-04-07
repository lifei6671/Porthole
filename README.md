# Porthole

**一款为 Windows 打造的桌面端口转发工具。**

Porthole 想解决的是一件很现实的事：
当你只是想把一个端口稳稳转出去时，不应该被命令行、复杂配置和系统细节反复打断。

它提供一个清爽的桌面工作台，让你用更低的心智负担完成：

- `TCP / UDP` 端口转发
- `IPv4 / IPv6` 监听与目标转发
- `IPv4 -> IPv6`、`IPv6 -> IPv4` 互转
- 规则级启停、全局启停、日志排障
- Windows 防火墙自动同步

---

## 为什么是 Porthole

很多端口转发工具能“用”，但不一定“顺手”。

Porthole 更像一个真正的桌面产品，而不是套了壳的脚本：

- **更直观**
  左侧导航 + 右侧工作区，规则、日志、设置分区清晰。

- **更省心**
  启动规则时自动处理 `gost` sidecar、运行状态、日志采集和防火墙同步。

- **更适合 Windows**
  支持托盘驻留、单实例唤起、自动恢复上次运行中的规则。

- **更适合日常使用**
  不只是“能转发”，而是把创建、启停、查看状态、排查问题这一整套流程做顺。

---

## 核心特性

- **桌面化规则管理**
  用图形界面创建、编辑、删除和启动端口转发规则。

- **支持 TCP / UDP**
  覆盖常见服务转发和局域网调试场景。

- **支持 IPv4 / IPv6**
  既支持同族转发，也支持 `IPv4 <-> IPv6` 互转。

- **基于 `gost` 的稳定转发内核**
  前端负责体验，底层交给成熟组件执行。

- **自动同步 Windows 防火墙**
  非回环监听规则启动时，会按协议和端口精确申请放行。

- **托盘驻留**
  关闭窗口时可直接隐藏到托盘，不中断当前转发。

- **自动恢复**
  重新启动应用后，可自动恢复上次仍在运行的规则集合。

- **混合日志面板**
  同时查看应用日志和 `gost` 运行日志，排障更直接。

---

## 适合谁

Porthole 特别适合这些场景：

- 本地开发时，把服务端口临时转发到局域网设备
- 调试需要 `IPv4 / IPv6` 混合环境的服务
- 为测试机、虚拟机、容器服务做快速转发
- 不想长期记忆和维护一堆命令行参数的人

---

## 当前体验

当前版本已经具备完整 MVP 工作流：

- 首页：看状态、看关键指标
- 规则：管理转发规则、单条启停
- 日志：查看 app / gost 混合日志
- 设置：查看当前运行策略和系统说明

目前已经支持：

- 单实例运行
- 自定义关闭提示
- 隐藏到托盘
- Windows 交叉构建产物输出到 `bin/`

---

## 快速开始

### 安装依赖

```bash
npm install
```

### 本地开发

```bash
npm run dev
```

如果你要配合 Tauri 开发运行：

```bash
npm run tauri dev
```

### 前端构建

```bash
npm run build
```

### 一键构建可执行文件

当前仓库已经提供 `Makefile`：

```bash
make build
```

Windows 产物可用：

```bash
make build-win
```

构建后的文件会放在根目录的 `bin/` 下。

---

## 技术栈

- **Tauri 2**
- **React 18**
- **TypeScript**
- **Rust**
- **gost**

---

## 项目文档

如果你想看更完整的设计和实现说明，可以直接看这些文档：

- [设计方案](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-design.md)
- [实现计划](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-implementation-plan.md)
- [验证报告](/home/lifei6671/src/github.com/lifei6671/Porthole/docs/2026-04-07-tauri-windows-port-forwarding-verification-report.md)

---

## Roadmap

接下来优先考虑的方向：

- 更完整的 Windows 手工验证矩阵
- 更细粒度的设置项
- 自启动与更多托盘行为
- 导入 / 导出规则
- 更丰富的运行状态展示

---

## License

本项目使用 [MIT License](LICENSE)。
