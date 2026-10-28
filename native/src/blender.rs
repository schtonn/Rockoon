//! Adapted from Ghomist/Rockoon c0525ee (2.7.2), without Tauri.
//! 制图工具：Blender + BBP（Ballance Blender Plugin，yyc12345/BallanceBlenderHelper）。
//!
//! - Blender 不代装：只找本机已经装好的（注册表卸载项 / 常见目录 / PATH），用户也可以手选；
//! - BBP 一键安装：前端用 `start_patch_install` 把 GitHub release 的 zip 解压进 Blender 的
//!   `extensions/user_default/bbp_ng`（位置由 `blender_probe` 问 Blender 得到），再调
//!   `blender_configure_bbp` 启用插件、把材质目录指向当前游戏的 Textures。
//!   不用 `blender --command extension install-file`：BBP 已启用时它会把 BMap.dll 载进
//!   执行命令的那个 Blender 进程，覆盖安装直接报 `BMap operation failed`；
//! - 一键编辑：`blender_open_map` 起 GUI 版 Blender，跑 open_map.py 清场景 + 用 BBP 导入地图。
//!
//! 跟 Blender 打交道全靠 `blender -b --python <脚本>`，脚本内嵌在二进制里，用时写到临时目录；
//! 脚本往 stdout 打一行 `ROCKOON_…` 作为结果。

use rockoon_core::common::exception::{RcError, RcResult, RcResultWith};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

static OPERATION: std::sync::Mutex<()> = std::sync::Mutex::new(());

const PROBE_PY: &str = include_str!("../resources/blender/probe.py");
const CONFIGURE_PY: &str = include_str!("../resources/blender/configure.py");
const OPEN_MAP_PY: &str = include_str!("../resources/blender/open_map.py");

const BBP_RELEASE_API: &str =
    "https://api.github.com/repos/yyc12345/BallanceBlenderHelper/releases/latest";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BbpModule {
    /// `bl_ext.user_default.bbp_ng`（4.0+ 扩展）或旧插件的模块名
    pub module: String,
    pub version: String,
    pub enabled: bool,
    pub texture_folder: String,
    /// 插件所在目录（提示用户手动删旧版时用）
    pub path: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlenderProbe {
    pub version: String,
    /// user_default 扩展仓库的目录，BBP 解压到它下面的 bbp_ng
    pub extensions_dir: String,
    pub bbp: Vec<BbpModule>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BbpRelease {
    pub tag: String,
    pub asset_name: String,
    pub url: String,
    pub size: u64,
}

fn other(msg: impl Into<String>) -> RcError {
    RcError::Other(msg.into())
}

/// 用户可能选的是 Blender.app（macOS）或 blender-launcher.exe（Windows，它会脱离控制台、拿不到输出），
/// 统一换成真正的可执行文件。
fn resolve_exe(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("app")) {
        return p.join("Contents").join("MacOS").join("Blender");
    }
    if p.file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("blender-launcher.exe"))
    {
        return p.with_file_name("blender.exe");
    }
    p
}

fn blender_command(exe: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(resolve_exe(exe));
    // blender.exe 是控制台程序：不加这个会闪一个黑框（GUI 窗口不受影响）
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

fn write_script(name: &str, body: &str) -> Result<PathBuf, RcError> {
    let dir = crate::preferences::directory()
        .map_err(other)?
        .join("cache/blender-scripts");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(name);
    crate::workspace::atomic(&path, body.as_bytes()).map_err(other)?;
    Ok(path)
}

/// 后台跑一个脚本，返回脚本打出的 `ROCKOON_<tag>` 行（不含前缀）。
fn run_background(exe: &str, name: &str, body: &str, args: &[&str]) -> Result<String, RcError> {
    let script = write_script(name, body)?;
    let stdout_file = tempfile::NamedTempFile::new()?;
    let stderr_file = tempfile::NamedTempFile::new()?;
    let mut child = blender_command(exe)
        .args(["-b", "--python"])
        .arg(&script)
        .arg("--")
        .args(args)
        .stdin(Stdio::null())
        .stdout(stdout_file.reopen()?)
        .stderr(stderr_file.reopen()?)
        .spawn()
        .map_err(|e| other(format!("无法启动 Blender：{e}")))?;
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > std::time::Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(other("Blender 检测超时，请关闭 Blender 后重试"));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    use std::io::Read;
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    std::fs::File::open(stdout_file.path())?
        .take(1024 * 1024)
        .read_to_end(&mut stdout_bytes)?;
    std::fs::File::open(stderr_file.path())?
        .take(1024 * 1024)
        .read_to_end(&mut stderr_bytes)?;
    let stdout = String::from_utf8_lossy(&stdout_bytes);
    if status.success()
        && let Some(line) = stdout
            .lines()
            .find_map(|l| l.trim().strip_prefix("ROCKOON_"))
    {
        return Ok(line.to_string());
    }
    let stderr = String::from_utf8_lossy(&stderr_bytes);
    let tail: Vec<&str> = stderr.lines().rev().take(5).collect();
    Err(other(format!(
        "Blender 没有返回结果（{}）{}",
        status,
        tail.into_iter().rev().collect::<Vec<_>>().join("\n")
    )))
}

/// 本机能找到的 Blender（只看文件在不在，不启动它）。
pub fn blender_find() -> Vec<String> {
    let mut found: Vec<PathBuf> = Vec::new();
    let exe = if cfg!(windows) {
        "blender.exe"
    } else {
        "blender"
    };

    #[cfg(target_os = "windows")]
    {
        found.extend(registry_install_dirs().into_iter().map(|d| d.join(exe)));
        for var in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Ok(pf) = std::env::var(var) {
                let base = Path::new(&pf).join("Blender Foundation");
                if let Ok(rd) = std::fs::read_dir(&base) {
                    found.extend(rd.flatten().map(|e| e.path().join(exe)));
                }
                found.push(
                    Path::new(&pf)
                        .join("Steam/steamapps/common/Blender")
                        .join(exe),
                );
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mut roots = vec![PathBuf::from("/Applications")];
        if let Some(home) = std::env::var_os("HOME") {
            roots.push(Path::new(&home).join("Applications"));
        }
        for root in roots {
            if let Ok(rd) = std::fs::read_dir(&root) {
                found.extend(rd.flatten().map(|e| e.path()).filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("Blender") && n.ends_with(".app"))
                }));
            }
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        found.extend(std::env::split_paths(&path).map(|d| d.join(exe)));
    }

    let mut out: Vec<String> = Vec::new();
    for p in found {
        if !resolve_exe(&p.to_string_lossy()).is_file() {
            continue;
        }
        let s = p.to_string_lossy().to_string();
        if !out.iter().any(|o| o.eq_ignore_ascii_case(&s)) {
            out.push(s);
        }
    }
    out
}

/// Windows 卸载项里 DisplayName 以 Blender 开头的 InstallLocation（官方 msi 安装会写）。
#[cfg(target_os = "windows")]
fn registry_install_dirs() -> Vec<PathBuf> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::*;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    let uninstall = wide(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall");
    let mut out = Vec::new();
    let views = [
        (HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY, RRF_SUBKEY_WOW6464KEY),
        (HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY, RRF_SUBKEY_WOW6432KEY),
        (HKEY_CURRENT_USER, 0, 0),
    ];
    for (root, view, sub_view) in views {
        let mut key: HKEY = std::ptr::null_mut();
        if unsafe { RegOpenKeyExW(root, uninstall.as_ptr(), 0, KEY_READ | view, &mut key) }
            != ERROR_SUCCESS
        {
            continue;
        }
        let mut index = 0;
        loop {
            let mut name = [0u16; 256];
            let mut len = name.len() as u32;
            let rc = unsafe {
                RegEnumKeyExW(
                    key,
                    index,
                    name.as_mut_ptr(),
                    &mut len,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            if rc != ERROR_SUCCESS {
                break;
            }
            index += 1;
            let read = |value: &str| -> Option<String> {
                let value = wide(value);
                let mut buf = [0u16; 1024];
                let mut size = (buf.len() * 2) as u32;
                let rc = unsafe {
                    RegGetValueW(
                        key,
                        name.as_ptr(),
                        value.as_ptr(),
                        RRF_RT_REG_SZ | sub_view,
                        std::ptr::null_mut(),
                        buf.as_mut_ptr().cast(),
                        &mut size,
                    )
                };
                (rc == ERROR_SUCCESS).then(|| {
                    String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)])
                })
            };
            if read("DisplayName").is_some_and(|n| n.starts_with("Blender"))
                && let Some(dir) = read("InstallLocation").filter(|d| !d.is_empty())
            {
                out.push(PathBuf::from(dir));
            }
        }
        unsafe { RegCloseKey(key) };
    }
    out
}

/// 启动一次 Blender（后台，约 1 秒）看版本、BBP 装没装 / 启用没 / 材质目录。
pub fn blender_probe(exe: String) -> RcResultWith<BlenderProbe> {
    let _guard = OPERATION.lock().map_err(|e| other(e.to_string()))?;
    probe_unlocked(exe)
}
fn probe_unlocked(exe: String) -> RcResultWith<BlenderProbe> {
    let line = run_background(&exe, "probe.py", PROBE_PY, &[])?;
    let json = line
        .strip_prefix("JSON:")
        .ok_or_else(|| other(format!("Blender 返回了意外的结果：{line}")))?;
    serde_json::from_str(json).map_err(|e| other(format!("解析 Blender 返回失败：{e}")))
}

/// 启用 BBP 并把材质目录设成 `textures`（空串表示不改），保存 Blender 偏好设置。
pub fn blender_configure_bbp(exe: String, textures: String) -> RcResult {
    let _guard = OPERATION.lock().map_err(|e| other(e.to_string()))?;
    require_closed().map_err(other)?;
    configure_unlocked(exe, textures)
}
fn configure_unlocked(exe: String, textures: String) -> RcResult {
    let line = run_background(&exe, "configure.py", CONFIGURE_PY, &[&textures])?;
    if line == "OK" {
        Ok(())
    } else {
        Err(other(line.strip_prefix("ERROR:").unwrap_or(&line)))
    }
}

/// 打开 Blender 并用 BBP 导入地图（不等它退出）。
pub fn blender_open_map(exe: String, map: String, textures: String) -> RcResult {
    let script = write_script("open_map.py", OPEN_MAP_PY)?;
    let mut cmd = blender_command(&exe);
    if let Some(dir) = Path::new(&map).parent() {
        cmd.current_dir(dir);
    }
    let mut child = cmd
        .arg("--python")
        .arg(&script)
        .arg("--")
        .arg(&map)
        .arg(&textures)
        // GUI 进程长期存活，别让它继承我们的标准输入输出
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| other(format!("无法启动 Blender：{e}")))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// GitHub 上 BBP 的最新 release，挑出当前系统对应的 zip。
pub fn bbp_latest_release() -> RcResultWith<BbpRelease> {
    let platform = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        _ => "linux",
    };
    let body = crate::jobs::json_url(BBP_RELEASE_API).map_err(other)?;
    let tag = body["tag_name"].as_str().unwrap_or_default().to_string();
    let asset = body["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| {
            a["name"].as_str().is_some_and(|n| {
                n.contains(platform)
                    && n.contains(if std::env::consts::ARCH == "aarch64" {
                        "arm64"
                    } else {
                        "amd64"
                    })
                    && n.ends_with(".zip")
            })
        })
        .ok_or_else(|| other(format!("BBP {tag} 没有 {platform} 版本的安装包")))?;
    Ok(BbpRelease {
        tag,
        asset_name: asset["name"].as_str().unwrap_or_default().to_string(),
        url: asset["browser_download_url"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        size: asset["size"].as_u64().unwrap_or(0),
    })
}

pub const MODULE: &str = "bl_ext.user_default.bbp_ng";
pub fn package_key() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("bbp-windows"),
        ("macos", "aarch64") => Ok("bbp-macos"),
        _ => Err("下载站暂无适合当前系统架构的 BBP 安装包".into()),
    }
}
pub fn latest_release() -> Result<BbpRelease, String> {
    let key = package_key()?;
    let from_hub = || -> Result<BbpRelease, String> {
        let list = crate::jobs::hub("/packages?category=x-patch&limit=100")?;
        let package = list["items"]
            .as_array()
            .and_then(|items| items.iter().find(|p| p["source_key"] == key))
            .ok_or("下载站没有当前系统的 BBP")?;
        let id = package["id"].as_u64().ok_or("BBP 资源编号缺失")?;
        let versions = crate::jobs::hub(&format!("/packages/{id}/versions"))?;
        let version = versions["items"]
            .as_array()
            .and_then(|items| items.iter().find(|v| v["is_current"] == true))
            .ok_or("BBP 当前版本缺失")?;
        let number = version["version"].as_u64().ok_or("BBP 文件序号缺失")?;
        Ok(BbpRelease {
            tag: version["note"].as_str().unwrap_or("最新版本").into(),
            asset_name: version["file_name"].as_str().unwrap_or("BBP.zip").into(),
            url: format!(
                "{}/packages/{id}/versions/{number}/download",
                crate::jobs::origin()
            ),
            size: version["file_size"].as_u64().unwrap_or(0),
        })
    };
    from_hub().or_else(|_| bbp_latest_release().map_err(crate::workspace::error))
}
pub fn version_at_least(actual: &str, minimum: &str) -> bool {
    fn numbers(v: &str) -> [u32; 3] {
        let mut parts = v.trim_start_matches(['v', 'V']).split('.');
        std::array::from_fn(|_| parts.next().and_then(|v| v.parse().ok()).unwrap_or(0))
    }
    numbers(actual) >= numbers(minimum)
}
pub fn require_closed() -> Result<(), String> {
    let name = if cfg!(windows) {
        "blender.exe"
    } else {
        "Blender"
    };
    if !rockoon_platform::process::find_processes(name.into())
        .map_err(crate::workspace::error)?
        .is_empty()
    {
        return Err("请先关闭 Blender，再安装或更新 BBP".into());
    }
    Ok(())
}
fn manifest_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim().trim_matches(['\"', '\'']))
}
fn plugin_entries(
    archive: &Path,
    blender_version: &str,
) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let mut entries = crate::workspace::archive_entries(archive, false)?;
    let manifest_path = entries
        .iter()
        .find(|(p, _)| p.file_name().is_some_and(|f| f == "blender_manifest.toml"))
        .map(|(p, _)| p.clone())
        .ok_or("安装包没有 Blender 扩展清单")?;
    let prefix = manifest_path.parent().unwrap_or(Path::new(""));
    for (path, _) in &mut entries {
        *path = path
            .strip_prefix(prefix)
            .map_err(|_| "BBP 安装包目录结构不正确")?
            .to_owned();
    }
    let manifest = entries
        .iter()
        .find(|(p, _)| p == Path::new("blender_manifest.toml"))
        .unwrap();
    let text = std::str::from_utf8(&manifest.1).map_err(crate::workspace::error)?;
    if manifest_value(text, "id") != Some("bbp_ng")
        || !entries.iter().any(|(p, _)| p == Path::new("__init__.py"))
    {
        return Err("这个安装包不是 BBP 扩展".into());
    }
    let minimum =
        manifest_value(text, "blender_version_min").ok_or("扩展清单没有 Blender 最低版本")?;
    if !version_at_least(blender_version, minimum) {
        return Err(format!("这个 BBP 版本需要 Blender {minimum} 或更新版本"));
    }
    Ok(entries)
}
/// Validate and stage the complete extension, keeping the previous plugin if activation fails.
pub fn install_archive(
    extensions: &Path,
    archive: &Path,
    exe: &str,
    textures: &str,
) -> Result<(), String> {
    let _guard = OPERATION.lock().map_err(crate::workspace::error)?;
    require_closed()?;
    let probe = probe_unlocked(exe.into()).map_err(crate::workspace::error)?;
    if probe.extensions_dir.is_empty()
        || std::path::absolute(&probe.extensions_dir).map_err(crate::workspace::error)?
            != std::path::absolute(extensions).map_err(crate::workspace::error)?
    {
        return Err("Blender 扩展目录已变化，请重新检测后安装".into());
    }
    if probe.bbp.iter().any(|p| p.module != MODULE) {
        return Err("检测到旧版 BBP，请先在 Blender 中卸载旧版插件".into());
    }
    let entries = plugin_entries(archive, &probe.version)?;
    replace_plugin(extensions, entries, || {
        configure_unlocked(exe.into(), textures.into()).map_err(crate::workspace::error)
    })
}
fn replace_plugin(
    extensions: &Path,
    entries: Vec<(PathBuf, Vec<u8>)>,
    activate: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    std::fs::create_dir_all(extensions).map_err(crate::workspace::error)?;
    let staging = tempfile::Builder::new()
        .prefix(".rockoon-bbp-")
        .tempdir_in(extensions)
        .map_err(crate::workspace::error)?;
    let staged = staging.path().join("bbp_ng");
    std::fs::create_dir(&staged).map_err(crate::workspace::error)?;
    for (path, bytes) in entries {
        let target = crate::workspace::checked(&staged, &staged.join(path))?;
        crate::workspace::atomic(&target, &bytes)?;
    }
    let target = crate::workspace::checked(extensions, &extensions.join("bbp_ng"))?;
    let backup = crate::workspace::checked(
        extensions,
        &extensions
            .join(".rockoon-bbp-backups")
            .join(crate::workspace::id()),
    )?;
    let existed = target.exists();
    if existed {
        std::fs::create_dir_all(backup.parent().ok_or("扩展备份路径无效")?)
            .map_err(crate::workspace::error)?;
        std::fs::rename(&target, &backup).map_err(|e| format!("请关闭 Blender 后重试：{e}"))?;
    }
    if let Err(error) = std::fs::rename(&staged, &target) {
        if existed {
            let _ = std::fs::rename(&backup, &target);
        }
        return Err(error.to_string());
    }
    if let Err(error) = activate() {
        std::fs::rename(&target, &staged).map_err(|e| format!("{error}，回退失败：{e}"))?;
        if existed {
            std::fs::rename(&backup, &target).map_err(crate::workspace::error)?;
        }
        return Err(format!("BBP 未能启用，已回退：{error}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn archive(root: &Path, id: &str) -> PathBuf {
        use std::io::Write;
        let path = root.join("bbp.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        for (name, bytes) in [
            (
                "blender_manifest.toml",
                format!("id = \"{id}\"\nblender_version_min = \"4.5.0\"\n"),
            ),
            ("__init__.py", "# plugin".into()),
        ] {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(bytes.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        path
    }
    #[test]
    fn plugin_identity_and_minimum_blender_version_are_validated_before_installation() {
        let root = tempfile::tempdir().unwrap();
        let path = archive(root.path(), "bbp_ng");
        assert_eq!(plugin_entries(&path, "4.5.0").unwrap().len(), 2);
        assert!(
            plugin_entries(&path, "4.2.9")
                .unwrap_err()
                .contains("4.5.0")
        );
        let path = archive(root.path(), "different_plugin");
        assert!(plugin_entries(&path, "5.0.0").is_err());
    }
    #[test]
    fn extension_replacement_rolls_back_on_activation_failure_and_backs_up_on_success() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("bbp_ng");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("old.dll"), b"old").unwrap();
        let entries = || vec![(PathBuf::from("new.dll"), b"new".to_vec())];
        assert!(
            replace_plugin(root.path(), entries(), || {
                assert_eq!(std::fs::read(target.join("new.dll")).unwrap(), b"new");
                Err("activation failed".into())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(target.join("old.dll")).unwrap(), b"old");
        assert!(!target.join("new.dll").exists());
        replace_plugin(root.path(), entries(), || Ok(())).unwrap();
        assert!(!target.join("old.dll").exists());
        assert_eq!(std::fs::read(target.join("new.dll")).unwrap(), b"new");
        assert!(
            std::fs::read_dir(root.path().join(".rockoon-bbp-backups"))
                .unwrap()
                .flatten()
                .any(|e| e.path().join("old.dll").is_file())
        );
    }
    #[test]
    fn versions_compare_with_missing_components_and_v_prefix() {
        assert!(version_at_least("4.5.1", "4.5"));
        assert!(version_at_least("v4.4", "4.4.0"));
        assert!(!version_at_least("4.2.9", "4.5"));
    }
    #[test]
    #[ignore = "read-only validation of a separately downloaded real BBP archive"]
    fn validate_real_bbp_archive() {
        let path = std::env::var("ROCKOON_BBP_ARCHIVE").unwrap();
        let entries = plugin_entries(Path::new(&path), "4.5.0").unwrap();
        assert!(entries.len() > 100);
        assert!(
            entries
                .iter()
                .any(|(p, _)| p.extension().is_some_and(|s| s == "dll"))
        );
    }
}
