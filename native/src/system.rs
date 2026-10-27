use crate::workspace::{self, error};
use fs2::FileExt;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};
static WAKE: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);
pub fn events() -> impl iced::futures::Stream<Item = ()> {
    use iced::futures::SinkExt;
    iced::stream::channel::<()>(1, async move |mut output| {
        loop {
            WAKE.notified().await;
            if output.send(()).await.is_err() {
                break;
            }
        }
    })
}
static INBOX: LazyLock<Arc<Mutex<Vec<Vec<String>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));
pub struct Guard {
    _lock: fs::File,
    _session: PathBuf,
}
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self._session);
    }
}
pub fn start() -> Result<Option<Guard>, String> {
    let directory = crate::preferences::directory()?;
    fs::create_dir_all(&directory).map_err(error)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("instance.lock"))
        .map_err(error)?;
    let session = directory.join("session.json");
    if lock.try_lock_exclusive().is_err() {
        let args: Vec<_> = std::env::args().skip(1).collect();
        for _ in 0..20 {
            let forwarded = (|| -> Result<(), String> {
                let value: Value =
                    serde_json::from_slice(&fs::read(&session).map_err(error)?).map_err(error)?;
                let port = value["port"].as_u64().ok_or("会话端口缺失")?;
                let address: SocketAddr = format!("127.0.0.1:{port}").parse().map_err(error)?;
                let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(200))
                    .map_err(error)?;
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .map_err(error)?;
                stream
                    .write_all(
                        &serde_json::to_vec(&json!({"token":value["token"],"args":args}))
                            .map_err(error)?,
                    )
                    .map_err(error)?;
                Ok(())
            })();
            if forwarded.is_ok() {
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        return Err("已有启动器实例，但无法传递启动参数".into());
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(error)?;
    let token = workspace::id();
    workspace::atomic(
        &session,
        &serde_json::to_vec(
            &json!({"port":listener.local_addr().map_err(error)?.port(),"token":token}),
        )
        .map_err(error)?,
    )?;
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
            let mut bytes = Vec::new();
            if std::io::Read::by_ref(&mut stream)
                .take(16385)
                .read_to_end(&mut bytes)
                .is_err()
                || bytes.len() > 16384
            {
                continue;
            }
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                if value["token"] != token {
                    continue;
                }
                if let Ok(args) = serde_json::from_value::<Vec<String>>(value["args"].clone())
                    && let Ok(mut inbox) = INBOX.lock()
                    && inbox.len() < 32
                {
                    inbox.push(args);
                    WAKE.notify_one();
                }
            }
        }
    });
    Ok(Some(Guard {
        _lock: lock,
        _session: session,
    }))
}
pub fn messages() -> Vec<Vec<String>> {
    INBOX
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default()
}
/// Read-only check: opening the launcher never claims another installation's protocol handler.
pub fn protocol_registered() -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Registry::{
            HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW,
        };
        let key: Vec<u16> = r"Software\Classes\rockoon\shell\open\command"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut size = 0;
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                std::ptr::null(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if result != 0 || size == 0 || size > 65536 {
            return false;
        }
        let mut value = vec![0u16; size as usize / 2 + 1];
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                std::ptr::null(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                value.as_mut_ptr().cast(),
                &mut size,
            )
        };
        let Ok(executable) = std::env::current_exe() else {
            return false;
        };
        let actual = String::from_utf16_lossy(
            &value[..value.iter().position(|c| *c == 0).unwrap_or(value.len())],
        );
        result == 0 && actual.eq_ignore_ascii_case(&format!("\"{}\" \"%1\"", executable.display()))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.ancestors().nth(2).map(|p| p.join("Info.plist")))
            .and_then(|p| fs::read_to_string(p).ok())
            .is_some_and(|s| s.contains("rockoon"))
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        false
    }
}

pub fn register_protocol() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let executable = std::env::current_exe().map_err(error)?;
        let command = format!("\"{}\" \"%1\"", executable.display());
        for (key, name, value) in [
            (
                r"HKCU\Software\Classes\rockoon",
                "",
                "URL:Rockoon native protocol",
            ),
            (r"HKCU\Software\Classes\rockoon", "URL Protocol", ""),
            (
                r"HKCU\Software\Classes\rockoon\shell\open\command",
                "",
                command.as_str(),
            ),
        ] {
            let mut process = std::process::Command::new("reg.exe");
            process.args(["add", key, "/t", "REG_SZ", "/d", value, "/f"]);
            if name.is_empty() {
                process.arg("/ve");
            } else {
                process.args(["/v", name]);
            }
            let output = process.creation_flags(0x08000000).output().map_err(error)?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let status=std::process::Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
            .arg("-f").arg(std::env::current_exe().map_err(error)?.ancestors().nth(3).ok_or("请从 .app 启动")?).status().map_err(error)?;
        if status.success() {
            Ok(())
        } else {
            Err("注册协议失败".into())
        }
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        Err("请通过桌面环境注册 rockoon 协议".into())
    }
}
pub fn install_update(file: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let executable = std::env::current_exe().map_err(error)?;
        let directory = crate::preferences::directory()?
            .join("updates")
            .join(workspace::id());
        fs::create_dir_all(&directory).map_err(error)?;
        let script = directory.join("install.ps1");
        let quote = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "''"));
        let content = format!(
            "$ErrorActionPreference = 'Stop'\nWait-Process -Id {} -ErrorAction SilentlyContinue\nCopy-Item -LiteralPath {} -Destination {} -Force\nCopy-Item -LiteralPath {} -Destination {} -Force\n",
            std::process::id(),
            quote(&executable),
            quote(&executable.with_extension("exe.previous")),
            quote(file),
            quote(&executable)
        );
        fs::write(&script, content).map_err(error)?;
        std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-WindowStyle",
                "Hidden",
                "-File",
            ])
            .arg(script)
            .creation_flags(0x08000000)
            .spawn()
            .map_err(error)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = file;
        Err("请从下载目录手动安装此平台的更新".into())
    }
}

#[cfg(target_os = "macos")]
pub fn install_url_handler() {
    // Keep the +1 allocated Objective-C target alive for the application lifetime; only queue URL text here.
    use std::ffi::{CStr, c_char, c_void};
    type Obj = *mut c_void;
    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Obj;
        fn sel_registerName(name: *const c_char) -> Obj;
        fn objc_allocateClassPair(parent: Obj, name: *const c_char, extra: usize) -> Obj;
        fn objc_registerClassPair(class: Obj);
        fn class_addMethod(
            class: Obj,
            selector: Obj,
            implementation: unsafe extern "C" fn(),
            types: *const c_char,
        ) -> bool;
        fn objc_msgSend();
    }
    unsafe fn selector(name: &[u8]) -> Obj {
        unsafe { sel_registerName(name.as_ptr().cast()) }
    }
    unsafe fn send0(object: Obj, selector: Obj) -> Obj {
        let call: unsafe extern "C" fn(Obj, Obj) -> Obj =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { call(object, selector) }
    }
    unsafe extern "C" fn handle(_this: Obj, _cmd: Obj, event: Obj, _reply: Obj) {
        let get: unsafe extern "C" fn(Obj, Obj, u32) -> Obj =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let descriptor = unsafe {
            get(
                event,
                selector(b"paramDescriptorForKeyword:\0"),
                u32::from_be_bytes(*b"----"),
            )
        };
        if descriptor.is_null() {
            return;
        }
        let string = unsafe { send0(descriptor, selector(b"stringValue\0")) };
        if string.is_null() {
            return;
        }
        let bytes = unsafe { send0(string, selector(b"UTF8String\0")) } as *const c_char;
        if bytes.is_null() {
            return;
        }
        let value = unsafe { CStr::from_ptr(bytes) }
            .to_string_lossy()
            .into_owned();
        if value.starts_with("rockoon://") {
            if let Ok(mut inbox) = INBOX.lock() {
                if inbox.len() < 32 {
                    inbox.push(vec![value]);
                    WAKE.notify_one();
                }
            }
        }
    }
    unsafe {
        let parent = objc_getClass(c"NSObject".as_ptr());
        let class = objc_allocateClassPair(parent, c"RockoonNativeUrlHandler".as_ptr(), 0);
        if class.is_null() {
            return;
        }
        let method: unsafe extern "C" fn() =
            std::mem::transmute(handle as unsafe extern "C" fn(Obj, Obj, Obj, Obj));
        class_addMethod(
            class,
            selector(b"handleURL:reply:\0"),
            method,
            c"v@:@@".as_ptr(),
        );
        objc_registerClassPair(class);
        let target = send0(send0(class, selector(b"alloc\0")), selector(b"init\0"));
        let manager = send0(
            objc_getClass(c"NSAppleEventManager".as_ptr()),
            selector(b"sharedAppleEventManager\0"),
        );
        let register: unsafe extern "C" fn(Obj, Obj, Obj, Obj, u32, u32) =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        register(
            manager,
            selector(b"setEventHandler:andSelector:forEventClass:andEventID:\0"),
            target,
            selector(b"handleURL:reply:\0"),
            u32::from_be_bytes(*b"GURL"),
            u32::from_be_bytes(*b"GURL"),
        );
    }
}
