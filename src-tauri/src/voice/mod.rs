pub mod manager;
pub mod model;
pub mod provider;
pub mod registry;
pub mod session_store;

use std::sync::{Arc, LazyLock};

use reqwest::Url;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use self::{
    manager::VoiceManager,
    model::{VoiceServerContext, VoiceStatus},
    registry::VoiceRegistry,
    session_store::DpapiVoiceSessionStore,
};

pub const STATE_EVENT: &str = "voice://state";

static MANAGER: LazyLock<VoiceManager> = LazyLock::new(|| {
    VoiceManager::new(
        VoiceRegistry::empty(),
        Arc::new(DpapiVoiceSessionStore::in_app_data()),
    )
});

fn current_server_context() -> Option<VoiceServerContext> {
    let state = crate::providers::orchestrator::current_state();
    let origin = state.website?;
    let snapshot = crate::providers::orchestrator::current_snapshot();
    Some(VoiceServerContext {
        origin,
        server_id: snapshot.as_ref().and_then(|value| value.server_id.clone()),
        server_name: snapshot.and_then(|value| value.server_name),
    })
}

fn current_status() -> VoiceStatus {
    MANAGER.status(current_server_context())
}

fn emit_status(app: &AppHandle) {
    crate::events::emit_all(app, STATE_EVENT, current_status());
}

#[tauri::command]
pub fn voice_status() -> VoiceStatus {
    current_status()
}

#[tauri::command]
pub fn voice_start_login(app: AppHandle) -> Result<VoiceStatus, String> {
    let context = current_server_context().ok_or_else(|| "not-configured".to_string())?;
    let start = MANAGER
        .start_login(context)
        .map_err(|error| error.as_str().to_string())?;
    if app
        .opener()
        .open_url(start.authorization_url, None::<&str>)
        .is_err()
    {
        MANAGER.cancel_login();
        return Err("provider-unavailable".to_string());
    }
    let status = current_status();
    emit_status(&app);
    Ok(status)
}

#[tauri::command]
pub fn voice_logout(app: AppHandle) -> Result<VoiceStatus, String> {
    let context = current_server_context().ok_or_else(|| "not-configured".to_string())?;
    MANAGER
        .logout(context)
        .map_err(|error| error.as_str().to_string())?;
    let status = current_status();
    emit_status(&app);
    Ok(status)
}

pub fn handle_deep_link(app: &AppHandle, url: &str) -> bool {
    let is_voice_callback = Url::parse(url).is_ok_and(|parsed| {
        parsed.scheme() == "islemap-thienvyma"
            && parsed.host_str() == Some("voice")
            && parsed.path() == "/callback"
    });
    if !is_voice_callback {
        return false;
    }
    if let Err(error) = MANAGER.handle_callback(url) {
        log::warn!("voice callback rejected: {}", error.as_str());
    }
    crate::tray::show_main(app);
    emit_status(app);
    true
}
