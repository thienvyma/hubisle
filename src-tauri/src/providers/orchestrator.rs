use std::{
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, LazyLock, Mutex,
    },
    time::Duration,
};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{
    islepilot::{self, parser::StatBar, DinoUpdate},
    pipeline,
    state::LockExt,
};

use super::{
    era::{self, EraError},
    model::{
        ConnectionStatus, ProviderFeaturePayload, ProviderId, ProviderSnapshot, ProviderState,
        SharedPlayer, SharedStatBar,
    },
    registry::{detect_provider, DetectedProvider},
    session_store::ProviderSessionStore,
    titan::{self, TitanError},
};

pub const PROVIDER_STATE: &str = "provider://state";
pub const PROVIDER_SNAPSHOT: &str = "provider://snapshot";
const LOGIN_WINDOW: &str = "provider-login";

#[derive(Debug, Default)]
pub struct PublishGate {
    generation: u64,
    active: Option<ProviderId>,
}

impl PublishGate {
    pub fn activate(&mut self, provider: ProviderId) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.active = Some(provider);
        self.generation
    }

    pub fn deactivate(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.active = None;
        self.generation
    }

    pub fn accepts(&self, generation: u64, provider: ProviderId) -> bool {
        self.generation == generation && self.active == Some(provider)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetAction {
    Reset,
}

#[derive(Debug)]
pub struct StateMachine {
    pub provider: Option<ProviderId>,
    pub status: ConnectionStatus,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self {
            provider: None,
            status: ConnectionStatus::Unconfigured,
        }
    }
}

impl StateMachine {
    pub fn select(&mut self, provider: ProviderId) -> ResetAction {
        self.provider = Some(provider);
        self.status = ConnectionStatus::Detecting;
        ResetAction::Reset
    }

    pub fn logout(&mut self) -> ResetAction {
        self.provider = None;
        self.status = ConnectionStatus::LoggedOut;
        ResetAction::Reset
    }
}

pub fn allows_main(status: ConnectionStatus) -> bool {
    matches!(
        status,
        ConnectionStatus::AuthenticatedOnline
            | ConnectionStatus::AuthenticatedOffline
            | ConnectionStatus::TemporaryError
    )
}

#[derive(Debug, Default)]
struct Runtime {
    gate: PublishGate,
    machine: StateMachine,
    website: Option<String>,
    message: Option<String>,
    last_received_at_ms: Option<i64>,
    last_snapshot: Option<ProviderSnapshot>,
}

static RUNTIME: LazyLock<Mutex<Runtime>> = LazyLock::new(|| Mutex::new(Runtime::default()));

fn runtime_state(runtime: &Runtime) -> ProviderState {
    ProviderState {
        provider: runtime.machine.provider,
        website: runtime.website.clone(),
        status: runtime.machine.status,
        message: runtime.message.clone(),
        last_received_at_ms: runtime.last_received_at_ms,
        data_stale: runtime.machine.status == ConnectionStatus::TemporaryError
            && runtime.last_snapshot.is_some(),
    }
}

fn emit_state(app: &AppHandle, state: ProviderState) {
    crate::events::emit_all(app, PROVIDER_STATE, state);
}

pub fn current_state() -> ProviderState {
    runtime_state(&RUNTIME.lock_safe())
}

pub fn current_snapshot() -> Option<ProviderSnapshot> {
    RUNTIME.lock_safe().last_snapshot.clone()
}

fn active_server_session() -> Result<(ProviderId, String, String, Option<String>), String> {
    let (provider, origin, server_id) = {
        let runtime = RUNTIME.lock_safe();
        let provider = runtime
            .machine
            .provider
            .ok_or_else(|| "Chưa chọn server.".to_string())?;
        let origin = runtime
            .website
            .clone()
            .ok_or_else(|| "Chưa có website server.".to_string())?;
        let server_id = runtime
            .last_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.server_id.clone());
        (provider, origin, server_id)
    };
    let cookie = ProviderSessionStore::get(provider, &origin)
        .ok_or_else(|| "Phiên đăng nhập server đã hết hạn.".to_string())?;
    Ok((provider, origin, cookie, server_id))
}

pub fn garage_fetch() -> Result<ProviderFeaturePayload, String> {
    let (provider, origin, cookie, _) = active_server_session()?;
    let data = match provider {
        ProviderId::Era => era::garage_fetch(&cookie)?,
        ProviderId::Titan => titan::garage_fetch(&cookie, &origin)?,
        ProviderId::IslePilot => {
            return Err("Garage IslePilot dùng kết nối riêng của IslePilot.".to_string())
        }
    };
    Ok(ProviderFeaturePayload { provider, data })
}

pub fn garage_action(
    action: &str,
    slot: Option<usize>,
    state_hash: Option<&str>,
) -> Result<ProviderFeaturePayload, String> {
    let (provider, origin, cookie, _) = active_server_session()?;
    let data = match provider {
        ProviderId::Era => era::garage_action(
            &cookie,
            action,
            slot.ok_or_else(|| "Chưa chọn slot Era.".to_string())?,
            state_hash,
        )?,
        ProviderId::Titan => titan::garage_action(&cookie, &origin, action, slot)?,
        ProviderId::IslePilot => {
            return Err("Garage IslePilot dùng kết nối riêng của IslePilot.".to_string())
        }
    };
    Ok(ProviderFeaturePayload { provider, data })
}

pub fn skin_state() -> Result<ProviderFeaturePayload, String> {
    let (provider, origin, cookie, _) = active_server_session()?;
    let data = match provider {
        ProviderId::Era => era::skin_state(&cookie)?,
        ProviderId::Titan => titan::skin_state(&cookie, &origin)?,
        ProviderId::IslePilot => {
            return Err("Server IslePilot này chưa công bố API đổi skin cho overlay.".to_string())
        }
    };
    Ok(ProviderFeaturePayload { provider, data })
}

pub fn skin_apply(colors: &[String], variation: f64) -> Result<ProviderFeaturePayload, String> {
    let (provider, origin, cookie, server_id) = active_server_session()?;
    let data = match provider {
        ProviderId::Era => era::skin_apply(&cookie, colors)?,
        ProviderId::Titan => titan::skin_apply(
            &cookie,
            &origin,
            server_id
                .as_deref()
                .ok_or_else(|| "Chưa nhận diện được server Titan đang chơi.".to_string())?,
            colors,
            variation,
        )?,
        ProviderId::IslePilot => {
            return Err("Server IslePilot này chưa công bố API đổi skin cho overlay.".to_string())
        }
    };
    Ok(ProviderFeaturePayload { provider, data })
}

pub fn last_has_stamina() -> bool {
    current_snapshot()
        .and_then(|snapshot| snapshot.player)
        .is_some_and(|player| player.stamina.is_some())
}

pub fn last_quest_count() -> usize {
    current_snapshot()
        .and_then(|snapshot| snapshot.player)
        .map(|player| player.prime_quests.len())
        .unwrap_or(0)
}

fn accepts(generation: u64, provider: ProviderId) -> bool {
    RUNTIME.lock_safe().gate.accepts(generation, provider)
}

fn set_status(
    app: &AppHandle,
    generation: u64,
    provider: ProviderId,
    status: ConnectionStatus,
    message: Option<String>,
) -> bool {
    let state = {
        let mut runtime = RUNTIME.lock_safe();
        if !runtime.gate.accepts(generation, provider) {
            return false;
        }
        runtime.machine.status = status;
        runtime.message = message;
        let retain_era = provider == ProviderId::Era
            && status == ConnectionStatus::TemporaryError
            && era::may_retain_snapshot(runtime.last_received_at_ms, chrono::Utc::now().timestamp_millis());
        if !retain_era && matches!(
            status,
            ConnectionStatus::LoginRequired | ConnectionStatus::TemporaryError
        ) {
            runtime.last_snapshot = None;
        }
        runtime_state(&runtime)
    };
    emit_state(app, state);
    true
}

fn publish_snapshot(
    app: &AppHandle,
    generation: u64,
    mut snapshot: ProviderSnapshot,
    clear_when_position_missing: bool,
) -> bool {
    let calibration = app.state::<crate::state::AppState>().active_calibration();
    for friend in &mut snapshot.friends {
        friend.position_px = friend
            .position_cm
            .map(|(x, y, _)| overlay_core::world_to_pixel(x, y, calibration));
    }
    let state = {
        let mut runtime = RUNTIME.lock_safe();
        if !runtime.gate.accepts(generation, snapshot.provider) {
            return false;
        }
        runtime.machine.status = snapshot.status;
        runtime.message = None;
        runtime.last_received_at_ms = Some(snapshot.received_at_ms);
        runtime.last_snapshot = Some(snapshot.clone());
        runtime_state(&runtime)
    };
    if let Some((x, y, z)) = snapshot.position_cm {
        pipeline::ingest_sample_with_heading(app, x, y, z, snapshot.heading_deg);
    } else if snapshot.status == ConnectionStatus::AuthenticatedOffline
        || clear_when_position_missing
    {
        pipeline::clear_position(app);
    }
    crate::events::emit_all(app, PROVIDER_SNAPSHOT, snapshot);
    emit_state(app, state);
    true
}

fn activate(
    app: &AppHandle,
    detected: DetectedProvider,
    status: ConnectionStatus,
    persist: bool,
) -> u64 {
    islepilot::stop_poller();
    let (generation, state) = {
        let mut runtime = RUNTIME.lock_safe();
        let generation = runtime.gate.activate(detected.id);
        runtime.machine.select(detected.id);
        runtime.machine.status = status;
        runtime.website = Some(detected.origin.clone());
        runtime.message = None;
        runtime.last_received_at_ms = None;
        runtime.last_snapshot = None;
        (generation, runtime_state(&runtime))
    };
    pipeline::clear_position(app);
    if persist {
        crate::commands::apply_settings_patch(
            app,
            serde_json::json!({
                "provider": {
                    "id": provider_key(detected.id),
                    "website": detected.origin,
                    "automatic_position": true
                }
            }),
        );
    }
    emit_state(app, state);
    generation
}

fn provider_key(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::IslePilot => "isle-pilot",
        ProviderId::Era => "era",
        ProviderId::Titan => "titan",
    }
}

fn provider_from_key(value: &str) -> Option<ProviderId> {
    match value {
        "isle-pilot" => Some(ProviderId::IslePilot),
        "era" => Some(ProviderId::Era),
        "titan" => Some(ProviderId::Titan),
        _ => None,
    }
}

fn interruptible_sleep(generation: u64, provider: ProviderId, seconds: u64) -> bool {
    for _ in 0..seconds.saturating_mul(2) {
        std::thread::sleep(Duration::from_millis(500));
        if !accepts(generation, provider) {
            return false;
        }
    }
    true
}

fn handle_era_error(app: &AppHandle, generation: u64, error: EraError) -> bool {
    match error {
        EraError::LoginRequired => {
            pipeline::clear_position(app);
            set_status(
                app,
                generation,
                ProviderId::Era,
                ConnectionStatus::LoginRequired,
                Some(error.to_string()),
            );
            false
        }
        EraError::Temporary | EraError::InvalidResponse => {
            if !set_status(
                app,
                generation,
                ProviderId::Era,
                ConnectionStatus::TemporaryError,
                Some(error.to_string()),
            ) { return false; }
            if current_snapshot().is_none() {
                pipeline::clear_position(app);
            }
            true
        }
    }
}

fn run_era(app: AppHandle, generation: u64, cookie: String) {
    std::thread::spawn(move || {
        let mut previous_snapshot: Option<ProviderSnapshot> = None;
        let mut failures: u32 = 0;
        loop {
            if !accepts(generation, ProviderId::Era) {
                return;
            }
            match era::poll_with_events(&cookie) {
                Ok((snapshot, mut events)) => {
                    failures = 0;
                    if let Some(previous) = previous_snapshot.as_ref() {
                        if let Some(event) = crate::combat::infer_era_health_drop(previous, &snapshot)
                        {
                            events.push(event);
                        }
                    }
                    if snapshot.status == ConnectionStatus::AuthenticatedOnline {
                        previous_snapshot = Some(snapshot.clone());
                    } else {
                        previous_snapshot = None;
                    }
                    if publish_snapshot(&app, generation, snapshot, true) {
                        crate::combat::ingest(&app, events);
                    }
                }
                Err(error) => {
                    failures = failures.saturating_add(1);
                    previous_snapshot = None;
                    if !handle_era_error(&app, generation, error) {
                        return;
                    }
                }
            }
            // Poll at the official map's cadence, and ease off on a slow bridge.
            if !interruptible_sleep(generation, ProviderId::Era, era::retry_delay_secs(failures)) {
                return;
            }
        }
    });
}

fn handle_titan_error(app: &AppHandle, generation: u64, error: TitanError) -> bool {
    match error {
        TitanError::LoginRequired => {
            pipeline::clear_position(app);
            set_status(
                app,
                generation,
                ProviderId::Titan,
                ConnectionStatus::LoginRequired,
                Some(error.to_string()),
            );
            false
        }
        TitanError::Temporary | TitanError::InvalidResponse | TitanError::MissingServer => {
            pipeline::clear_position(app);
            set_status(
                app,
                generation,
                ProviderId::Titan,
                ConnectionStatus::TemporaryError,
                Some(error.to_string()),
            );
            true
        }
        TitanError::InvalidOrigin => false,
    }
}

fn run_titan_stream(
    app: AppHandle,
    generation: u64,
    origin: String,
    cookie: String,
    last_stream_sample_ms: Arc<AtomicI64>,
) {
    std::thread::spawn(move || {
        loop {
            if !accepts(generation, ProviderId::Titan) {
                return;
            }
            // Polling keeps running while offline and discovers the new pawn.
            // Never replay samples from a stream opened for the previous life.
            if current_state().status == ConnectionStatus::AuthenticatedOffline {
                if !interruptible_sleep(generation, ProviderId::Titan, 1) { return; }
                continue;
            }
            let mut last_camera_heading = None;
            let result = titan::stream(&cookie, &origin, |sample| {
                if !accepts(generation, ProviderId::Titan)
                    || current_state().status == ConnectionStatus::AuthenticatedOffline {
                    return false;
                }
                let Some(sample) = sample else { return true; };
                let (x, y, z) = sample.position_cm;
                // Match Titan's camera-follow mode: once a camera sample exists,
                // keep it across sparse packets instead of jumping back to body
                // yaw. Body yaw is only the initial fallback.
                let heading = titan::preferred_live_heading(
                    &mut last_camera_heading, &sample, chrono::Utc::now().timestamp_millis());
                last_stream_sample_ms
                    .store(chrono::Utc::now().timestamp_millis(), Ordering::SeqCst);
                pipeline::ingest_sample_with_heading(&app, x, y, z, heading);
                true
            });
            if !accepts(generation, ProviderId::Titan) {
                return;
            }
            last_stream_sample_ms.store(0, Ordering::SeqCst);
            match result {
                Err(TitanError::LoginRequired) => return,
                Err(error) => log::debug!("titan live stream reconnecting: {error}"),
                Ok(()) => {}
            }
            if !interruptible_sleep(generation, ProviderId::Titan, 2) {
                return;
            }
        }
    });
}

fn run_titan(
    app: AppHandle,
    generation: u64,
    origin: String,
    cookie: String,
    initial_server_id: Option<String>,
) {
    std::thread::spawn(move || {
        // Titan's own downloadable HUD gets camera direction from this SSE
        // stream (~20 Hz). Keep the JSON poll as the stats/offline fallback.
        let last_stream_sample_ms = Arc::new(AtomicI64::new(0));
        run_titan_stream(
            app.clone(),
            generation,
            origin.clone(),
            cookie.clone(),
            last_stream_sample_ms.clone(),
        );
        let mut server_id = initial_server_id;
        loop {
            if !accepts(generation, ProviderId::Titan) {
                return;
            }
            if server_id.is_none() {
                match titan::validate(&cookie, &origin) {
                    Ok(Some(discovered)) => server_id = Some(discovered),
                    Ok(None) => {
                        let snapshot = ProviderSnapshot {
                            provider: ProviderId::Titan,
                            status: ConnectionStatus::AuthenticatedOffline,
                            server_id: None,
                            server_name: None,
                            received_at_ms: chrono::Utc::now().timestamp_millis(),
                            source_timestamp_ms: None,
                            player: None,
                            friends: Vec::new(),
                            position_cm: None,
                            heading_deg: None,
                        };
                        publish_snapshot(&app, generation, snapshot, true);
                    }
                    Err(error) => {
                        if !handle_titan_error(&app, generation, error) {
                            return;
                        }
                    }
                }
            }
            if let Some(id) = server_id.as_deref() {
                match titan::poll(&cookie, &origin, id) {
                    Ok(mut snapshot) => {
                        let stream_is_fresh = chrono::Utc::now().timestamp_millis()
                            - last_stream_sample_ms.load(Ordering::SeqCst)
                            < 1_500;
                        if stream_is_fresh
                            && snapshot.status == ConnectionStatus::AuthenticatedOnline
                        {
                            // The poll still refreshes stats, but must not
                            // overwrite the 20 Hz camera stream every 2 s.
                            snapshot.position_cm = None;
                            snapshot.heading_deg = None;
                        }
                        let publish_position = !stream_is_fresh
                            || snapshot.status != ConnectionStatus::AuthenticatedOnline;
                        publish_snapshot(&app, generation, snapshot, publish_position);
                    }
                    Err(TitanError::MissingServer | TitanError::InvalidResponse) => {
                        server_id = None;
                    }
                    Err(error) => {
                        if !handle_titan_error(&app, generation, error) {
                            return;
                        }
                    }
                }
            }
            if !interruptible_sleep(generation, ProviderId::Titan, 2) {
                return;
            }
        }
    });
}

fn start_selected_worker(
    app: &AppHandle,
    generation: u64,
    provider: ProviderId,
    origin: String,
    titan_server_id: Option<String>,
) {
    if provider == ProviderId::IslePilot {
        crate::commands::apply_settings_patch(
            app,
            serde_json::json!({"islepilot": {"enabled": true}}),
        );
        islepilot::restart_poller(app);
        return;
    }
    let Some(cookie) = ProviderSessionStore::get(provider, &origin) else {
        set_status(
            app,
            generation,
            provider,
            ConnectionStatus::LoginRequired,
            Some("Hãy đăng nhập website của server.".to_string()),
        );
        return;
    };
    set_status(
        app,
        generation,
        provider,
        ConnectionStatus::Validating,
        None,
    );
    match provider {
        ProviderId::Era => run_era(app.clone(), generation, cookie),
        ProviderId::Titan => run_titan(app.clone(), generation, origin, cookie, titan_server_id),
        ProviderId::IslePilot => {}
    }
}

pub fn detect(input: &str) -> Result<DetectedProvider, String> {
    detect_provider(input)
}

pub fn initialize(app: &AppHandle) {
    let (configured_id, configured_website, automatic) = {
        let state = app.state::<crate::state::AppState>();
        let settings = state.settings.lock_safe();
        (
            crate::settings::get_str(&settings, &["provider", "id"], "").to_string(),
            crate::settings::get_str(&settings, &["provider", "website"], "").to_string(),
            crate::settings::get_bool(&settings, &["provider", "automatic_position"], true),
        )
    };
    if !automatic && configured_id.is_empty() {
        select_manual(app);
        return;
    }

    let configured = provider_from_key(&configured_id).and_then(|id| {
        detect_provider(&configured_website)
            .ok()
            .filter(|detected| detected.id == id)
    });
    if let Some(detected) = configured {
        let generation = activate(app, detected.clone(), ConnectionStatus::Validating, false);
        start_selected_worker(app, generation, detected.id, detected.origin, None);
        return;
    }

    if islepilot::current_state(app).logged_in {
        let website = {
            let state = app.state::<crate::state::AppState>();
            let settings = state.settings.lock_safe();
            if crate::settings::get_str(&settings, &["islepilot", "auth_mode"], "legacy") == "token"
            {
                islepilot::api::API_ORIGIN.to_string()
            } else {
                crate::settings::get_str(&settings, &["islepilot", "domain"], "").to_string()
            }
        };
        if let Ok(detected) = detect_provider(&website) {
            let generation = activate(app, detected.clone(), ConnectionStatus::Validating, true);
            start_selected_worker(app, generation, detected.id, detected.origin, None);
        }
    }
}

pub fn start_login(app: &AppHandle, website: String) -> Result<(), String> {
    let detected = detect_provider(&website)?;
    let generation = activate(app, detected.clone(), ConnectionStatus::LoginRequired, true);
    if detected.id == ProviderId::IslePilot {
        crate::commands::apply_settings_patch(
            app,
            serde_json::json!({"islepilot": {"enabled": true, "auth_mode": "token"}}),
        );
        return islepilot::start_token_login(app);
    }

    if let Some(existing) = app.get_webview_window(LOGIN_WINDOW) {
        let _ = existing.close();
    }
    let login_url = match detected.id {
        ProviderId::Era => format!("{}/api/auth/steam", detected.origin),
        ProviderId::Titan => format!("{}/login?next=/nguoi-choi", detected.origin),
        ProviderId::IslePilot => unreachable!(),
    };
    let url: tauri::Url = login_url.parse().map_err(|e| format!("URL: {e}"))?;
    let window = WebviewWindowBuilder::new(app, LOGIN_WINDOW, WebviewUrl::External(url.clone()))
        .title(match detected.id {
            ProviderId::Era => "Era Gaming VN — Steam",
            ProviderId::Titan => "The Real Server VN — Đăng nhập",
            ProviderId::IslePilot => "IslePilot — Steam",
        })
        .inner_size(620.0, 760.0)
        .build()
        .map_err(|e| e.to_string())?;

    let finished = Arc::new(AtomicBool::new(false));
    let close_finished = finished.clone();
    let close_app = app.clone();
    let close_provider = detected.id;
    window.on_window_event(move |event| {
        if matches!(
            event,
            tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
        ) && !close_finished.load(Ordering::SeqCst)
        {
            set_status(
                &close_app,
                generation,
                close_provider,
                ConnectionStatus::LoginRequired,
                Some("Cửa sổ đăng nhập đã đóng.".to_string()),
            );
        }
    });

    let app = app.clone();
    std::thread::spawn(move || {
        for _ in 0..150 {
            std::thread::sleep(Duration::from_secs(2));
            if !accepts(generation, detected.id) || finished.load(Ordering::SeqCst) {
                return;
            }
            let Some(window) = app.get_webview_window(LOGIN_WINDOW) else {
                return;
            };
            let Ok(cookies) = window.cookies_for_url(url.clone()) else {
                continue;
            };
            if cookies.is_empty() {
                continue;
            }
            let header = cookies
                .iter()
                .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                .collect::<Vec<_>>()
                .join("; ");
            let validation: Result<(Option<String>, Option<ProviderSnapshot>), EraOrTitanError> =
                match detected.id {
                    ProviderId::Era => era::validate(&header)
                        .map(|snapshot| (None, Some(snapshot)))
                        .map_err(EraOrTitanError::from),
                    ProviderId::Titan => titan::validate(&header, &detected.origin)
                        .map(|server_id| (server_id, None))
                        .map_err(EraOrTitanError::from),
                    ProviderId::IslePilot => unreachable!(),
                };
            match validation {
                Ok((server_id, era_snapshot)) => {
                    if ProviderSessionStore::set(detected.id, &detected.origin, &header).is_err() {
                        set_status(
                            &app,
                            generation,
                            detected.id,
                            ConnectionStatus::TemporaryError,
                            Some("Không thể lưu phiên đăng nhập an toàn.".to_string()),
                        );
                        return;
                    }
                    finished.store(true, Ordering::SeqCst);
                    let _ = window.close();
                    if let Some(snapshot) = era_snapshot {
                        publish_snapshot(&app, generation, snapshot, true);
                    }
                    start_selected_worker(
                        &app,
                        generation,
                        detected.id,
                        detected.origin,
                        server_id,
                    );
                    return;
                }
                Err(EraOrTitanError::LoginRequired) => continue,
                Err(EraOrTitanError::Temporary) => continue,
                Err(EraOrTitanError::Invalid) => continue,
            }
        }
        set_status(
            &app,
            generation,
            detected.id,
            ConnectionStatus::LoginRequired,
            Some("Đăng nhập quá thời gian chờ; hãy thử lại.".to_string()),
        );
    });
    Ok(())
}

#[derive(Debug)]
enum EraOrTitanError {
    LoginRequired,
    Temporary,
    Invalid,
}

impl From<EraError> for EraOrTitanError {
    fn from(value: EraError) -> Self {
        match value {
            EraError::LoginRequired => Self::LoginRequired,
            EraError::Temporary => Self::Temporary,
            EraError::InvalidResponse => Self::Invalid,
        }
    }
}

impl From<TitanError> for EraOrTitanError {
    fn from(value: TitanError) -> Self {
        match value {
            TitanError::LoginRequired => Self::LoginRequired,
            TitanError::Temporary => Self::Temporary,
            TitanError::InvalidResponse | TitanError::MissingServer | TitanError::InvalidOrigin => {
                Self::Invalid
            }
        }
    }
}

pub fn cancel_login(app: &AppHandle) {
    islepilot::cancel_login(app);
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW) {
        let _ = window.close();
    }
    let state = {
        let mut runtime = RUNTIME.lock_safe();
        if let Some(provider) = runtime.machine.provider {
            runtime.gate.activate(provider);
        } else {
            runtime.gate.deactivate();
        }
        runtime.machine.status = ConnectionStatus::LoginRequired;
        runtime.message = Some("Đã hủy đăng nhập.".to_string());
        runtime_state(&runtime)
    };
    emit_state(app, state);
}

pub fn logout(app: &AppHandle) -> Result<(), String> {
    let selected = {
        let runtime = RUNTIME.lock_safe();
        runtime
            .machine
            .provider
            .zip(runtime.website.as_deref().map(str::to_string))
    };
    if let Some((provider, origin)) = selected {
        match provider {
            ProviderId::IslePilot => islepilot::logout(app)?,
            ProviderId::Era | ProviderId::Titan => {
                ProviderSessionStore::remove(provider, &origin)?;
            }
        }
    }
    let state = {
        let mut runtime = RUNTIME.lock_safe();
        runtime.gate.deactivate();
        runtime.machine.logout();
        runtime.website = None;
        runtime.message = None;
        runtime.last_received_at_ms = None;
        runtime.last_snapshot = None;
        runtime_state(&runtime)
    };
    crate::commands::apply_settings_patch(
        app,
        serde_json::json!({
            "provider": {"id": null, "website": null, "automatic_position": true}
        }),
    );
    pipeline::clear_position(app);
    emit_state(app, state);
    Ok(())
}

pub fn select_manual(app: &AppHandle) {
    islepilot::stop_poller();
    let state = {
        let mut runtime = RUNTIME.lock_safe();
        runtime.gate.deactivate();
        runtime.machine.provider = None;
        runtime.machine.status = ConnectionStatus::AuthenticatedOffline;
        runtime.website = None;
        runtime.message = Some("Chế độ tọa độ thủ công đang bật.".to_string());
        runtime.last_snapshot = None;
        runtime_state(&runtime)
    };
    crate::commands::apply_settings_patch(
        app,
        serde_json::json!({
            "provider": {"id": null, "website": null, "automatic_position": false}
        }),
    );
    pipeline::clear_position(app);
    emit_state(app, state);
}

fn shared_bar(value: &StatBar) -> Option<SharedStatBar> {
    match (value.current, value.max) {
        (Some(current), Some(max)) if current.is_finite() && max.is_finite() && max > 0.0 => {
            Some(SharedStatBar::from_values(current, max))
        }
        _ => None,
    }
}

pub fn publish_islepilot(app: &AppHandle, update: &DinoUpdate) {
    let (generation, selected) = {
        let runtime = RUNTIME.lock_safe();
        (runtime.gate.generation, runtime.machine.provider)
    };
    if selected != Some(ProviderId::IslePilot) {
        return;
    }
    if update.error.is_some() {
        pipeline::clear_position(app);
        set_status(
            app,
            generation,
            ProviderId::IslePilot,
            ConnectionStatus::TemporaryError,
            Some("Tạm thời không thể cập nhật IslePilot.".to_string()),
        );
        return;
    }
    let status = if update.player.as_ref().and_then(|player| player.online) == Some(true) {
        ConnectionStatus::AuthenticatedOnline
    } else {
        ConnectionStatus::AuthenticatedOffline
    };
    if status == ConnectionStatus::AuthenticatedOffline {
        pipeline::clear_position(app);
    }
    let player = update.player.as_ref().map(|player| SharedPlayer {
        name: None,
        dino_name: player.dino_name.clone(),
        female: player.female,
        growth_pct: player.growth_pct,
        health: player.health.as_ref().and_then(shared_bar),
        stamina: player.stamina.as_ref().and_then(shared_bar),
        hunger: player.hunger.as_ref().and_then(shared_bar),
        thirst: player.thirst.as_ref().and_then(shared_bar),
        mutations: Vec::new(),
        nutrition: player.nutrition,
        prime_quests: player.prime_quests.clone(),
    });
    let snapshot = ProviderSnapshot {
        provider: ProviderId::IslePilot,
        status,
        server_id: None,
        server_name: update
            .player
            .as_ref()
            .and_then(|player| player.server.clone()),
        received_at_ms: update.fetched_at_ms as i64,
        source_timestamp_ms: None,
        player,
        friends: Vec::new(),
        position_cm: None,
        heading_deg: None,
    };
    publish_snapshot(app, generation, snapshot, false);
}

pub fn islepilot_auth_expired(app: &AppHandle) {
    let generation = {
        let runtime = RUNTIME.lock_safe();
        if runtime.machine.provider != Some(ProviderId::IslePilot) {
            return;
        }
        runtime.gate.generation
    };
    pipeline::clear_position(app);
    set_status(
        app,
        generation,
        ProviderId::IslePilot,
        ConnectionStatus::LoginRequired,
        Some("Phiên đăng nhập IslePilot đã hết hạn.".to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_generation_cannot_publish_after_switch() {
        let mut gate = PublishGate::default();
        let era = gate.activate(ProviderId::Era);
        let titan = gate.activate(ProviderId::Titan);
        assert!(!gate.accepts(era, ProviderId::Era));
        assert!(gate.accepts(titan, ProviderId::Titan));
    }

    #[test]
    fn startup_gate_allows_authenticated_and_temporary_states() {
        assert!(allows_main(ConnectionStatus::AuthenticatedOnline));
        assert!(allows_main(ConnectionStatus::AuthenticatedOffline));
        assert!(allows_main(ConnectionStatus::TemporaryError));
        assert!(!allows_main(ConnectionStatus::LoginRequired));
        assert!(!allows_main(ConnectionStatus::Unconfigured));
    }

    #[test]
    fn provider_switch_and_logout_request_a_reset() {
        let mut machine = StateMachine::default();
        assert_eq!(machine.select(ProviderId::Era), ResetAction::Reset);
        machine.status = ConnectionStatus::TemporaryError;
        assert_eq!(machine.provider, Some(ProviderId::Era));
        assert_eq!(machine.select(ProviderId::Titan), ResetAction::Reset);
        assert_eq!(machine.logout(), ResetAction::Reset);
        assert_eq!(machine.provider, None);
        assert_eq!(machine.status, ConnectionStatus::LoggedOut);
    }
}
