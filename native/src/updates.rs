use crate::workspace::error;
use serde_json::Value;
#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub url: String,
    pub sha256: String,
}
pub fn check() -> Result<Option<Release>, String> {
    let response = ureq::get("https://api.github.com/repos/schtonn/Rockoon/releases?per_page=20")
        .set("User-Agent", "RockoonNative")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(error)?;
    let releases: Value = response.into_json().map_err(error)?;
    for release in releases.as_array().ok_or("发布数据格式异常")? {
        if release["draft"] == true || release["prerelease"] == true {
            continue;
        }
        let tag = release["tag_name"].as_str().unwrap_or("");
        let Some(version) = tag.strip_prefix("native-v") else {
            continue;
        };
        let components = |s: &str| {
            s.split('.')
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        if components(version) <= components(env!("CARGO_PKG_VERSION")) {
            continue;
        }
        let Some(assets) = release["assets"].as_array() else {
            continue;
        };
        #[cfg(windows)]
        let expected = "Rockoon-native-windows-x64.exe";
        #[cfg(not(windows))]
        let expected = "Rockoon-native-macos-universal.tar.gz";
        if let Some(asset) = assets.iter().find(|a| a["name"] == expected) {
            let url = asset["browser_download_url"]
                .as_str()
                .ok_or("更新下载地址缺失")?;
            if !url.starts_with("https://github.com/schtonn/Rockoon/releases/download/") {
                return Err("更新地址不属于此 fork".into());
            }
            let hash = asset["digest"]
                .as_str()
                .and_then(|s| s.strip_prefix("sha256:"))
                .ok_or("发布资源缺少 SHA-256，无法自动更新")?;
            if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("更新哈希格式异常".into());
            }
            return Ok(Some(Release {
                version: version.into(),
                notes: release["body"].as_str().unwrap_or("").into(),
                url: url.into(),
                sha256: hash.to_lowercase(),
            }));
        }
    }
    Ok(None)
}
