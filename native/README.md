# Rockoon 原生桌面版

Rust + Iced 0.14，使用 tiny-skia 软件渲染。运行不依赖 Tauri、WebView2、JavaScript 或 wgpu。
当前版本为 0.2.0；Windows 是本机验证平台，macOS 继续作为实验平台。

## 界面

- 左侧仅保留 Ballance、资源、配置、下载、设置；资源分地图/模组/素材，配置分参数/存档/方案，下载分任务/安装。
- Ballance 页的原版天空纹理铺满内容区高度；游玩按钮直接使用游戏原版菜单图集，START 字样从该目录的原版 Font_1.tga 字库提取。
- 右上角文件夹图标展开“打开目录 / 更换目录”，后面显示当前游戏目录。
- 按钮有悬停与按下反馈；START 使用原版正常/选中纹理，140ms 平滑切换。动画由控件按需绘制，不触发整个页面逐帧重建，静止后不再请求刷新。
- 输入框、下拉框、复选框、滑块与滚动条使用一致的暖色细节；输入焦点和菜单展开状态都有明确反馈。
- 目录菜单、资源“···”和补丁版本列表使用锚点浮层，不挤动正文；点击外部、滚动外部或 Esc 关闭。
- 普通消息在右下角悬浮约 5 秒，错误约 8 秒，开头 0.16 秒淡入、最后 0.3 秒淡出；通知清除后停止通知定时刷新。确认对话框仍等待用户选择。
- TAS 在独立原生窗口中打开，通过设置启用并打开。
- 没有辅助灰色小字、宣传卡片或重绘的游戏插画。

正文内嵌思源黑体 CN Regular，导航使用 Medium（15px）；无需在系统中安装字体。
字体来自 Adobe Source Han Sans 2.005R，原始字体、OFL 许可证和来源哈希见 native/resources/fonts/。

## 已接入的功能

| 功能 | 原生实现 |
| --- | --- |
| 启动与进程 | 游戏目录选择、验证与记忆，原生重复进程检查，启动/结束自己启动的游戏，隐藏/恢复窗口 |
| 地图 | 目录浏览与面包屑、新建目录、导入、移动、重命名、删除与撤销，多关键词搜索，指定地图启动 |
| 模组 | 导入、移动、删除与撤销，BMOD/BMODP/ZIP 启停，对应配置快捷入口 |
| 游戏配置 | 原版 Database.tdb 选项、原版键位名称选择，以及已安装 NewPlayer 的 Player.ini 分组编辑 |
| 存档 | 关卡解锁与全部解锁，按关卡编辑排行榜名字和分数，重置当前关卡排行榜 |
| 模组配置 | CFG/INI 分组、搜索、布尔与数字/文本编辑；保留注释和未编辑内容 |
| 配置方案 | 兼容 .rockoon/profiles.json；创建、切换、重命名、删除，保存模组启停与游戏/模组配置 |
| 游戏和补丁 | 安装原版游戏，安装/更新 BML、BML+、两种 NewPlayer 构建，选择补丁历史版本；BML+ 自动补装 NewPlayer，禁用冲突加载器 |
| 下载任务 | 后台进度、速度、去重、下载阶段取消、失败重试、清理完成记录；游戏安装失败可重试自己的安装目录 |
| 资源包 | BRP/ZIP 校验与导入、拖放、HTTPS/rockoon://import 链接、资源依赖解析，按加载器选择模组产物 |
| 天空/材质/音效 | 原版 BMP/TGA 预览，五面天空盒方向/俯仰预览，WAV 试听与停止，资源包天空目标关卡选择 |
| 原版保护 | 启动前 NMO 差异提醒，按内置清单恢复原版关卡，保留自定义文件；正常启动的 TAS 自动播放提醒及关闭入口 |
| RockoonIO | 地图启动或安装 BML+ 时自动安装缺失的内置模组；尊重已有 .disable 状态 |
| TAS | 独立窗口、兼容原 .tas 压缩格式、时间轴按键编辑、范围操作、复制/剪切/粘贴、插入帧、撤销/重做、导入/导出；从头/起始帧回放、帧率控制和 Win32 owner 视窗嵌入 |
| 启动器偏好 | 中英界面切换、启动隐藏、地图独立模式和槽位、MOTD、TAS 开关、共享游玩时长和最近游玩记录 |
| 系统集成 | 原生单实例与参数转发、下载站协议注册、社区外部链接 |
| 更新 | 检查此 fork 的原生发布包，统一任务下载并校验 GitHub 提供的 SHA-256；Windows 可安装后退出 |
| macOS | 复用原版 Wine 运行时、前缀、Rosetta 与重试启动器逻辑，支持已有 Wine；构建流程提供通用 .app 和原 Wine 运行时 |

上述对应基线为旧 release 提交 6160dad4a0cacdc44100322a6f9fe446c742466a，依据实际源码。
外部资源站的上传、作者管理没有对应的旧版桌面内置页面，没有作为桌面功能重新实现。
旧版实际保存一个当前游戏目录和多套配置方案，并非完整的多实例管理器。

## 构建和运行

Windows 安装 Rust stable（最低 1.88）和 Visual Studio C++ Build Tools：

~~~powershell
cargo run --manifest-path native/Cargo.toml
cargo build --release --manifest-path native/Cargo.toml
~~~

输出为 native/target/release/rockoon-native.exe，复制使用时请同时保留随包的 Rockoon-SourceHanSans-LICENSE.txt。
命令行可传入游戏根目录，不改写已保存的目录选择：

~~~powershell
.\rockoon-native.exe "D:\Games\Ballance"
~~~

可直接打开页面或独立编辑器：

~~~powershell
.\rockoon-native.exe "D:\Games\Ballance" --page=maps
.\rockoon-native.exe "D:\Games\Ballance" --tas-editor project.tas
~~~

ROCKOON_NATIVE_CONFIG_DIR 可指定配置目录，用于便携使用或隔离验收。
独立 TAS 编辑器将偏好存入该目录的 tas-editor 子目录；游玩记录通过文件锁共享。

macOS 构建参见 .github/workflows/native.yml：
分别构建 Apple Silicon/Intel，使用 lipo 合并，复用 scripts/fetch-wine.mjs，
以 Zig 编译原 src-launcher/launch.c，再执行 native/scripts/bundle-macos.py。
运行本身无需 Node.js。Wine 运行时及原始许可证作为 .app 资源随包提供。

## 素材与格式复用

首页从所选游戏目录读取：

- Textures/Sky/Sky_A_Front.bmp。
- Textures/Button01_select.tga；缺失时尝试 Button01_deselect.tga。
- Textures/Font_1.tga：按 16 × 16 网格提取 START 的 ASCII 字形，保留原始笔画、透明度与基线；不在发行包中捆绑游戏字库。
- 菜单按钮裁取图集左上角 256 × 64 像素，保留原始像素与透明度。
- 图片缺失时使用纯色和普通按钮；不生成替代插画。游戏目录采用替换纹理时会显示该目录中的素材。

预览使用的原版纹理已与 src-tauri/resources/vanilla/manifest.json 中的 SHA-256 对照。
原版文件恢复直接复用仓库已有 vanilla-core.zip 和清单，仅覆盖清单中 3D Entities 下的 NMO。
替换前保留备份，额外文件不删除。

内置 RockoonIO.bmodp 来自官方 [Rockoon 2.7.1 Windows 安装包](https://github.com/Ghomist/Rockoon/releases/tag/rockoon-v2.7.1)，
未执行安装程序，仅提取原有资源；64,000 字节，SHA-256：
1454de9b2bf6723b3eacedc41a6c03760b2c60fd76209e372af305685282424a。
源码仍在 src-bmodp/，更新模组时应一起核对来源和哈希。

native/core 直接编译旧版纯 Rust TDB/BRP/输入修复源码。
native/platform 的构建脚本仅适配原有 Win32/Wine 命令的宏、句柄与路径接口，不引入 Tauri。
原始后端和旧前端仍保留。

## 写入与兼容边界

- 修改游戏文件前按 Player.exe 的实际路径检查所选目录；其他目录的进程不阻拦，同目录的外部进程仍保护文件。结束游戏只针对本窗口持有的进程。
- Windows 无法读取进程路径时会说明无法确认目录，不把未知进程直接认作当前游戏；Wine 重建涉及共享环境，仍要求退出游戏。
- 文件替换先保留 .rockoon/backups 备份；批量写入失败会回退。删除使用 .rockoon/trash，可撤销本次删除。
- 拒绝路径越界、符号链接/目录联接、压缩包重复路径和超出限制的解压内容。
- TDB 修改保留其他表、未编辑选项和最新进度；配置文件如被其他操作修改，会拒绝覆盖并要求重新读取。
- 旧版 WebView 的 localStorage 不会自动读取；已有游戏文件、补丁版本记录和配置方案可直接读取。
- 原版 Player 不读取 Player.ini；该入口仅在检测到 NewPlayer 时出现。
- TAS 回放要求安装 BallanceTAS 2.x，生成配置后再使用；临时改动在游戏退出后恢复，崩溃后的恢复记录会保留到下次正常启动。
- TAS 已覆盖上述功能，交互为原生实现；旧编辑器的全部绘制和手势细节不逐一复制。
- 原生自更新仅识别此 fork 的 native-v* 发布与固定产物名，拒绝上游 Tauri 安装包。首次发布原生产物前不会有可用更新。
- macOS 继续实验性维护，需要实机验证 Wine、Rosetta、URL 事件及游戏窗口表现；TAS 嵌入/回放只在 Windows 提供。
- 不恢复旧版装饰背景、轮播或辅助说明文案。

## 验证

~~~powershell
cargo fmt --manifest-path native/Cargo.toml --check
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path native/Cargo.toml --workspace
cargo test --manifest-path native/Cargo.toml official_game_download_and_archive_validation -- --ignored --nocapture
~~~

普通检查覆盖格式兼容、真实原版数据库保真、配置方案切换、文件备份/回退、
中文路径与模组启停、BRP 模组产物选择、天空目标关卡、TAS 格式与编辑历史。
最后一项会实际下载官方游戏包，仅验证网络与压缩包内容，不安装或运行游戏。

Windows CI 跳过继承的 Explorer 桌面进程测试，其余测试照常执行。
工作流只上传构建产物，不创建应用 release，也不会替换 release 分支的旧版发布流程。
