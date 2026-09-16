//! islemap-thienvyma multi-provider Tauri application shell.
//!
//! Position flows ONE way: selected provider (or the manual clipboard
//! fallback) -> tracker -> both windows.
//! See win/mod.rs for the anti-cheat safety boundary this app must never
//! cross.

pub mod clipboard;
pub mod combat;
pub mod commands;
pub mod events;
pub mod fetch;
pub mod hotkeys;
pub mod islepilot;
pub mod local_telemetry;
pub mod minimap;
pub mod mutation_locale;
pub mod mutation_overlay;
pub mod pipeline;
pub mod prime;
pub mod providers;
pub mod replay;
pub mod secure_store;
pub mod settings;
pub mod state;
pub mod store;
pub mod telemetry;
pub mod translate;
pub mod tray;
pub mod voice;
pub mod webview_mem;
pub mod win;

use std::path::PathBuf;

use tauri::Manager;

use crate::state::{AppState, LockExt};

pub fn run(replay_file: Option<PathBuf>) {
    settings::ensure_dirs().expect("failed to prepare islemap-thienvyma data directories");
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: settings::local_dir().join("logs"),
                        file_name: None,
                    }),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. }
                if window.label() == "main" && !tray::is_quitting() =>
            {
                api.prevent_close();
                log::info!("main window: hide (X to tray)");
                let _ = window.hide();
                if let Some(webview) = window.app_handle().get_webview_window("main") {
                    webview_mem::on_hidden(&webview);
                }
            }
            tauri::WindowEvent::Destroyed => win::vis::unregister(window.label()),
            _ => {}
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::patch_settings,
            voice::voice_status,
            voice::voice_start_login,
            voice::voice_logout,
            mutation_overlay::mutation_overlay_status,
            mutation_overlay::mutation_overlay_set_manual,
            mutation_overlay::mutation_overlay_clear_manual,
            mutation_overlay::mutation_overlay_begin_calibration,
            mutation_overlay::mutation_overlay_save_calibration,
            mutation_overlay::mutation_overlay_cancel_calibration,
            mutation_overlay::mutation_overlay_preview,
            commands::get_current_position,
            commands::get_current_heading,
            commands::list_waypoints,
            commands::list_waypoints_px,
            commands::add_waypoint_at_pixel,
            commands::set_destination_at_pixel,
            commands::add_waypoint_here,
            commands::rename_waypoint,
            commands::set_waypoint_color,
            commands::delete_waypoint,
            commands::resolve_coordinates,
            commands::get_previous_trail,
            commands::get_current_trail,
            commands::clear_trail,
            commands::data_status,
            commands::get_basemap_paths,
            commands::set_basemap_source,
            commands::get_pois,
            commands::get_pois_render,
            commands::nearest_waypoint,
            commands::get_fullscreen_mode,
            commands::get_map_info,
            commands::check_hotkey_available,
            commands::apply_hotkeys,
            commands::open_trails_folder,
            commands::fetch_data,
            commands::islepilot_login,
            commands::islepilot_set_cookie,
            commands::islepilot_cancel_login,
            commands::islepilot_token_login,
            commands::islepilot_set_token,
            commands::islepilot_overlay_map,
            commands::islepilot_friends,
            commands::islepilot_friend_action,
            commands::islepilot_open_friend_search,
            commands::islepilot_cdn_asset,
            commands::islepilot_garage,
            commands::islepilot_garage_park,
            commands::islepilot_garage_park_finalize,
            commands::islepilot_garage_park_cancel,
            commands::islepilot_garage_wait,
            commands::islepilot_http_pause,
            commands::islepilot_garage_restore,
            commands::islepilot_garage_sell,
            commands::islepilot_garage_self_slay,
            commands::islepilot_garage_rename,
            commands::islepilot_logout,
            commands::islepilot_apply,
            commands::islepilot_state,
            commands::islepilot_copy_steam_id,
            commands::provider_detect,
            commands::provider_start_login,
            commands::provider_cancel_login,
            commands::provider_state,
            commands::provider_snapshot,
            commands::provider_garage,
            commands::provider_garage_action,
            commands::provider_skin_state,
            commands::provider_skin_apply,
            commands::provider_logout,
            commands::provider_change_connection,
            commands::provider_select_manual,
            combat::combat_history,
            telemetry::track_feature,
            telemetry::submit_feedback,
            telemetry::submit_crash,
            #[cfg(debug_assertions)]
            commands::simulate_position,
        ]);

    builder
        .setup(move |app| {
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    log::warn!("deep-link register failed: {e}");
                }
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        let url = url.to_string();
                        if voice::handle_deep_link(&handle, &url) {
                            continue;
                        }
                        if !url.contains("token=") {
                            continue;
                        }
                        tray::show_main(&handle);
                        let app = handle.clone();
                        std::thread::spawn(move || {
                            if let Err(e) = crate::islepilot::manual_token(&app, url) {
                                log::warn!("deep-link token login failed: {e}");
                            }
                        });
                    }
                });
            }
            fetch::ensure_pois_current();
            fetch::spawn_topup(app.handle());
            {
                let state = app.state::<AppState>();
                let source = state.active_source();
                if let Some(variant) = fetch::IslemapsVariant::for_source(source) {
                    if !variant.dest().exists() {
                        log::warn!(
                            "selected basemap {} missing on disk - reverting to vulnona",
                            source.key()
                        );
                        let mut s = state.settings.lock_safe();
                        *s = settings::merge(
                            &s,
                            &serde_json::json!({ "map": { "basemap": "vulnona" } }),
                        );
                        drop(s);
                        state.request_settings_save();
                    }
                }
            }
            if let Some(main) = app.get_webview_window("main") {
                if let Ok(hwnd) = main.hwnd() {
                    win::vis::register("main", hwnd.0 as isize);
                }
            }
            minimap::create(app.handle())?;
            mutation_overlay::create(app.handle())?;
            tray::create(app.handle())?;
            clipboard::spawn(app.handle().clone());
            pipeline::spawn_heading_watchdog(app.handle().clone());
            local_telemetry::spawn(app.handle().clone());
            webview_mem::spawn_watchdog(app.handle().clone());
            {
                let state = app.state::<AppState>();
                state.hotkeys.restart(app.handle().clone());
            }
            providers::orchestrator::initialize(app.handle());
            telemetry::spawn(app.handle());
            if let Some(path) = replay_file {
                replay::spawn(app.handle().clone(), path);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
