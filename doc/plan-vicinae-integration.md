# Plan: Starcatch × Vicinae 集成

> 让 Starcatch 星捕通过 Vicinae 启动器实现一键捕获星光 ✨

---

## 1. 概述

### 1.1 目标

通过 Vicinae 的 script 系统和 dmenu 模式，为 Starcatch 提供图形化的快速操作界面，无需打开终端即可完成捕获、查看和管理。

### 1.2 背景

- **Starcatch** (星捕): Rust CLI + C++ Qt 6 GUI 的 Wayland 原生捕获工具，支持 Todo/Idea/Log 三种记录类型，Pipe 模式支持 stdin 输入
- **Vicinae**: 已安装在 `/usr/bin/vicinae`，支持 `dmenu` 模式（从 stdin 渲染选择列表）和 `script` 系统（注册自定义脚本）
- **用户环境**: Wayland, Emacs 键绑定已配置, Matugen 主题

### 1.3 技术约束

- Vicinae 脚本存放在 `~/.config/vicinae/scripts/` 目录
- 脚本通过 `# @vicinae.*` 注释声明元数据
- dmenu 模式返回用户选中的行文本
- 数据存储在 `~/.local/share/starcatch/starcatch.db`（CLI 与 GUI 共享）

---

## 2. 分阶段实现

### Phase 1: 基础快捷操作脚本（MVP）

**目标**: 实现一个 vicinae script，提供最常用的 3 个操作，可直接绑定快捷键

#### 2.1.1 脚本: `starcatch-capture`

```bash
# @vicinae.schemaVersion 1
# @vicinae.title Starcatch Capture
# @vicinae.mode compact
# @vicinae.exec ["/bin/bash"]
```

功能菜单:

```
Starcatch 星捕
├── 📋 新增待办    → 弹出输入 → starcatch pipe todo
├── 💭 新增灵感    → 弹出输入 → starcatch pipe idea
└── 📓 写日记      → 弹出输入 → starcatch pipe log
```

实现方式：

1. 用 vicinae dmenu 渲染菜单
2. 用户选择类型
3. 用第二级 dmenu（或 wofi/dmenu 等外部工具）获取文本输入
4. 调用 `starcatch pipe <type>` 写入数据库
5. 显示 toast/通知确认

#### 2.1.2 快捷键绑定

在 `~/.config/vicinae/settings.json` 中添加：

```json
"shortcuts": [
  {
    "key": "Ctrl+Shift+C",
    "command": "starcatch-capture"
  }
]
```

---

### Phase 2: 查看与管理面板

**目标**: 在 vicinae 中浏览和操作已有数据

#### 2.2.1 脚本: `starcatch-list`

功能菜单:

```
📋 查看待办
├── 显示所有 P0/P1/P2/P3 待办
├── 选择某一项 → 标记完成/归档/查看详情
├── ⬜ 买牛奶 (P2) due:明天
├── ⬜ 写博客 (P1) due:下周一
└── ...
```

实现方式：

1. `starcatch todo list --all` 获取数据
2. 格式化后通过 vicinae dmenu 渲染
3. 选中后根据状态（pending/done/archived）提供操作选项
4. 执行 `starcatch todo done <id>` 或 `starcatch todo archive <id>`

同理实现：
- `starcatch-list-ideas` — 查看近期灵感
- `starcatch-list-logs` — 查看近期日志

#### 2.2.2 导航层级

```
Starcatch 星捕
├── 📋 新增... (Phase 1)
├── 📋 查看待办 → 选择 → 标记完成/归档
├── 💭 查看灵感 → 选择 → 查看详情
├── 📓 查看日志 → 选择 → 查看详情
└── 📊 统计信息 → 显示今日/本周统计
```

---

### Phase 3: 增强功能

**目标**: 更丰富的交互体验

#### 2.3.1 自然语言快速输入

用一级 dmenu 直接输入文本，自动识别意图：

```
用户输入: "明天买牛奶 P1 #shopping"
→ 解析: title="买牛奶", due="明天", priority=P1, tags=["shopping"]
→ 调用 starcatch todo add ...
```

可复用 Starcatch 已有的 `parse_natural_date()` 逻辑，在 bash 端用简单规则实现，或通过 `starcatch` 子命令扩展支持。

#### 2.3.2 快速查看详情

选中一条记录后，vicinae 的 quick look 面板显示详情（利用 `--no-quick-look` 的反面，即默认的 quick look 功能）。

#### 2.3.3 Emacs 集成桥接

通过 vicinae script 将内容直接发送到 Emacs org-capture：

```
选择 "发送到 Emacs" → vicinae 调用 emacsclient 打开 org-capture 模板
```

---

## 3. 文件结构

```
~/.config/vicinae/
├── settings.json              # 快捷键绑定（已有）
├── colors.toml                # 主题（已有）
└── scripts/
    ├── starcatch-capture      # Phase 1: 快速捕获
    ├── starcatch-list         # Phase 2: 查看管理
    ├── starcatch-list-ideas   # Phase 2: 查看灵感
    ├── starcatch-list-logs    # Phase 2: 查看日志
    └── lib/
        └── starcatch-common   # 共享函数库（通知、格式化等）
```

---

## 4. 关键技术决策

### 4.1 文本输入方案

由于 vicinae dmenu 是**选择列表**而非自由文本输入框，获取用户输入有以下方案：

| 方案 | 说明 | 推荐度 |
|:--|:--|:--:|
| **方案 A: wofi --dmenu** | 用 wofi 弹出文本输入框，获取输入后 pipe 给 starcatch | ⭐ 推荐（Wayland 原生） |
| **方案 B: 二次 vicinae dmenu** | 用 dmenu 的模糊搜索框作为"伪输入"（搜索即输入） | 🟡 不够直观 |
| **方案 C: Qt 对话框** | 用 zenity/kdialog 弹出输入对话框 | 🟢 备选 |
| **方案 D: vicinae 内嵌输入框** | 利用 vicinae 搜索框作为输入（通过 `--query` 参数预设） | ⭐ 推荐（原生体验） |

**推荐**: 方案 A (wofi) + 方案 D (vicinae 搜索框) 结合使用。

### 4.2 通知反馈

```
vicinae 不支持原生 toast → 用 notify-send (libnotify) 发送桌面通知
```

### 4.3 错误处理

- 所有脚本检测 `starcatch` 命令是否存在
- 数据库操作失败时通过 notify-send 显示错误
- 管道输入为空时友好提示

---

## 5. 验收标准

- [ ] Phase 1: 按快捷键 `Ctrl+Shift+C` 弹出捕获菜单
- [ ] Phase 1: 选择「新增待办」后输入文本，成功写入数据库
- [ ] Phase 1: 操作完成后显示桌面通知确认
- [ ] Phase 2: 查看待办列表，选中后可标记完成
- [ ] Phase 2: 查看灵感/日志列表
- [ ] Phase 3: 自然语言快速输入（可选）
- [ ] 所有脚本通过 `vicinae script check` 验证
- [ ] 不影响现有 starcatch CLI 功能

---

## 6. 未纳入范围

- 不修改 Starcatch Rust 源码（仅使用现有的 CLI 接口）
- 不修改 Vicinae 本身
- 不实现 Qt GUI 的 vicinae 集成（那是独立的 C++ 项目）

---

## 7. 附录

### 参考命令

```bash
# Vicinae dmenu 基本用法
echo -e "选项1\n选项2\n选项3" | vicinae dmenu --placeholder "选择..."

# Vicinae 脚本模板生成
vicinae script template --title "xxx" --lang bash --mode compact

# 脚本验证
vicinae script check ~/.config/vicinae/scripts/starcatch-capture

# Starcatch pipe 模式
echo "内容" | starcatch pipe todo
echo "内容" | starcatch pipe idea
echo "内容" | starcatch pipe log

# 桌面通知
notify-send "Starcatch" "✅ 已捕获: 买牛奶"
```
