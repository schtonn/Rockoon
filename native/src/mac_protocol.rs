// Launch Services reports the selected handler, not merely Info.plist capability.
// https://developer.apple.com/documentation/coreservices/1441725-lscopydefaulthandlerforurlscheme
use std::ffi::{CStr, c_char, c_void};

type Ref = *const c_void;
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(allocator: Ref, text: *const c_char, encoding: u32) -> Ref;
    fn CFEqual(left: Ref, right: Ref) -> u8;
    fn CFRelease(value: Ref);
}
#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSCopyDefaultHandlerForURLScheme(scheme: Ref) -> Ref;
    fn LSSetDefaultHandlerForURLScheme(scheme: Ref, bundle: Ref) -> i32;
}
struct Owned(Ref);
impl Owned {
    fn string(value: &CStr) -> Option<Self> {
        let value =
            unsafe { CFStringCreateWithCString(std::ptr::null(), value.as_ptr(), 0x08000100) };
        (!value.is_null()).then_some(Self(value))
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) };
    }
}
pub fn registered() -> bool {
    let (Some(scheme), Some(bundle)) = (
        Owned::string(c"rockoon"),
        Owned::string(c"dev.schtonn.rockoon.native"),
    ) else {
        return false;
    };
    let handler = unsafe { LSCopyDefaultHandlerForURLScheme(scheme.0) };
    if handler.is_null() {
        return false;
    }
    let handler = Owned(handler);
    unsafe { CFEqual(handler.0, bundle.0) != 0 }
}
pub fn associate() -> Result<(), String> {
    let scheme = Owned::string(c"rockoon").ok_or("无法创建安装链接类型")?;
    let bundle = Owned::string(c"dev.schtonn.rockoon.native").ok_or("无法读取应用标识")?;
    let result = unsafe { LSSetDefaultHandlerForURLScheme(scheme.0, bundle.0) };
    if result != 0 {
        return Err(format!("关联资源站安装链接失败，系统错误 {result}"));
    }
    if !registered() {
        return Err("系统尚未将安装链接关联到 Rockoon，请完成可能出现的系统确认后重新关联".into());
    }
    Ok(())
}
