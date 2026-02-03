use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityCommand {
    #[serde(default)]
    pub timestamp_ns: u64,
    pub vx_m_s: f32,
    pub vy_m_s: f32,
    pub omega_rad_s: f32,
    pub source: String,
    pub priority: i32,
}

impl VelocityCommand {
    pub fn zero(source: impl Into<String>, timestamp_ns: u64) -> Self {
        Self {
            timestamp_ns,
            vx_m_s: 0.0,
            vy_m_s: 0.0,
            omega_rad_s: 0.0,
            source: source.into(),
            priority: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    #[serde(default)]
    pub timestamp_ns: u64,
    pub request_id: String,
    pub skill_name: String,
    pub params: Value,
    pub timeout_s: f32,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstopCommand {
    #[serde(default)]
    pub timestamp_ns: u64,
    pub enabled: bool,
    pub reason: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogControl {
    #[serde(default)]
    pub timestamp_ns: u64,
    pub action: LogAction,
    pub topics: Option<Vec<String>>,
    pub session_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatus {
    pub timestamp_ns: u64,
    pub active: bool,
    pub file_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_s: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Odometry {
    pub timestamp_ns: u64,
    pub x_m: f64,
    pub y_m: f64,
    pub theta_rad: f64,
    pub vx_m_s: f32,
    pub vy_m_s: f32,
    pub omega_rad_s: f32,
    pub frame_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoState {
    pub id: u8,
    pub name: String,
    pub position_rad: f32,
    pub velocity_rad_s: f32,
    pub load: f32,
    pub temperature_c: f32,
    pub voltage_v: f32,
    pub error_flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoStateArray {
    pub timestamp_ns: u64,
    pub servos: Vec<ServoState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerState {
    pub timestamp_ns: u64,
    pub battery_voltage_v: f32,
    pub battery_percent: f32,
    pub low_battery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub timestamp_ns: u64,
    pub status: DiagnosticStatus,
    pub warnings: Vec<String>,
    pub last_error: Option<String>,
    pub uptime_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub timestamp_ns: u64,
    pub camera_name: String,
    pub frame_id: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub data_base64: String,
}
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityCommand {
    pub timestamp_ns: u64,
    pub vx_m_s: f32,
    pub vy_m_s: f32,
    pub omega_rad_s: f32,
    pub source: String,
    pub priority: i32,
}

impl VelocityCommand {
    pub fn zero(source: &str, timestamp_ns: u64) -> Self {
        Self {
            timestamp_ns,
            vx_m_s: 0.0,
            vy_m_s: 0.0,
            omega_rad_s: 0.0,
            source: source.to_string(),
            priority: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    pub request_id: String,
    pub skill_name: String,
    pub params: Value,
    pub timeout_s: f32,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstopCommand {
    pub timestamp_ns: u64,
    pub enabled: bool,
    pub reason: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogControl {
    pub timestamp_ns: u64,
    pub action: LogAction,
    pub topics: Option<Vec<String>>,
    pub session_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatus {
    pub timestamp_ns: u64,
    pub active: bool,
    pub file_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_s: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Odometry {
    pub timestamp_ns: u64,
    pub x_m: f64,
    pub y_m: f64,
    pub theta_rad: f64,
    pub vx_m_s: f32,
    pub vy_m_s: f32,
    pub omega_rad_s: f32,
    pub frame_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoState {
    pub id: u8,
    pub name: String,
    pub position_rad: f32,
    pub velocity_rad_s: f32,
    pub load: f32,
    pub temperature_c: f32,
    pub voltage_v: f32,
    pub error_flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoStateArray {
    pub timestamp_ns: u64,
    pub servos: Vec<ServoState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerState {
    pub timestamp_ns: u64,
    pub battery_voltage_v: f32,
    pub battery_percent: f32,
    pub low_battery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub timestamp_ns: u64,
    pub status: DiagnosticStatus,
    pub warnings: Vec<String>,
    pub last_error: Option<String>,
    pub uptime_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub timestamp_ns: u64,
    pub camera_name: String,
    pub frame_id: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub data_base64: String,
}
