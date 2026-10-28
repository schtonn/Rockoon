use std::{
    collections::HashMap,
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};
static ENGLISH: AtomicBool = AtomicBool::new(false);
static WORDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    fn visit(zh: &serde_json::Value, en: &serde_json::Value, map: &mut HashMap<String, String>) {
        if let (Some(zh), Some(en)) = (zh.as_str(), en.as_str()) {
            map.insert(zh.into(), en.into());
        } else if let Some(values) = zh.as_object() {
            for (key, value) in values {
                visit(value, &en[key], map);
            }
        }
    }
    let mut map = HashMap::new();
    let zh =
        serde_json::from_str(include_str!("../../src/i18n/languages/zh.json")).unwrap_or_default();
    let en =
        serde_json::from_str(include_str!("../../src/i18n/languages/en.json")).unwrap_or_default();
    visit(&zh, &en, &mut map);
    for (zh, en) in [
        (
            "选好游戏后，会继续引导你准备 Wine 运行环境。",
            "After choosing the game, set up Wine to run it.",
        ),
        ("准备运行环境", "Set up the game runtime"),
        (
            "Ballance 需要通过 Wine 运行，完成下面的准备后即可启动游戏。",
            "Ballance runs through Wine. Complete the setup below to play.",
        ),
        ("游戏运行环境", "Game runtime"),
        ("正在检查运行环境…", "Checking the game runtime…"),
        ("已就绪", "Ready"),
        ("自选 Wine", "Custom Wine"),
        ("内置 Wine", "Bundled Wine"),
        ("自选 Wine 无法运行", "Custom Wine cannot run"),
        ("先安装 Rosetta 2", "Install Rosetta 2 first"),
        ("初始化游戏环境", "Initialize game runtime"),
        ("请选择已有 Wine", "Choose an installed Wine"),
        ("Wine 路径", "Wine location"),
        ("选择已有 Wine", "Choose installed Wine"),
        ("使用内置 Wine", "Use bundled Wine"),
        ("打开 Wine 设置", "Open Wine settings"),
        ("安装 Rosetta 2", "Install Rosetta 2"),
        ("重新检测", "Check again"),
        ("环境检测", "Runtime check"),
        (
            "正在处理，请完成可能出现的系统窗口…",
            "Working. Complete any system dialogs that appear…",
        ),
        (
            "选择 Wine .app 或根目录",
            "Choose a Wine app or root folder",
        ),
        (
            "让 Apple 芯片运行 Wine 所需的 Intel 程序，安装时会出现系统授权窗口。",
            "Runs the Intel programs required by Wine on Apple Silicon and asks for system authorization.",
        ),
        (
            "用于运行 Windows 版 Ballance。",
            "Runs the Windows version of Ballance.",
        ),
        (
            "使用随软件附带的 Wine 或已选择的 Wine 准备游戏环境，初始化可能需要几十秒。",
            "Prepares the game runtime using bundled or selected Wine and may take tens of seconds.",
        ),
        (
            "可选择已安装的 Wine 应用或包含 bin/wine 的目录。",
            "Choose a Wine app or a folder containing bin/wine.",
        ),
        (
            "打开 Wine 自带的声音和显示设置。",
            "Opens the sound and display settings included with Wine.",
        ),
        ("启动器更新", "Launcher updates"),
        ("资源站安装链接", "Resource site install links"),
        ("关联到 Rockoon", "Link to Rockoon"),
        ("重新关联", "Relink"),
        (
            "让浏览器中的资源站安装链接打开 Rockoon。",
            "Opens browser install links in Rockoon.",
        ),
        ("选择 Ballance", "Choose Ballance"),
        (
            "选择已有的游戏文件夹，或安装一份游戏。",
            "Choose an existing game folder, or install a copy.",
        ),
        ("选择已有游戏", "Choose existing game"),
        ("下载并安装", "Download and install"),
        ("游戏设置", "Game settings"),
        ("分辨率", "Resolution"),
        ("音量", "Volume"),
        ("画面", "Graphics"),
        ("声音", "Audio"),
        ("键位与相机", "Keys and camera"),
        ("高级", "Advanced"),
        ("社区资源站 ↗", "Community resources ↗"),
        ("正在连接…", "Connecting…"),
        ("这里还没有资源", "No resources here yet"),
        ("没有匹配的资源", "No matching resources"),
        (
            "可以导入本地文件，或打开社区资源站下载。",
            "Import local files, or download from the community site.",
        ),
        (
            "在社区资源站点击“一键安装”，也可以导入本地文件。",
            "Use one-click install on the community site, or import local files.",
        ),
        ("启用一键安装", "Enable one-click install"),
        ("重新关联一键安装", "Relink one-click install"),
        (
            "使用自制地图和模组需要 BML / BML+。",
            "Custom maps and mods require BML / BML+.",
        ),
        ("安装支持", "Install support"),
        (
            "安装到新的空文件夹，完成后即可启动游戏。",
            "Install into a new empty folder. You can play when installation finishes.",
        ),
        ("兼容增强（NewPlayer）", "Compatibility (NewPlayer)"),
        ("模组与地图（BML+）", "Mods and maps (BML+)"),
        ("进入游戏", "Go to game"),
        ("查看资源", "View resources"),
        ("没有匹配的配置项", "No matching settings"),
        (
            "无法读取配置，请重新读取或选择其他配置。",
            "Unable to read settings. Reload or choose another configuration.",
        ),
        ("资源", "Resources"),
        ("下载", "Downloads"),
        ("参数", "Options"),
        ("启用 TAS 编辑器", "Enable TAS editor"),
        ("打开 TAS 编辑器", "Open TAS editor"),
        ("最新版本", "Latest version"),
        ("地图", "Maps"),
        ("模组", "Mods"),
        ("素材", "Assets"),
        ("配置", "Configuration"),
        ("存档", "Save data"),
        ("方案", "Profiles"),
        ("安装", "Install"),
        ("任务", "Tasks"),
        ("设置", "Settings"),
        ("天空盒", "Skyboxes"),
        ("材质", "Textures"),
        ("音效", "Sounds"),
        ("选择游戏目录", "Select game folder"),
        ("打开目录", "Open folder"),
        ("更换目录", "Change folder"),
        ("启动游戏", "Play"),
        ("结束游戏", "Stop game"),
        ("根目录", "Root"),
        ("新建文件夹", "New folder"),
        ("撤销删除", "Undo delete"),
        ("打开文件夹", "Open folder"),
        ("游玩", "Play"),
        ("预览", "Preview"),
        ("试听", "Listen"),
        ("停用", "Disable"),
        ("启用", "Enable"),
        ("停止试听", "Stop preview"),
        ("保存修改", "Save changes"),
        ("游戏选项", "Game options"),
        ("常规", "General"),
        ("游戏", "Game"),
        ("新建方案", "New profile"),
        ("当前", "Current"),
        ("切换", "Switch"),
        ("安装 Ballance", "Install Ballance"),
        ("版本", "Versions"),
        ("恢复原版关卡", "Restore original levels"),
        ("导入到", "Import into"),
        ("清理已完成", "Clear completed"),
        (
            "资源包 HTTPS 地址或 rockoon:// 链接",
            "Package HTTPS URL or rockoon:// link",
        ),
        ("地图槽位", "Map slot"),
        ("游戏内消息", "In-game message"),
        ("内容", "Content"),
        ("显示 TAS 编辑器", "Show TAS editor"),
        ("启动游戏后隐藏窗口", "Hide launcher after starting game"),
        ("自制地图独立模式", "Map-only mode"),
        ("检查更新", "Check for updates"),
        ("注册下载站一键安装", "Register one-click installs"),
        ("未导出", "Unsaved"),
        ("导出", "Export"),
        ("另存为", "Save as"),
        ("上一段", "Previous"),
        ("下一段", "Next"),
        ("起始帧", "First frame"),
        ("结束帧", "Last frame"),
        ("帧数", "Frames"),
        ("插入", "Insert"),
        ("语言", "Language"),
        ("外观", "Appearance"),
        ("浅色", "Light"),
        ("深色", "Dark"),
        ("全部解锁", "Unlock all"),
        ("重置排行", "Reset scores"),
        ("正在启动…", "Starting…"),
        ("暂无资源", "No resources"),
        ("读取中…", "Loading…"),
        ("搜索配置", "Search configuration"),
        ("音量（0–1）", "Volume (0–1)"),
        ("玩家名", "Player name"),
        ("关卡", "Levels"),
        ("前进", "Forward"),
        ("后退", "Backward"),
        ("左转", "Left"),
        ("右转", "Right"),
        ("旋转相机", "Rotate camera"),
        ("抬升相机", "Lift camera"),
        ("云层", "Cloud layer"),
        ("反转相机", "Invert camera"),
        ("自制地图（X）", "Custom maps (X)"),
        ("天空盒目标", "Skybox target"),
        ("百科", "Wiki"),
        ("论坛", "Forum"),
        ("制图手册", "Mapping manual"),
        ("下载站", "Downloads"),
        ("安装更新并退出", "Install update and exit"),
        ("下载更新", "Download update"),
    ] {
        map.insert(zh.into(), en.into());
    }
    map
});
pub fn set(language: &str) {
    ENGLISH.store(language == "en", Ordering::Relaxed);
}
pub fn text(value: &str) -> String {
    if ENGLISH.load(Ordering::Relaxed) {
        WORDS.get(value).cloned().unwrap_or_else(|| value.into())
    } else {
        value.into()
    }
}
