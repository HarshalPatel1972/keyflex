// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod analytics;
pub mod apps;
pub mod commands;
pub mod db;
pub mod gaps;
pub mod hook;
pub mod types;

use commands::DbState;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_notification::NotificationExt;

pub struct TrayMenu(pub tauri::menu::Menu<tauri::Wry>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            // 1. Initialize database
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let conn =
                db::init(&app_dir).expect("Failed to initialize database");
            let db_state = DbState(Arc::new(Mutex::new(conn)));
            app.manage(db_state.clone());

            // 2. Start keyboard hook in background thread
            let (tx, rx) = std::sync::mpsc::channel::<String>();
            hook::start_hook(tx);

            // 3. Spawn receiver thread
            let receiver_db = db_state.clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(shortcut) = rx.recv() {
                    let (app_name, exe_path) = apps::get_foreground_app();
                    // Skip if app is Keyflex itself
                    if app_name == "Keyflex" {
                        continue;
                    }

                    let conn = receiver_db.0.lock();
                    let _ = db::insert_shortcut(&conn, shortcut.as_str(), app_name.as_str(), exe_path.as_str());
                    let today = db::today_date_string();
                    let stats = db::get_today_stats(&conn, "All Apps").unwrap_or_default();
                    let score_data = analytics::compute_efficiency_score(&conn).unwrap_or_default();
                    let today_total = stats.total;
                    let score_val = score_data.score;

                    let streak = db::update_streak(&conn, today.as_str()).ok();
                    let milestones =
                        analytics::check_milestones(&conn, shortcut.as_str(), app_name.as_str());
                    drop(conn); // Release lock before emitting

                    // Emit events to frontend
                    let _ = app_handle.emit(
                        "shortcut-recorded",
                        serde_json::json!({ "shortcut": shortcut, "app": app_name }),
                    );

                    for m in milestones {
                        let _ = app_handle.emit("milestone-achieved", &m);
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title(format!("🏆 {}", m.label))
                            .body(&m.description)
                            .show();
                    }

                    if let Some(s) = streak {
                        let _ = app_handle.emit("streak-updated", &s);
                        // Update Tray
                        let menu = app_handle.state::<TrayMenu>();
                        if let Some(item) = menu.0.get("streak") {
                            if let Some(mi) = item.as_menuitem() {
                                let _ = mi.set_text(format!("🔥 Streak: {} days", s.current));
                            }
                        }
                    }

                    // Update Today Stats in Tray
                    let menu = app_handle.state::<TrayMenu>();
                    if let Some(item) = menu.0.get("today") {
                        if let Some(mi) = item.as_menuitem() {
                            let _ = mi.set_text(format!("📅 Today: {} shortcuts", today_total));
                        }
                    }
                    if let Some(item) = menu.0.get("score") {
                        if let Some(mi) = item.as_menuitem() {
                            let _ = mi.set_text(format!("⚡ Score: {:.0}", score_val));
                        }
                    }
                }
            });

            // 4. Setup system tray
            let menu = setup_tray(app)?;
            app.manage(TrayMenu(menu));

            // 5. Show window (it starts hidden, but show on first launch for onboarding)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_top_shortcuts,
            commands::get_heatmap_data,
            commands::get_apps_list,
            commands::get_today_stats,
            commands::get_efficiency_score,
            commands::get_streak,
            commands::get_day_activity,
            commands::get_unique_shortcuts,
            commands::get_milestones,
            commands::get_setting,
            commands::set_setting,
            commands::get_shortcut_of_day,
            commands::get_per_app_breakdown,
            commands::get_gap_items,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error running Keyflex");
}

fn setup_tray(app: &tauri::App) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let open_item = MenuItemBuilder::with_id("open", "Open Dashboard").build(app)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let today_item = MenuItemBuilder::with_id("today", "📅 Today: 0 shortcuts")
        .enabled(false)
        .build(app)?;
    let streak_item = MenuItemBuilder::with_id("streak", "🔥 Streak: 0 days")
        .enabled(false)
        .build(app)?;
    let score_item = MenuItemBuilder::with_id("score", "⚡ Score: --")
        .enabled(false)
        .build(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit Keyflex").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &open_item,
            &sep1,
            &today_item,
            &streak_item,
            &score_item,
            &sep2,
            &quit_item,
        ])
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Keyflex — running")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(menu)
}
