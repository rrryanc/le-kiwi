use std::sync::Arc;

use anyhow::Result;
use foxglove::{ChannelBuilder, Context, PartialMetadata, RawChannel};
use serde::Serialize;

use crate::messages::{
    CameraFrame, Diagnostics, EstopCommand, LogControl, LogStatus, Odometry, PowerState,
    ServoStateArray, SkillCommand, VelocityCommand,
};

pub const TOPIC_CMD_VELOCITY: &str = "/cmd/velocity";
pub const TOPIC_CMD_SKILL: &str = "/cmd/skill";
pub const TOPIC_CMD_ESTOP: &str = "/cmd/estop";
pub const TOPIC_STATE_ODOM: &str = "/state/odometry";
pub const TOPIC_STATE_SERVOS: &str = "/state/servos";
pub const TOPIC_STATE_POWER: &str = "/state/power";
pub const TOPIC_SYSTEM_DIAG: &str = "/system/diagnostics";
pub const TOPIC_CAMERA_BASE: &str = "/sensors/camera/base";
pub const TOPIC_CAMERA_WRIST: &str = "/sensors/camera/wrist";
pub const TOPIC_LOG_CONTROL: &str = "/log/control";
pub const TOPIC_LOG_STATUS: &str = "/log/status";

#[derive(Clone)]
pub struct Telemetry {
    ctx: Arc<Context>,
    cmd_velocity: Arc<RawChannel>,
    cmd_skill: Arc<RawChannel>,
    cmd_estop: Arc<RawChannel>,
    odometry: Arc<RawChannel>,
    servos: Arc<RawChannel>,
    power: Arc<RawChannel>,
    diagnostics: Arc<RawChannel>,
    camera_base: Arc<RawChannel>,
    camera_wrist: Arc<RawChannel>,
    log_control: Arc<RawChannel>,
    log_status: Arc<RawChannel>,
}

impl Telemetry {
    pub fn new(ctx: &Arc<Context>) -> Result<Self> {
        Ok(Self {
            ctx: ctx.clone(),
            cmd_velocity: build_json_channel(ctx, TOPIC_CMD_VELOCITY)?,
            cmd_skill: build_json_channel(ctx, TOPIC_CMD_SKILL)?,
            cmd_estop: build_json_channel(ctx, TOPIC_CMD_ESTOP)?,
            odometry: build_json_channel(ctx, TOPIC_STATE_ODOM)?,
            servos: build_json_channel(ctx, TOPIC_STATE_SERVOS)?,
            power: build_json_channel(ctx, TOPIC_STATE_POWER)?,
            diagnostics: build_json_channel(ctx, TOPIC_SYSTEM_DIAG)?,
            camera_base: build_json_channel(ctx, TOPIC_CAMERA_BASE)?,
            camera_wrist: build_json_channel(ctx, TOPIC_CAMERA_WRIST)?,
            log_control: build_json_channel(ctx, TOPIC_LOG_CONTROL)?,
            log_status: build_json_channel(ctx, TOPIC_LOG_STATUS)?,
        })
    }

    pub fn context(&self) -> Arc<Context> {
        self.ctx.clone()
    }

    pub fn log_cmd_velocity(&self, msg: &VelocityCommand) {
        log_json(&self.cmd_velocity, msg, msg.timestamp_ns);
    }

    pub fn log_cmd_skill(&self, msg: &SkillCommand) {
        log_json(&self.cmd_skill, msg, msg.timestamp_ns);
    }

    pub fn log_cmd_estop(&self, msg: &EstopCommand) {
        log_json(&self.cmd_estop, msg, msg.timestamp_ns);
    }

    pub fn log_odometry(&self, msg: &Odometry) {
        log_json(&self.odometry, msg, msg.timestamp_ns);
    }

    pub fn log_servo_state(&self, msg: &ServoStateArray) {
        log_json(&self.servos, msg, msg.timestamp_ns);
    }

    pub fn log_power_state(&self, msg: &PowerState) {
        log_json(&self.power, msg, msg.timestamp_ns);
    }

    pub fn log_diagnostics(&self, msg: &Diagnostics) {
        log_json(&self.diagnostics, msg, msg.timestamp_ns);
    }

    pub fn log_camera_frame(&self, msg: &CameraFrame) {
        let channel = if msg.camera_name == "wrist" {
            &self.camera_wrist
        } else {
            &self.camera_base
        };
        log_json(channel, msg, msg.timestamp_ns);
    }

    pub fn log_log_control(&self, msg: &LogControl) {
        log_json(&self.log_control, msg, msg.timestamp_ns);
    }

    pub fn log_log_status(&self, msg: &LogStatus) {
        log_json(&self.log_status, msg, msg.timestamp_ns);
    }
}

fn build_json_channel(ctx: &Arc<Context>, topic: &str) -> Result<Arc<RawChannel>> {
    let channel = ChannelBuilder::new(topic)
        .context(ctx)
        .message_encoding("json")
        .build_raw()?;
    Ok(channel)
}

fn log_json<T: Serialize>(channel: &RawChannel, msg: &T, timestamp_ns: u64) {
    match serde_json::to_vec(msg) {
        Ok(encoded) => {
            channel.log_with_meta(&encoded, PartialMetadata::with_log_time(timestamp_ns));
        }
        Err(err) => {
            tracing::warn!("Failed to serialize {}: {err}", channel.topic());
        }
    }
}

