//! JSON client for the CENTRAL IslePilot overlay API (`islepilot.eu`).
//!
//! Unlike the per-server HTML panels (parser.rs), this API authenticates with
//! ONE bearer overlay-token that follows the player across every IslePilot
//! server — the backend itself knows which server they are on. Endpoints and
//! headers were verified against the official overlay app (see
//! rv/TheIsleVN-Gacha-HUD-integration-guide.md).

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use overlay_core::{map_yaw_to_bearing_deg, pixel_to_world, Calibration};

use super::parser::{Nutrition, PlayerStats, QuestStatus, StatBar};

pub const API_ORIGIN: &str = "https://islepilot.eu";

#[derive(Debug)]
pub enum ApiError {
    /// 401 / `{"error":"unauthorized"}` — token expired or revoked.
    Unauthorized,
    /// 404 — account has never been on an IslePilot server. Not a failure.
    NotFound,
    Http(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "unauthorized"),
            ApiError::NotFound => write!(f, "not found"),
            ApiError::Http(e) => write!(f, "{e}"),
        }
    }
}

fn request(
    client: &reqwest::blocking::Client,
    method: reqwest::Method,
    path: &str,
    token: &str,
    body: Option<&Value>,
) -> Result<Value, ApiError> {
    let mut req = client
        .request(method, format!("{API_ORIGIN}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .header("X-Overlay-Version", "2");
    if let Some(body) = body {
        // reqwest's `json` feature is off in this crate — set the body by hand.
        req = req
            .header("Content-Type", "application/json")
            .body(body.to_string());
    }
    let resp = req.send().map_err(|e| ApiError::Http(e.to_string()))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| ApiError::Http(e.to_string()))?;
    if status.as_u16() == 401 {
        return Err(ApiError::Unauthorized);
    }
    if status.as_u16() == 404 {
        return Err(ApiError::NotFound);
    }
    if !status.is_success() {
        return Err(ApiError::Http(format!("{path} -> HTTP {status}")));
    }
    let v: Value =
        serde_json::from_str(&text).map_err(|e| ApiError::Http(format!("{path}: {e}")))?;
    // Some auth failures come back 200 with an error body.
    if v.get("error").and_then(|e| e.as_str()) == Some("unauthorized") {
        return Err(ApiError::Unauthorized);
    }
    Ok(v)
}

fn get(client: &reqwest::blocking::Client, path: &str, token: &str) -> Result<Value, ApiError> {
    request(client, reqwest::Method::GET, path, token, None)
}

fn post(
    client: &reqwest::blocking::Client,
    path: &str,
    token: &str,
    body: &Value,
) -> Result<Value, ApiError> {
    request(client, reqwest::Method::POST, path, token, Some(body))
}

// ---------------------------------------------------------------------------
// /api/overlay/me — vitals + position + prime progress
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayMe {
    pub has_data: bool,
    pub steam_id: Option<String>,
    pub name: Option<String>,
    pub server: Option<String>,
    pub online: Option<bool>,
    pub species: Option<String>,
    pub female: Option<bool>,
    pub growth: Option<f64>,
    pub health: Option<f64>,
    pub max_health: Option<f64>,
    pub hunger: Option<f64>,
    pub max_hunger: Option<f64>,
    pub thirst: Option<f64>,
    pub max_thirst: Option<f64>,
    pub stamina: Option<f64>,
    pub max_stamina: Option<f64>,
    pub nutrition: Option<OverlayNutrition>,
    pub position: Option<OverlayPosition>,
    pub prime: Option<OverlayPrime>,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayNutrition {
    pub carb: f64,
    pub protein: f64,
    pub lipid: f64,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPosition {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub yaw: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPrime {
    pub eligible: bool,
    pub elder: bool,
    pub quests: Vec<OverlayQuest>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct OverlayQuest {
    pub name: String,
    pub done: bool,
}

pub fn get_me(client: &reqwest::blocking::Client, token: &str) -> Result<OverlayMe, ApiError> {
    let v = get(client, "/api/overlay/me", token)?;
    serde_json::from_value(v).map_err(|e| ApiError::Http(format!("/api/overlay/me: {e}")))
}

/// Map the JSON vitals into the exact struct the HTML parser produces, so the
/// whole downstream (DinoTab, minimap panels, translate) is untouched.
pub fn to_player_stats(me: &OverlayMe) -> PlayerStats {
    // Observed as a 0..1 fraction (0.2628); tolerate an already-percent value
    // defensively.
    let growth_pct = me.growth.map(|g| if g <= 1.5 { g * 100.0 } else { g });
    let bar = |cur: Option<f64>, max: Option<f64>| -> Option<StatBar> {
        Some(StatBar::from_values(cur?, max?))
    };
    PlayerStats {
        dino_name: me.species.clone(),
        online: me.online,
        growth: growth_pct.map(|p| format!("{}%", p.round() as i64)),
        growth_pct,
        health: bar(me.health, me.max_health),
        hunger: bar(me.hunger, me.max_hunger),
        thirst: bar(me.thirst, me.max_thirst),
        prime_quests: me
            .prime
            .as_ref()
            .map(|p| {
                p.quests
                    .iter()
                    .map(|q| QuestStatus {
                        text: q.name.clone(),
                        text_vi: None,
                        completed: q.done,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        stamina: bar(me.stamina, me.max_stamina),
        nutrition: me.nutrition.map(|n| Nutrition {
            carb: n.carb,
            protein: n.protein,
            lipid: n.lipid,
        }),
        server: me.server.clone(),
        female: me.female,
    }
}

/// Own position in game cm, OUR axis convention (their x = our y — the same
/// swap `parse_own_marker` uses, verified against named landmarks).
pub fn position_cm(me: &OverlayMe) -> Option<(f64, f64)> {
    let pos = me.position?;
    Some((pos.y?, pos.x?))
}

/// Exact north-up compass heading supplied by IslePilot. The API yaw uses
/// the same convention as its web map: zero points to screen-right/east.
pub fn position_heading_deg(me: &OverlayMe) -> Option<f64> {
    me.position?.yaw.and_then(map_yaw_to_bearing_deg)
}

pub fn position_cm3(me: &OverlayMe) -> Option<(f64, f64, f64)> {
    let pos = me.position?;
    Some((pos.y?, pos.x?, pos.z.unwrap_or(0.0)))
}

pub fn position_cm3_with_calibration(
    me: &OverlayMe,
    calibration: Option<&OverlayCalibration>,
) -> Option<(f64, f64, f64)> {
    let pos = me.position?;
    let (x_cm, y_cm) = world_point_cm(calibration, pos.x?, pos.y?)?;
    Some((x_cm, y_cm, pos.z.filter(|z| z.is_finite()).unwrap_or(0.0)))
}

pub fn position_heading_with_calibration(
    me: &OverlayMe,
    calibration: Option<&OverlayCalibration>,
) -> Option<f64> {
    let pos = me.position?;
    let yaw = pos.yaw?;
    match (calibration, pos.x, pos.y) {
        (Some(cal), Some(x), Some(y)) => cal
            .heading_deg(x, y, yaw)
            .or_else(|| map_yaw_to_bearing_deg(yaw)),
        _ => map_yaw_to_bearing_deg(yaw),
    }
}

// ---------------------------------------------------------------------------
// /api/overlay/friends — relationship list, with optional live positions
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayFriends {
    pub share_location: Option<bool>,
    pub limit: Option<u32>,
    pub used: Option<u32>,
    pub friends: Vec<OverlayFriend>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayFriend {
    pub id: Option<String>,
    pub steam_id: Option<String>,
    pub name: Option<String>,
    pub species: Option<String>,
    pub dino_name: Option<String>,
    pub online: Option<bool>,
    pub status: Option<String>,
    pub incoming: Option<bool>,
    pub position: Option<OverlayPosition>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub yaw: Option<f64>,
}

impl OverlayFriend {
    pub fn position_cm3(&self) -> Option<(f64, f64, f64)> {
        if let Some(pos) = self.position {
            return Some((pos.y?, pos.x?, pos.z.unwrap_or(0.0)));
        }
        Some((self.y?, self.x?, self.z.unwrap_or(0.0)))
    }

    pub fn position_cm3_with_calibration(
        &self,
        calibration: Option<&OverlayCalibration>,
    ) -> Option<(f64, f64, f64)> {
        let position = self.position.unwrap_or(OverlayPosition {
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
        });
        let (x_cm, y_cm) = world_point_cm(calibration, position.x?, position.y?)?;
        Some((
            x_cm,
            y_cm,
            position.z.filter(|z| z.is_finite()).unwrap_or(0.0),
        ))
    }

    pub fn heading_deg(&self) -> Option<f64> {
        self.position
            .and_then(|pos| pos.yaw)
            .or(self.yaw)
            .and_then(map_yaw_to_bearing_deg)
    }
}

pub fn get_friends(
    client: &reqwest::blocking::Client,
    token: &str,
) -> Result<OverlayFriends, ApiError> {
    let v = get(client, "/api/overlay/friends", token)?;
    serde_json::from_value(v).map_err(|e| ApiError::Http(format!("/api/overlay/friends: {e}")))
}

// ---------------------------------------------------------------------------
// /api/overlay/skin — live skin editor
// ---------------------------------------------------------------------------

pub fn skin_state(client: &reqwest::blocking::Client, token: &str) -> Result<Value, ApiError> {
    get(client, "/api/overlay/skin", token)
}

pub fn skin_apply(
    client: &reqwest::blocking::Client,
    token: &str,
    palette: Value,
) -> Result<Value, String> {
    post(
        client,
        "/api/overlay/skin/apply",
        token,
        &serde_json::json!({ "palette": palette }),
    )
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// /api/overlay/map — POIs + categories (token mode extra map layers)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayMap {
    pub live_map_enabled: Option<bool>,
    pub allowed: Option<bool>,
    pub calibration: Option<OverlayCalibration>,
    pub markers: Vec<OverlayMarker>,
    pub pois: Vec<OverlayPoi>,
    pub categories: Vec<OverlayCategory>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayMarker {
    pub steam_id: Option<String>,
    pub label: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub yaw: Option<f64>,
    #[serde(rename = "self")]
    pub is_self: bool,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(default)]
pub struct OverlayCalibration {
    pub a: OverlayCalPoint,
    pub b: OverlayCalPoint,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayCalPoint {
    pub u: f64,
    pub v: f64,
    pub world_x: f64,
    pub world_y: f64,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPoi {
    pub id: Option<String>,
    pub name: Option<String>,
    pub category_id: Option<String>,
    pub color: Option<String>,
    pub shape: Option<String>,
    pub size: Option<f64>,
    pub points: Vec<OverlayPoint>,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(default)]
pub struct OverlayPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayCategory {
    pub id: Option<String>,
    pub name: Option<String>,
    pub color: Option<String>,
}

pub fn get_map(client: &reqwest::blocking::Client, token: &str) -> Result<OverlayMap, ApiError> {
    let v = get(client, "/api/overlay/map", token)?;
    serde_json::from_value(v).map_err(|e| ApiError::Http(format!("/api/overlay/map: {e}")))
}

impl OverlayCalibration {
    /// Server world coordinates -> the normalized map frame the server uses.
    /// IslePilot servers may deploy their own calibration, so treating every
    /// raw X/Y pair as the stock Gateway frame can place DinoVietnam markers
    /// hundreds of metres away from the website's own map.
    fn world_to_uv(&self, world_x: f64, world_y: f64) -> Option<(f64, f64)> {
        let (a, b) = (self.a, self.b);
        let dx = b.world_x - a.world_x;
        let dy = b.world_y - a.world_y;
        if ![world_x, world_y, dx, dy]
            .iter()
            .all(|value| value.is_finite())
            || dx.abs() < f64::EPSILON
            || dy.abs() < f64::EPSILON
        {
            return None;
        }
        let u = a.u + (world_x - a.world_x) / dx * (b.u - a.u);
        let v = a.v + (world_y - a.world_y) / dy * (b.v - a.v);
        [u, v]
            .iter()
            .all(|value| value.is_finite())
            .then_some((u, v))
    }

    pub fn heading_deg(&self, world_x: f64, world_y: f64, yaw: f64) -> Option<f64> {
        if !yaw.is_finite() {
            return None;
        }
        let radians = yaw.to_radians();
        let (u0, v0) = self.world_to_uv(world_x, world_y)?;
        let (u1, v1) = self.world_to_uv(
            world_x + 1_000.0 * radians.cos(),
            world_y + 1_000.0 * radians.sin(),
        )?;
        let du = u1 - u0;
        let dv = v1 - v0;
        if du.hypot(dv) < f64::EPSILON {
            return None;
        }
        Some(du.atan2(-dv).to_degrees().rem_euclid(360.0))
    }
}

fn normalized_to_internal_cm(u: f64, v: f64) -> Option<(f64, f64)> {
    if !u.is_finite()
        || !v.is_finite()
        || !(-0.25..=1.25).contains(&u)
        || !(-0.25..=1.25).contains(&v)
    {
        return None;
    }
    let gateway = Calibration::gateway();
    Some(pixel_to_world(
        u * gateway.image_width_px as f64,
        v * gateway.image_height_px as f64,
        gateway,
    ))
}

/// Normalize an IslePilot server's native world frame into this app's
/// established internal Lat/Long convention. Calibration wins when present;
/// older responses without it retain the proven raw-axis swap fallback.
pub fn world_point_cm(
    calibration: Option<&OverlayCalibration>,
    world_x: f64,
    world_y: f64,
) -> Option<(f64, f64)> {
    if !world_x.is_finite() || !world_y.is_finite() {
        return None;
    }
    if let Some((u, v)) = calibration.and_then(|cal| cal.world_to_uv(world_x, world_y)) {
        return normalized_to_internal_cm(u, v);
    }
    Some((world_y, world_x))
}

impl OverlayMarker {
    pub fn position_cm3(
        &self,
        calibration: Option<&OverlayCalibration>,
    ) -> Option<(f64, f64, f64)> {
        let world_x = self.x?;
        let world_y = self.y?;
        let (x_cm, y_cm) = world_point_cm(calibration, world_x, world_y)?;
        Some((x_cm, y_cm, self.z.filter(|z| z.is_finite()).unwrap_or(0.0)))
    }

    pub fn heading_deg(&self, calibration: Option<&OverlayCalibration>) -> Option<f64> {
        let yaw = self.yaw?;
        match (calibration, self.x, self.y) {
            (Some(cal), Some(x), Some(y)) => cal
                .heading_deg(x, y, yaw)
                .or_else(|| map_yaw_to_bearing_deg(yaw)),
            _ => map_yaw_to_bearing_deg(yaw),
        }
    }
}

/// One POI point in game cm, OUR axis convention. POI points have been seen
/// both as u,v fractions and as raw world cm depending on backend version, so
/// disambiguate by magnitude: |coord| <= 1.5 can only be a fraction (1.5 cm
/// off the world origin is not a real POI).
pub fn poi_point_cm(cal: Option<&OverlayCalibration>, p: OverlayPoint) -> Option<(f64, f64)> {
    if p.x.abs() <= 1.5 && p.y.abs() <= 1.5 {
        normalized_to_internal_cm(p.x, p.y)
    } else {
        world_point_cm(cal, p.x, p.y)
    }
}

// ---------------------------------------------------------------------------
// /api/overlay/garage — gacha park/restore/sell/rename
// ---------------------------------------------------------------------------

pub fn garage_list(client: &reqwest::blocking::Client, token: &str) -> Result<Value, ApiError> {
    get(client, "/api/overlay/garage", token)
}

/// POST a garage command and, when it is asynchronous (`commandId` in the
/// response), poll its status to completion: 1.5 s x 40 tries (~60 s), the
/// exact pattern the official app uses.
pub fn garage_command(
    client: &reqwest::blocking::Client,
    token: &str,
    path: &str,
    body: Value,
) -> Result<Value, String> {
    let res = post(client, path, token, &body).map_err(|e| e.to_string())?;
    if let Some(err) = res.get("error").and_then(|e| e.as_str()) {
        return Err(err.to_string());
    }
    let Some(command_id) = res
        .get("commandId")
        .and_then(|c| c.as_str())
        .map(String::from)
    else {
        return Ok(res); // synchronous command (e.g. rename, sell)
    };
    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(1500));
        let s = get(
            client,
            &format!("/api/overlay/garage/status?id={command_id}"),
            token,
        )
        .map_err(|e| e.to_string())?;
        match s.get("status").and_then(|st| st.as_str()) {
            Some("done") => return Ok(s),
            Some("failed") => {
                return Err(s
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("failed")
                    .to_string())
            }
            _ => {} // pending — keep waiting
        }
    }
    Err("timeout".to_string())
}

/// Serialized state for the frontend garage panel.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GarageState {
    pub dinos: Value,
    pub selling_enabled: bool,
    pub live_swap: bool,
    pub currency_name: Option<String>,
    /// Current central-overlay player state. The Garage endpoint itself can
    /// answer while the player is offline, so keep this explicit for the UI.
    pub online: bool,
    pub has_active_dino: bool,
    pub server_name: Option<String>,
}

pub fn garage_state(raw: &Value) -> GarageState {
    let settings = raw.get("settings").cloned().unwrap_or(Value::Null);
    GarageState {
        dinos: raw
            .get("dinos")
            .cloned()
            .unwrap_or_else(|| Value::Array(vec![])),
        selling_enabled: settings
            .get("sellingEnabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        live_swap: settings
            .get("liveSwap")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        currency_name: settings
            .get("currencyName")
            .and_then(|v| v.as_str())
            .map(String::from),
        online: false,
        has_active_dino: false,
        server_name: None,
    }
}

pub fn attach_garage_player_state(state: &mut GarageState, player: &OverlayMe) {
    state.online = player.online == Some(true);
    state.has_active_dino = player.has_data && player.species.is_some();
    state.server_name = player.server.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: &str = include_str!("../../fixtures/islepilot/overlay_me.json");
    const ME_NODATA: &str = include_str!("../../fixtures/islepilot/overlay_me_nodata.json");

    #[test]
    fn overlay_me_maps_to_player_stats() {
        let me: OverlayMe = serde_json::from_str(ME).unwrap();
        assert!(me.has_data);
        let stats = to_player_stats(&me);
        assert_eq!(stats.dino_name.as_deref(), Some("Tyrannosaurus"));
        assert_eq!(stats.online, Some(true));
        assert_eq!(stats.growth.as_deref(), Some("26%"));
        assert!((stats.growth_pct.unwrap() - 26.28).abs() < 0.01);
        let health = stats.health.as_ref().unwrap();
        assert_eq!((health.current, health.max), (Some(49.01), Some(55.12)));
        assert_eq!(health.raw, "49 / 55.1");
        assert_eq!(stats.server.as_deref(), Some("PVN The Isle Viet Nam 01"));
        assert_eq!(stats.female, Some(false));
        let stamina = stats.stamina.as_ref().unwrap();
        assert_eq!(stamina.max, Some(336.52));
        let nut = stats.nutrition.unwrap();
        assert!((nut.carb - 4.04).abs() < 0.001);
        assert_eq!(stats.prime_quests.len(), 2);
        assert_eq!(
            stats.prime_quests[0].text,
            "Visit a Sanctuary as a juvenile"
        );
        assert!(!stats.prime_quests[0].completed);
        assert!(stats.prime_quests[1].completed);
        assert!(stats.looks_logged_in());
    }

    #[test]
    fn position_axis_swap_matches_markers_convention() {
        let me: OverlayMe = serde_json::from_str(ME).unwrap();
        // JSON: x=-263306, y=307415.69 -> ours (x=their y, y=their x).
        assert_eq!(position_cm(&me), Some((307415.69, -263306.0)));
        assert!((position_heading_deg(&me).unwrap() - 70.85).abs() < 0.001);
    }

    #[test]
    fn no_data_response_is_not_an_error() {
        let me: OverlayMe = serde_json::from_str(ME_NODATA).unwrap();
        assert!(!me.has_data);
        assert_eq!(position_cm(&me), None);
        let stats = to_player_stats(&me);
        assert!(!stats.looks_logged_in());
    }

    #[test]
    fn poi_points_convert_from_both_spaces() {
        let cal = OverlayCalibration {
            a: OverlayCalPoint {
                u: 0.0,
                v: 0.0,
                world_x: -100_000.0,
                world_y: -200_000.0,
            },
            b: OverlayCalPoint {
                u: 1.0,
                v: 1.0,
                world_x: 100_000.0,
                world_y: 200_000.0,
            },
        };
        // Fractions stay at the same normalized pixel in our canonical map.
        let gateway = Calibration::gateway();
        let centre = poi_point_cm(Some(&cal), OverlayPoint { x: 0.5, y: 0.5 }).unwrap();
        let centre_px = overlay_core::world_to_pixel(centre.0, centre.1, gateway);
        assert!((centre_px.0 - gateway.image_width_px as f64 * 0.5).abs() < 1e-6);
        assert!((centre_px.1 - gateway.image_height_px as f64 * 0.5).abs() < 1e-6);

        // A raw point is projected through the server calibration first:
        // x=50k -> u=.75 and y=-30k -> v=.425.
        let raw = poi_point_cm(
            Some(&cal),
            OverlayPoint {
                x: 50_000.0,
                y: -30_000.0,
            },
        )
        .unwrap();
        let raw_px = overlay_core::world_to_pixel(raw.0, raw.1, gateway);
        assert!((raw_px.0 - gateway.image_width_px as f64 * 0.75).abs() < 1e-6);
        assert!((raw_px.1 - gateway.image_height_px as f64 * 0.425).abs() < 1e-6);

        // Older uncalibrated world responses keep the established axis swap.
        assert_eq!(
            poi_point_cm(
                None,
                OverlayPoint {
                    x: 50_000.0,
                    y: -30_000.0
                }
            ),
            Some((-30_000.0, 50_000.0))
        );
        // An already-normalized point does not require calibration metadata.
        let uncalibrated_centre = poi_point_cm(None, OverlayPoint { x: 0.5, y: 0.5 }).unwrap();
        assert_eq!(uncalibrated_centre, centre);
    }

    #[test]
    fn map_markers_deserialize_and_follow_server_calibration() {
        let map: OverlayMap = serde_json::from_value(serde_json::json!({
            "liveMapEnabled": true,
            "allowed": true,
            "calibration": {
                "a": {"worldX": -505000.0, "worldY": -607000.0, "u": 0.0, "v": 0.0},
                "b": {"worldX": 607000.0, "worldY": 509000.0, "u": 1.0, "v": 1.0}
            },
            "markers": [{
                "steamId": "76561198000000001",
                "label": "You",
                "self": true,
                "x": 232414.14,
                "y": -17468.84,
                "z": 23732.46,
                "yaw": 0.0
            }]
        }))
        .unwrap();
        assert_eq!(map.markers.len(), 1);
        assert!(map.markers[0].is_self);
        let position = map.markers[0]
            .position_cm3(map.calibration.as_ref())
            .unwrap();
        let pixel = overlay_core::world_to_pixel(position.0, position.1, Calibration::gateway());
        assert!((pixel.0 - 5_172.51).abs() < 0.02);
        assert!((pixel.1 - 4_129.36).abs() < 0.02);
        assert!(
            (map.markers[0]
                .heading_deg(map.calibration.as_ref())
                .unwrap()
                - 90.0)
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn garage_state_reads_settings_flags() {
        let raw: Value = serde_json::json!({
            "dinos": [{"id": "d1", "species": "Carnotaurus"}],
            "settings": {"liveSwap": true, "sellingEnabled": false, "currencyName": "Points"}
        });
        let g = garage_state(&raw);
        assert!(g.live_swap);
        assert!(!g.selling_enabled);
        assert_eq!(g.currency_name.as_deref(), Some("Points"));
        assert_eq!(g.dinos.as_array().unwrap().len(), 1);
        assert!(!g.online);
    }

    #[test]
    fn garage_state_tracks_the_active_server_from_overlay_me() {
        let raw: Value = serde_json::json!({ "settings": {}, "dinos": [] });
        let mut state = garage_state(&raw);
        let player = OverlayMe {
            has_data: true,
            online: Some(true),
            species: Some("Deinosuchus".to_string()),
            server: Some("DinoVietnam VIP".to_string()),
            ..OverlayMe::default()
        };
        attach_garage_player_state(&mut state, &player);
        assert!(state.online);
        assert!(state.has_active_dino);
        assert_eq!(state.server_name.as_deref(), Some("DinoVietnam VIP"));
    }
}
