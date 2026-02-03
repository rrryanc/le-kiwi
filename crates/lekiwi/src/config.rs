use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub robot: RobotConfig,
    pub cameras: CamerasConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load(robot_path: &Path, cameras_path: &Path, logging_path: &Path) -> Result<Self> {
        let robot = read_yaml(robot_path).context("failed to read robot config")?;
        let cameras = read_yaml(cameras_path).context("failed to read cameras config")?;
        let logging = read_yaml(logging_path).context("failed to read logging config")?;
        Ok(Self {
            robot,
            cameras,
            logging,
        })
    }
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("unable to read {}", path.display()))?;
    let config = serde_yaml::from_str(&contents)
        .with_context(|| format!("unable to parse {}", path.display()))?;
    Ok(config)
}

#[derive(Debug, Clone, Deserialize)]
pub struct RobotConfig {
    pub robot: RobotInfo,
    pub battery: BatteryConfig,
    pub servo_bus: ServoBusConfig,
    pub drive: DriveConfig,
    pub limits: LimitsConfig,
    pub safety: SafetyConfig,
    pub frames: FramesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RobotInfo {
    pub name: String,
    pub platform: String,
    pub compute: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatteryConfig {
    pub r#type: String,
    pub nominal_voltage_v: f32,
    pub low_voltage_v: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServoBusConfig {
    pub port: String,
    pub baud_rate: u32,
    pub protocol: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveConfig {
    pub wheel_radius_m: f32,
    pub wheel_distance_m: f32,
    pub wheel_mounts: Vec<WheelMount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WheelMount {
    pub name: String,
    pub angle_deg: f32,
    pub servo_id: u8,
    pub direction: i8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitsConfig {
    pub max_vx_m_s: f32,
    pub max_vy_m_s: f32,
    pub max_omega_rad_s: f32,
    pub max_accel_m_s2: f32,
    pub max_alpha_rad_s2: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SafetyConfig {
    pub command_timeout_ms: u64,
    pub estop_enabled: bool,
    pub servo_temp_limit_c: f32,
    pub low_battery_stop: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FramesConfig {
    pub base_link: String,
    pub odom: String,
    pub map: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CamerasConfig {
    pub cameras: Vec<CameraConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CameraConfig {
    pub name: String,
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub format: String,
    pub auto_exposure: bool,
    pub auto_focus: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingSettings {
    pub directory: String,
    pub include_cameras: bool,
    pub rotate_on_size_mb: u64,
    pub default_topics: Vec<String>,
}
