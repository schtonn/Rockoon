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

- 开始页：选择、验证、记住游戏目录。
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

## 尚待迁移

游戏下载与补丁安装、BRP/深链、游戏设置与排行榜、配置方案、天空盒/材质/音效预览、
自动更新、TAS 编辑器和 macOS Wine 均未迁移。原版素材检查与 TAS 自动播放提示也尚未接入。
本版本不自动安装 RockoonIO，也不沿用上游自动更新端点。

不应以当前原生入口作为旧版全部功能的替代品。

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
