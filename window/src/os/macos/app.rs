use crate::connection::ConnectionOps;
use crate::macos::menu::RepresentedItem;
use crate::macos::{nsstring, nsstring_to_str};
use crate::menu::{Menu, MenuItem};
use crate::{ApplicationEvent, Connection};
use cocoa::appkit::NSApplicationTerminateReply;
use cocoa::base::id;
use cocoa::foundation::NSInteger;
use config::keyassignment::KeyAssignment;
use config::WindowCloseConfirmation;
use objc::declare::ClassDecl;
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, Sel, BOOL, NO, YES};
use objc::*;
use std::cell::RefCell;
use std::time::{Duration, Instant};

const CLS_NAME: &str = "KakuAppDelegate";

thread_local! {
    static LAST_OPEN_UNTITLED_SPAWN: RefCell<Option<Instant>> = RefCell::new(None);
}
// macOS can emit applicationOpenUntitledFile twice while no window has
// materialized yet; keep a wider debounce to avoid duplicate SpawnWindow work.
const OPEN_UNTITLED_SPAWN_DEBOUNCE: Duration = Duration::from_millis(1200);

extern "C" fn application_should_terminate(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> u64 {
    unsafe {
        match config::configuration().window_close_confirmation {
            WindowCloseConfirmation::NeverPrompt => terminate_now(),
            WindowCloseConfirmation::AlwaysPrompt => {
                let alert: id = msg_send![class!(NSAlert), alloc];
                let alert: id = msg_send![alert, init];
                let message_text = nsstring("Terminate Kaku?");
                let info_text = nsstring("Detach and close all panes and terminate Kaku?");
                let cancel = nsstring("Cancel");
                let ok = nsstring("Ok");

                let () = msg_send![alert, setMessageText: message_text];
                let () = msg_send![alert, setInformativeText: info_text];
                let () = msg_send![alert, addButtonWithTitle: cancel];
                let () = msg_send![alert, addButtonWithTitle: ok];
                #[allow(non_upper_case_globals)]
                const NSModalResponseCancel: NSInteger = 1000;
                #[allow(non_upper_case_globals, dead_code)]
                const NSModalResponseOK: NSInteger = 1001;
                let result: NSInteger = msg_send![alert, runModal];

                if result == NSModalResponseCancel {
                    NSApplicationTerminateReply::NSTerminateCancel as u64
                } else {
                    terminate_now()
                }
            }
        }
    }
}

extern "C" fn application_should_terminate_after_last_window_closed(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> BOOL {
    // Keep app process alive on macOS after the last window closes,
    // so Dock reopen can create a new window without cold-start.
    NO
}

fn terminate_now() -> u64 {
    // Persist the key (frontmost) window's geometry before the event loop
    // stops. This is the reliable save path for Cmd+Q; window_will_close
    // may not fire for every window before the process exits.
    super::window::on_app_terminating();
    if let Some(conn) = Connection::get() {
        conn.terminate_message_loop();
    }
    NSApplicationTerminateReply::NSTerminateNow as u64
}

extern "C" fn application_will_finish_launching(
    _self: &mut Object,
    _sel: Sel,
    _notif: *mut Object,
) {
    log::debug!("application_will_finish_launching");
}

extern "C" fn application_did_finish_launching(this: &mut Object, _sel: Sel, _notif: *mut Object) {
    log::debug!("application_did_finish_launching");
    unsafe {
        (*this).set_ivar("launched", YES);
    }
}

extern "C" fn application_open_untitled_file(
    this: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> BOOL {
    let launched: BOOL = unsafe { *this.get_ivar("launched") };
    if let Some(conn) = Connection::get() {
        if launched == YES {
            let existing_window = {
                let windows = conn.windows.borrow();
                windows.values().next().cloned()
            };
            if let Some(window) = existing_window {
                LAST_OPEN_UNTITLED_SPAWN.with(|last| {
                    last.borrow_mut().take();
                });
                window.borrow_mut().focus();
            } else {
                let should_spawn = LAST_OPEN_UNTITLED_SPAWN.with(|last| {
                    let now = Instant::now();
                    let mut last = last.borrow_mut();
                    let elapsed = last.map(|prior| now.duration_since(prior));
                    let too_soon = elapsed
                        .map(|duration| duration < OPEN_UNTITLED_SPAWN_DEBOUNCE)
                        .unwrap_or(false);
                    if too_soon {
                        false
                    } else {
                        *last = Some(now);
                        true
                    }
                });

                if should_spawn {
                    conn.dispatch_app_event(ApplicationEvent::PerformKeyAssignment(
                        KeyAssignment::SpawnWindow,
                    ));
                }
            }
        }
        return YES;
    }
    NO
}

extern "C" fn kaku_perform_key_assignment(_self: &mut Object, _sel: Sel, menu_item: *mut Object) {
    let menu_item = crate::os::macos::menu::MenuItem::with_menu_item(menu_item);
    // Safe because kakuPerformKeyAssignment: is only used with KeyAssignment
    let action = menu_item.get_represented_item();
    log::debug!("kaku_perform_key_assignment {action:?}",);
    match action {
        Some(RepresentedItem::KeyAssignment(action)) => {
            if let Some(conn) = Connection::get() {
                conn.dispatch_app_event(ApplicationEvent::PerformKeyAssignment(action));
            }
        }
        None => {}
    }
}

extern "C" fn show_settings_window(_self: &mut Object, _sel: Sel, _sender: *mut Object) {
    if let Some(conn) = Connection::get() {
        conn.dispatch_app_event(ApplicationEvent::PerformKeyAssignment(
            KeyAssignment::EmitEvent("open-kaku-config".to_string()),
        ));
    }
}

extern "C" fn application_open_file(
    this: &mut Object,
    _sel: Sel,
    _app: *mut Object,
    file_name: *mut Object,
) -> BOOL {
    let launched: BOOL = unsafe { *this.get_ivar("launched") };
    if launched == YES {
        let file_name = unsafe { nsstring_to_str(file_name) }.to_string();
        if let Some(conn) = Connection::get() {
            log::debug!("application_open_file {file_name}");
            conn.dispatch_app_event(ApplicationEvent::OpenCommandScript(file_name));
            return YES;
        }
    }
    NO
}

extern "C" fn application_dock_menu(
    _self: &mut Object,
    _sel: Sel,
    _app: *mut Object,
) -> *mut Object {
    let dock_menu = Menu::new_with_title("");
    let new_window_item =
        MenuItem::new_with("New Window", Some(sel!(kakuPerformKeyAssignment:)), "");
    new_window_item
        .set_represented_item(RepresentedItem::KeyAssignment(KeyAssignment::SpawnWindow));
    dock_menu.add_item(&new_window_item);
    dock_menu.autorelease()
}

fn get_class() -> &'static Class {
    Class::get(CLS_NAME).unwrap_or_else(|| {
        let mut cls = ClassDecl::new(CLS_NAME, class!(NSObject))
            .expect("Unable to register application delegate class");

        cls.add_ivar::<BOOL>("launched");

        unsafe {
            cls.add_method(
                sel!(applicationShouldTerminate:),
                application_should_terminate as extern "C" fn(&mut Object, Sel, *mut Object) -> u64,
            );
            cls.add_method(
                sel!(applicationShouldTerminateAfterLastWindowClosed:),
                application_should_terminate_after_last_window_closed
                    as extern "C" fn(&mut Object, Sel, *mut Object) -> BOOL,
            );
            cls.add_method(
                sel!(applicationWillFinishLaunching:),
                application_will_finish_launching as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(applicationDidFinishLaunching:),
                application_did_finish_launching as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(application:openFile:),
                application_open_file
                    as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object) -> BOOL,
            );
            cls.add_method(
                sel!(applicationDockMenu:),
                application_dock_menu
                    as extern "C" fn(&mut Object, Sel, *mut Object) -> *mut Object,
            );
            cls.add_method(
                sel!(kakuPerformKeyAssignment:),
                kaku_perform_key_assignment as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            // macOS may route "Settings..." through one of these standard selectors
            // instead of our custom menu-item selector.
            cls.add_method(
                sel!(showSettingsWindow:),
                show_settings_window as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(showPreferencesWindow:),
                show_settings_window as extern "C" fn(&mut Object, Sel, *mut Object),
            );
            cls.add_method(
                sel!(applicationOpenUntitledFile:),
                application_open_untitled_file
                    as extern "C" fn(&mut Object, Sel, *mut Object) -> BOOL,
            );
        }

        cls.register()
    })
}

pub fn create_app_delegate() -> StrongPtr {
    let cls = get_class();
    unsafe {
        let delegate: *mut Object = msg_send![cls, alloc];
        let delegate: *mut Object = msg_send![delegate, init];
        (*delegate).set_ivar("launched", NO);
        StrongPtr::new(delegate)
    }
}
