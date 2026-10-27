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
