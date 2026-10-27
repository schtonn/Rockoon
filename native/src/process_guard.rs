//! Protect files used by the selected game, irrespective of who launched it.
use std::path::Path;

#[cfg(windows)]
fn path_key(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    let value = path.to_string_lossy().replace('\\', "/");
    let value = value.strip_prefix("//?/").unwrap_or(&value);
    let value = value
        .strip_prefix("UNC/")
        .map(|s| format!("//{s}"))
        .unwrap_or_else(|| value.into());
    value.to_lowercase()
}

#[cfg(windows)]
pub fn running(root: &Path) -> Result<bool, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
        },
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
        },
    };
    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    let target = path_key(&root.join("Bin/Player.exe"));
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!(
                "无法检查游戏进程：{}",
                std::io::Error::last_os_error()
            ));
        }
        let snapshot = Handle(snapshot);
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut valid = Process32FirstW(snapshot.0, &mut entry);
        if valid == 0 {
            return Err(format!(
                "无法读取游戏进程列表：{}",
                std::io::Error::last_os_error()
            ));
        }
        let mut unreadable = false;
        while valid != 0 {
            let length = entry
                .szExeFile
                .iter()
                .position(|&v| v == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..length])
                .eq_ignore_ascii_case("Player.exe")
            {
                let process =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                if process.is_null() {
                    // The process may have exited between the snapshot and OpenProcess.
                    if std::io::Error::last_os_error().raw_os_error()
                        != Some(ERROR_INVALID_PARAMETER as i32)
                    {
                        unreadable = true;
                    }
                } else {
                    let process = Handle(process);
                    let mut name = vec![0u16; 32768];
                    let mut length = name.len() as u32;
                    if QueryFullProcessImageNameW(process.0, 0, name.as_mut_ptr(), &mut length) != 0
                    {
                        let image = OsString::from_wide(&name[..length as usize]);
                        if path_key(Path::new(&image)) == target {
                            return Ok(true);
                        }
                    } else {
                        let mut exit_code = 0;
                        if GetExitCodeProcess(process.0, &mut exit_code) == 0 || exit_code == 259 {
                            unreadable = true;
                        }
                    }
                }
            }
            valid = Process32NextW(snapshot.0, &mut entry);
        }
        let failure = std::io::Error::last_os_error();
        if failure.raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
            return Err(format!("无法完成游戏进程检查：{failure}"));
        }
        if unreadable {
            Err("无法读取某个 Player.exe 的运行目录，暂时无法确认当前游戏目录是否正在使用".into())
        } else {
            Ok(false)
        }
    }
}

#[cfg(any(not(windows), test))]
fn command_matches(command: &str, executable: &str) -> bool {
    let command = command.replace('\\', "/").to_lowercase();
    let executable = executable.replace('\\', "/").to_lowercase();
    command.match_indices(&executable).any(|(index, _)| {
        let before = command[..index].chars().next_back();
        let after = command[index + executable.len()..].chars().next();
        let boundary =
            |c: Option<char>| c.is_none_or(|c| c.is_whitespace() || c == '"' || c == '\'');
        boundary(before) && boundary(after)
    })
}

#[cfg(not(windows))]
pub fn running(root: &Path) -> Result<bool, String> {
    let executable = std::path::absolute(root)
        .map_err(|e| e.to_string())?
        .join("Bin/Player.exe");
    let canonical = executable
        .canonicalize()
        .unwrap_or_else(|_| executable.clone());
    let mut candidates = Vec::new();
    for path in [executable, canonical] {
        let path = path.to_string_lossy().into_owned();
        candidates.push(rockoon_platform::wine::to_windows_path(&path));
        candidates.push(path);
    }
    let output = std::process::Command::new("ps")
        .args(["-axo", "stat=,command="])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("无法检查当前游戏目录的运行进程".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).lines().any(|line| {
        let Some((state, command)) = line.trim_start().split_once(char::is_whitespace) else {
            return false;
        };
        !state.starts_with('Z')
            && candidates
                .iter()
                .any(|path| command_matches(command.trim(), path))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wine_command_matches_only_the_selected_executable() {
        let executable = "/Users/me/My Games/Ballance/Bin/Player.exe";
        assert!(command_matches(
            &format!("wine \"{executable}\" -w 512"),
            executable
        ));
        assert!(command_matches(
            "Z:\\Users\\me\\My Games\\Ballance\\Bin\\PLAYER.EXE -w 512",
            "Z:\\Users\\me\\My Games\\Ballance\\Bin\\Player.exe"
        ));
        assert!(!command_matches(&format!("{executable}.bak"), executable));
        assert!(!command_matches(&format!("/other{executable}"), executable));
        assert!(!command_matches(
            "/Users/me/My Games/Other/Bin/Player.exe",
            executable
        ));
    }

    #[cfg(windows)]
    #[test]
    fn foreign_game_directory_does_not_block_current_files() {
        use std::os::windows::process::CommandExt;
        use std::{
            fs,
            process::{Child, Command, Stdio},
        };
        struct Owned(Child);
        impl Drop for Owned {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let other = tempfile::tempdir().unwrap();
        let current = tempfile::tempdir().unwrap();
        fs::create_dir(other.path().join("Bin")).unwrap();
        let image = other.path().join("Bin/Player.exe");
        let windows = std::env::var_os("SystemRoot").unwrap();
        fs::copy(Path::new(&windows).join("System32/ping.exe"), &image).unwrap();
        let _owned = Owned(
            Command::new(&image)
                .args(["-n", "30", "127.0.0.1"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(0x08000000)
                .spawn()
                .unwrap(),
        );
        assert!(running(other.path()).unwrap());
        assert!(!running(current.path()).unwrap());
        crate::workspace::folder(current.path(), current.path(), "allowed").unwrap();
        assert!(crate::workspace::folder(other.path(), other.path(), "blocked").is_err());
        assert!(!other.path().join("blocked").exists());
    }
}
