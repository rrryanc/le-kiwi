use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::watch;

use crate::bus::Bus;
use crate::config::FramesConfig;
use crate::messages::{Odometry, VelocityCommand};
use crate::telemetry::Telemetry;
use crate::utils::now_nanos;

const TICK_HZ: u64 = 50;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    frames: FramesConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut cmd_rx = bus.cmd_out.subscribe();
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_HZ));

    let mut last_cmd = VelocityCommand::zero("state_estimator", now_nanos());
    let mut pose_x = 0.0f64;
    let mut pose_y = 0.0f64;
    let mut pose_theta = 0.0f64;
    let mut last_update = Instant::now();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let now = Instant::now();
                let dt = (now - last_update).as_secs_f64().max(1e-3);
                last_update = now;

                let vx = last_cmd.vx_m_s as f64;
                let vy = last_cmd.vy_m_s as f64;
                let omega = last_cmd.omega_rad_s as f64;

                let cos_t = pose_theta.cos();
                let sin_t = pose_theta.sin();
                let dx = (vx * cos_t - vy * sin_t) * dt;
                let dy = (vx * sin_t + vy * cos_t) * dt;

                pose_x += dx;
                pose_y += dy;
                pose_theta = normalize_angle(pose_theta + omega * dt);

                let odom = Odometry {
                    timestamp_ns: now_nanos(),
                    x_m: pose_x,
                    y_m: pose_y,
                    theta_rad: pose_theta,
                    vx_m_s: last_cmd.vx_m_s,
                    vy_m_s: last_cmd.vy_m_s,
                    omega_rad_s: last_cmd.omega_rad_s,
                    frame_id: frames.odom.clone(),
                };
                telemetry.log_odometry(&odom);
                let _ = bus.odometry.send(odom);
            }
            Ok(cmd) = cmd_rx.recv() => {
                last_cmd = cmd;
            }
            _ = shutdown.changed() => {
                break;
            }
        }
    }

    Ok(())
}

fn normalize_angle(mut theta: f64) -> f64 {
    while theta > std::f64::consts::PI {
        theta -= 2.0 * std::f64::consts::PI;
    }
    while theta < -std::f64::consts::PI {
        theta += 2.0 * std::f64::consts::PI;
    }
    theta
}
