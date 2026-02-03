use tokio::sync::broadcast;

use crate::messages::{
    CameraFrame, Diagnostics, EstopCommand, LogControl, LogStatus, Odometry, PowerState,
    ServoStateArray, SkillCommand, VelocityCommand,
};

const CHANNEL_SIZE: usize = 64;

#[derive(Debug)]
pub struct Bus {
    pub cmd_in: broadcast::Sender<VelocityCommand>,
    pub cmd_out: broadcast::Sender<VelocityCommand>,
    pub cmd_skill: broadcast::Sender<SkillCommand>,
    pub cmd_estop: broadcast::Sender<EstopCommand>,
    pub log_control: broadcast::Sender<LogControl>,
    pub log_status: broadcast::Sender<LogStatus>,
    pub odometry: broadcast::Sender<Odometry>,
    pub servos: broadcast::Sender<ServoStateArray>,
    pub power: broadcast::Sender<PowerState>,
    pub diagnostics: broadcast::Sender<Diagnostics>,
    pub camera: broadcast::Sender<CameraFrame>,
}

impl Bus {
    pub fn new() -> Self {
        let (cmd_in, _) = broadcast::channel(CHANNEL_SIZE);
        let (cmd_out, _) = broadcast::channel(CHANNEL_SIZE);
        let (cmd_skill, _) = broadcast::channel(CHANNEL_SIZE);
        let (cmd_estop, _) = broadcast::channel(CHANNEL_SIZE);
        let (log_control, _) = broadcast::channel(CHANNEL_SIZE);
        let (log_status, _) = broadcast::channel(CHANNEL_SIZE);
        let (odometry, _) = broadcast::channel(CHANNEL_SIZE);
        let (servos, _) = broadcast::channel(CHANNEL_SIZE);
        let (power, _) = broadcast::channel(CHANNEL_SIZE);
        let (diagnostics, _) = broadcast::channel(CHANNEL_SIZE);
        let (camera, _) = broadcast::channel(CHANNEL_SIZE);

        Self {
            cmd_in,
            cmd_out,
            cmd_skill,
            cmd_estop,
            log_control,
            log_status,
            odometry,
            servos,
            power,
            diagnostics,
            camera,
        }
    }
}
