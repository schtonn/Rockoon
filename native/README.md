# Rockoon 原生前端

这是正在迁移的 Windows 原生版本，使用 Rust + Iced 0.14 的 tiny-skia 软件渲染。
这个入口不使用 Tauri、WebView、JavaScript 或 wgpu；无需 Node.js 或 WebView2 即可运行。
旧版启动器仍在原目录中，功能迁移完成前独立维护。

## 运行

安装 Rust stable（最低 1.88）和 Visual Studio C++ Build Tools 后，在仓库根目录运行：

```powershell
cargo run --manifest-path native/Cargo.toml
cargo build --release --manifest-path native/Cargo.toml
```

可执行文件位于 `native/target/release/rockoon-native.exe`，可以单独复制使用。
也可以把游戏根目录作为第一个参数传入；这种方式不保存或替换上次选择：

```powershell
.\rockoon-native.exe "D:\Games\Ballance"
```

## 本批迁移

- 游戏页：选择、验证、记住游戏目录。
- 从 `Bin` 工作目录启动 `Player.exe`，原生查询进程，阻止已有游戏运行时重复启动。
- 地图页：递归扫描已解压的 NMO/CMO 文件，支持中文、搜索与刷新。
- 启动指定地图：要求实例中已启用 BML+ 和 `ModLoader/Mods/RockoonIO.bmodp`。
- 模组页：浏览 BMOD/BMODP/ZIP，按既有 `.disable` 约定启停文件。
  已有同名目标文件时拒绝覆盖，游戏运行时拒绝更改模组。
- 独立的用户配置文件；不修改旧版 WebView 的 localStorage，不自动写游戏数据库。
- 首页直接读取所选游戏目录中的天空盒和原版菜单按钮图集，不再附带重绘插画。
- 图片缺失或损坏时保留纯色界面和普通按钮，不生成替代图。
- 读取目录、启动游戏、保存配置等操作通过后台任务执行；
  只有本程序启动的游戏在运行时才每秒检查一次进程状态。

## 游戏素材来源

素材从所选实例的以下文件读取，后台解码后由原生软件渲染器显示：

- 背景：`Textures/Sky/Sky_A_Front.bmp`。
- 主按钮：`Textures/Button01_select.tga`；缺失时尝试 `Button01_deselect.tga`。
- 按钮使用游戏图集左上角的 256 × 64 像素区域，保留图集原有像素与透明度。
- 安装目录中如果使用了替换纹理，启动器也会显示那份纹理；程序不修改游戏素材。

预览中的三个素材文件均从仓库既有下载来源的原版游戏包取得，并与
`src-tauri/resources/vanilla/manifest.json` 中的 SHA-256 一致。
程序不打包这些游戏文件，也不包含自动下载图片或重绘的回退图。

## 与旧版的功能对照

对照基线为本仓库原有 release 提交 6160dad4a0cacdc44100322a6f9fe446c742466a。
以下依据页面、服务和后端实际代码，而非只看旧 README 的功能介绍。
当前原生版是基础可用的迁移版本，尚不能替代旧版的全部功能。

| 功能 | 原生版现状与缺口 | 旧版代码依据 |
| --- | --- | --- |
| 游戏目录 | 已有选择、验证、记住目录、在资源管理器打开；未读取旧版保存的设置 | [app.rs](src/app.rs)、[旧存储结构](../src/types/store.d.ts) |
| 游戏启动与进程 | 已有启动及防止重复运行；缺少停止游戏、启动后隐藏和结束后显示窗口、累计时长和最近游玩记录 | [launcher.ts](../src/services/launcher.ts) |
| 指定地图启动 | 部分支持；要求预先安装 BML+ 和 RockoonIO，缺少自动安装 RockoonIO、地图模式/关卡槽位选择和 TAS 自动播放提醒。目前固定 ROCKOON_MAP_ONLY=1、ROCKOON_LEVEL_SLOT=0 | [launcher.ts](../src/services/launcher.ts)、[app.ts](../src/stores/app.ts)、[instance.ts](../src/backend/instance.ts) |
| 地图管理 | 已有递归列表、简单搜索、刷新和启动；缺少导入、目录浏览/面包屑、新建目录、移动、删除。搜索尚未实现旧版的去扩展名、多关键词匹配 | [ResourcesMaps.tsx](../src/views/ResourcesMaps.tsx)、[search.ts](../src/utils/search.ts) |
| 模组管理 | 已有列表、简单搜索、刷新、启停；缺少导入、删除和跳转对应配置文件 | [ResourcesPageBase.tsx](../src/views/ResourcesPageBase.tsx) |
| 游戏设置 | 未迁移：分辨率、窗口/全屏、音量、垂直同步、按键、相机、云层等，以及 NewPlayer 的语言、跳过开场、帧率/宽屏解锁等设置 | [GameConfig.tsx](../src/views/GameConfig.tsx) |
| 游戏存档 | 未迁移：关卡解锁、排行榜玩家名和分数编辑、重置 | [GameData.tsx](../src/views/GameData.tsx) |
| 模组配置 | 未迁移：CFG/INI 配置搜索、按类型编辑、保存和打开配置目录 | [ModConfigs.tsx](../src/views/ModConfigs.tsx) |
| 配置方案 | 未迁移：创建、重命名、切换、删除游戏及模组配置方案 | [profiles.ts](../src/stores/profiles.ts) |
| 游戏与补丁安装 | 未迁移：下载原版游戏，安装/更新 BML、BML+、NewPlayer，检查依赖与安装记录 | [Patches.tsx](../src/views/Patches.tsx)、[game.ts](../src/services/game.ts)、[patches.ts](../src/services/patches.ts) |
| 资源包导入 | 未迁移：BRP/ZIP 文件选择和拖放、rockoon://import 深链、依赖解析、模组版本兼容处理及天空盒目标关卡选择 | [resourceImport.ts](../src/services/resourceImport.ts)、[brp.ts](../src/services/brp.ts)、[deepLinkImport.ts](../src/services/deepLinkImport.ts) |
| 下载任务 | 未迁移：统一任务列表、进度/速度、可取消任务的取消、失败重试和清理已完成项 | [downloads.ts](../src/stores/downloads.ts)、[Downloads.tsx](../src/views/Downloads.tsx) |
| 天空盒管理 | 未迁移：按关卡分组、缩略图/天空盒预览、导入和目标关卡选择；首页显示天空背景不等于资源管理 | [ResourcesSkys.tsx](../src/views/ResourcesSkys.tsx) |
| 材质管理 | 未迁移：BMP/TGA 浏览、预览和导入；首页读取菜单图片不等于材质管理 | [ResourcesTextures.tsx](../src/views/ResourcesTextures.tsx) |
| 音效管理 | 未迁移：WAV 列表、播放/停止和导入 | [ResourcesSounds.tsx](../src/views/ResourcesSounds.tsx) |
| 原版文件检查/恢复 | 未迁移：启动前 NMO 校验及差异提醒、原版素材恢复；旧版恢复会保留额外文件 | [vanilla.rs](../src-tauri/src/commands/vanilla.rs)、[Patches.tsx](../src/views/Patches.tsx) |
| TAS | 未迁移：独立编辑器、时间轴编辑/导入导出/撤销重做、游戏窗口嵌入与回放速度控制。旧版为 Windows 可选实验功能 | [TasEditorWindow.tsx](../src/views/tas/TasEditorWindow.tsx)、[tasWindow.ts](../src/services/tasWindow.ts) |
| 启动器偏好 | 目前只保存游戏目录；缺少中英切换、启动行为、地图启动模式/槽位、游戏内 MOTD 等。主题/背景效果属于简化设计中主动省去的外观功能 | [Settings.tsx](../src/views/Settings.tsx)、[store.d.ts](../src/types/store.d.ts) |
| 自动更新 | 未迁移：版本检查、更新说明、下载及安装；原生版没有沿用上游更新端点 | [updater.ts](../src/services/updater.ts) |
| 系统集成与平台 | 缺少启动器单实例唤起、协议注册与处理，以及 macOS 实验性 Wine 安装/配置/运行支持；当前原生版面向 Windows | [lib.rs](../src-tauri/src/lib.rs)、[wine.ts](../src/services/wine.ts) |
| 社区入口 | 未迁移：百科、制图手册、论坛、下载站等外部链接 | [menu.ts](../src/routers/menu.ts) |

### 对照边界

- 旧 README 的“多实例”与实际数据模型不完全一致：代码保存一个当前游戏目录，另有多套配置方案。原生版缺少的是配置方案管理，不能直接描述为删除了完整的多实例管理器。
- 旧 README 提到的资源上传、作者管理没有对应的桌面内置页面；外部资源站能力不计为本次删除的桌面功能。
- 旧版存在“退出启动器时结束游戏”的偏好字段，但未找到执行它的逻辑，因此没有计入已实现功能。
- 删除灰色辅助小字、冗余标题、轮播文案和装饰效果是界面简化，不等于上述业务功能已经迁移。
- 游戏页不再显示“开始”入口或标题；点击左上角 ROCKOON 返回游戏页。

### 建议迁移顺序

1. 游戏/补丁安装、RockoonIO 自动安装、BRP 导入和下载任务：先补齐从安装到游玩的完整流程。
2. 游戏设置、模组配置和配置方案：补齐日常调整能力。
3. 地图/模组完整管理、天空盒/材质/音效管理、存档编辑与原版恢复。
4. 自动更新及系统集成；TAS 与 macOS Wine 作为独立迁移项目。

## 验证

```powershell
cargo fmt --manifest-path native/Cargo.toml --check
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path native/Cargo.toml
```

测试覆盖目录校验、中文/嵌套/大小写资源扫描、模组启停无损、
同名冲突拒绝覆盖、实例外路径拒绝、启动工作目录和中文地图环境变量。CI 的原生工作流单独构建并提供 Windows EXE，
不创建应用 release。

源码分为 `app.rs`（界面与任务状态）、`game.rs`（实例与进程/资源操作）、
`preferences.rs`（原子配置保存）、`style.rs`（颜色与控件样式）。
后续先把原 Rust 解析和安装模块抽为无 Tauri 依赖的共享核心，再接到此界面。
