use std::path::Path;
#[cfg(windows)]
#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(sound: *const u16, module: *mut std::ffi::c_void, flags: u32) -> i32;
}
pub fn play(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        if unsafe { PlaySoundW(path.as_ptr(), std::ptr::null_mut(), 0x00020003) } == 0 {
            Err("无法播放此 WAV 文件".into())
        } else {
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        stop();
        let program = if cfg!(target_os = "macos") {
            "afplay"
        } else {
            "aplay"
        };
        let child = std::process::Command::new(program)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        *AUDIO.lock().map_err(|e| e.to_string())? = Some(child);
        Ok(())
    }
}
pub fn stop() {
    #[cfg(not(windows))]
    if let Ok(mut audio) = AUDIO.lock() {
        if let Some(mut child) = audio.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    #[cfg(windows)]
    unsafe {
        PlaySoundW(std::ptr::null(), std::ptr::null_mut(), 0);
    }
}

#[cfg(not(windows))]
static AUDIO: std::sync::LazyLock<std::sync::Mutex<Option<std::process::Child>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));
