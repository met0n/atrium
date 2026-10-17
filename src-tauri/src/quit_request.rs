//! On macOS, the stock Quit item (Cmd+Q) terminates the app inside AppKit before Tauri sees any
//! event, which skips the close flow where the workspace is saved and running agents are
//! confirmed. Atrium replaces that item with its own, which hands the request to the frontend;
//! the frontend runs the close flow and then quits through `quit_app`. A second request shortly
//! after the first quits directly, so a stuck frontend cannot trap the user.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::provider_common::now_ms;

pub const QUIT_REQUESTED_EVENT: &str = "app://quit-requested";
const QUIT_MENU_ID: &str = "atrium-quit";
const MAIN_WINDOW: &str = "main";
const FORCE_QUIT_WINDOW_MS: u64 = 10_000;

static LAST_DEFERRED_MS: AtomicU64 = AtomicU64::new(0);

/// The frontend can only run the close flow while the main window exists; a repeat within the
/// force-quit window means the previous request went unanswered.
fn should_defer(has_main_window: bool, last_ms: u64, now: u64) -> bool {
    has_main_window && (last_ms == 0 || now.saturating_sub(last_ms) >= FORCE_QUIT_WINDOW_MS)
}

/// Cmd+Q and Atrium ▸ Quit Atrium.
pub fn request_quit(app: &tauri::AppHandle) {
    use tauri::{Emitter, Manager};

    let has_main_window = app.get_webview_window(MAIN_WINDOW).is_some();
    let now = now_ms();
    let last = LAST_DEFERRED_MS.load(Ordering::SeqCst);
    if should_defer(has_main_window, last, now)
        && app.emit_to(MAIN_WINDOW, QUIT_REQUESTED_EVENT, ()).is_ok()
    {
        LAST_DEFERRED_MS.store(now, Ordering::SeqCst);
        return;
    }
    // Goes through RunEvent::ExitRequested, which tears down PTYs and helper processes.
    app.exit(0);
}

/// Installs the default macOS menu with Atrium's own Quit item in place of the stock one.
#[cfg(target_os = "macos")]
pub fn install_app_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, MenuItemKind};

    let menu = Menu::default(app)?;
    if let Some(MenuItemKind::Submenu(app_menu)) = menu.items()?.into_iter().next() {
        for item in app_menu.items()? {
            if let MenuItemKind::Predefined(predefined) = &item {
                if predefined.text()?.starts_with("Quit") {
                    app_menu.remove(&item)?;
                }
            }
        }
        let label = format!("Quit {}", app.package_info().name);
        let quit = MenuItem::with_id(app, QUIT_MENU_ID, label, true, Some("CmdOrCtrl+Q"))?;
        app_menu.append(&quit)?;
    }
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        if event.id().as_ref() == QUIT_MENU_ID {
            request_quit(app);
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_the_first_quit_to_the_frontend() {
        assert!(should_defer(true, 0, 50_000));
    }

    #[test]
    fn quits_directly_once_the_main_window_is_gone() {
        assert!(!should_defer(false, 0, 50_000));
    }

    #[test]
    fn a_second_quit_soon_after_forces_the_exit() {
        assert!(!should_defer(true, 45_000, 50_000));
        assert!(should_defer(true, 30_000, 50_000));
    }
}
