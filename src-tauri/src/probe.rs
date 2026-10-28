//! Reads the live state of the Mac: frontmost app and window title, idle time, screen lock.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

use crate::tracker::Window;

type CFTypeRef = *const c_void;

const ON_SCREEN_ONLY: u32 = 1 << 0;
const EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
const HID_SYSTEM_STATE: i32 = 1;
const ANY_INPUT_EVENT: u32 = !0;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFTypeRef;
    fn CGSessionCopyCurrentDictionary() -> CFTypeRef;
    fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
}

/// Takes ownership of a CF object returned by a `Copy` function. CF arrays and dictionaries are
/// toll-free bridged with their NS counterparts.
unsafe fn bridge<T: objc2::Message>(cf: CFTypeRef) -> Option<Retained<T>> {
    unsafe { Retained::from_raw(cf as *mut T) }
}

pub fn idle_ms() -> i64 {
    let secs = unsafe { CGEventSourceSecondsSinceLastEventType(HID_SYSTEM_STATE, ANY_INPUT_EVENT) };
    (secs * 1000.0) as i64
}

pub fn screen_locked() -> bool {
    let dict: Option<Retained<NSDictionary<NSString, AnyObject>>> =
        unsafe { bridge(CGSessionCopyCurrentDictionary()) };
    let Some(dict) = dict else { return false };
    dict.objectForKey(&NSString::from_str("CGSSessionScreenIsLocked"))
        .and_then(|v| v.downcast::<NSNumber>().ok())
        .is_some_and(|n| n.boolValue())
}

pub struct FrontApp {
    pub window: Window,
    /// Path of the `.app` bundle, used to look up its icon.
    pub bundle_path: Option<String>,
}

pub fn front_app() -> Option<FrontApp> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let pid = app.processIdentifier();
    let name = app.localizedName().map(|n| n.to_string()).unwrap_or_default();
    let app_id = match app.bundleIdentifier() {
        Some(id) => id.to_string(),
        None => format!("exe:{name}"),
    };
    let bundle_path = app.bundleURL().and_then(|u| u.path()).map(|p| p.to_string());
    Some(FrontApp {
        window: Window { app_id, app_name: name, title: front_window_title(pid).unwrap_or_default() },
        bundle_path,
    })
}

/// Title of the frontmost normal window of a process. Needs Screen Recording permission;
/// without it macOS leaves titles out and this returns `None`.
fn front_window_title(pid: i32) -> Option<String> {
    let windows: Retained<NSArray<NSDictionary<NSString, AnyObject>>> = unsafe {
        bridge(CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP_ELEMENTS, 0))
    }?;
    let number = |w: &NSDictionary<NSString, AnyObject>, key: &str| {
        w.objectForKey(&NSString::from_str(key))
            .and_then(|v| v.downcast::<NSNumber>().ok())
            .map(|n| n.intValue())
    };
    // The list is ordered front to back, so the first normal-layer window of the pid is on top.
    windows
        .iter()
        .find(|w| number(w, "kCGWindowOwnerPID") == Some(pid) && number(w, "kCGWindowLayer") == Some(0))
        .and_then(|w| w.objectForKey(&NSString::from_str("kCGWindowName")))
        .and_then(|v| v.downcast::<NSString>().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    /// Smoke test against the real machine: `cargo test probe -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn prints_live_state() {
        let app = super::front_app().expect("a frontmost app");
        println!("app: {:?} path: {:?}", app.window, app.bundle_path);
        println!("idle_ms: {} locked: {}", super::idle_ms(), super::screen_locked());
    }
}
