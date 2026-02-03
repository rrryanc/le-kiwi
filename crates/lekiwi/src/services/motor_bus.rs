use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::watch;

use crate::bus::Bus;
use crate::config::{BatteryConfig, DriveConfig, RobotConfig};
use crate::messages::{PowerState, ServoState, ServoStateArray, VelocityCommand};
use crate::telemetry::Telemetry;
use crate::utils::now_nanos;

const SERVO_PUBLISH_HZ: u64 = 10;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    robot: RobotConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut cmd_rx = bus.cmd_out.subscribe();
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / SERVO_PUBLISH_HZ));
    let mut last_cmd = VelocityCommand::zero("motor_bus", now_nanos());
    let mut last_power = Instant::now();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let servo_state = build_servo_state(&last_cmd, &robot.drive, &robot.battery);
                telemetry.log_servo_state(&servo_state);
                let _ = bus.servos.send(servo_state);

                if last_power.elapsed() >= Duration::from_secs(1) {
                    last_power = Instant::now();
                    let power = build_power_state(&robot.battery);
                    telemetry.log_power_state(&power);
                    let _ = bus.power.send(power);
                }
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

fn build_servo_state(
    cmd: &VelocityCommand,
    drive: &DriveConfig,
    battery: &BatteryConfig,
) -> ServoStateArray {
    let mut servos = Vec::with_capacity(drive.wheel_mounts.len());
    let timestamp_ns = now_nanos();

    let wheel_radius = drive.wheel_radius_m.max(1e-4);
    let wheel_distance = drive.wheel_distance_m;

    for mount in &drive.wheel_mounts {
        let angle = mount.angle_deg.to_radians();
        let v = -angle.sin() * cmd.vx_m_s + angle.cos() * cmd.vy_m_s + wheel_distance * cmd.omega_rad_s;
        let omega = (v / wheel_radius) * mount.direction as f32;

        servos.push(ServoState {
            id: mount.servo_id,
            name: mount.name.clone(),
            position_rad: 0.0,
            velocity_rad_s: omega,
            load: 0.0,
            temperature_c: 0.0,
            voltage_v: battery.nominal_voltage_v,
            error_flags: 0,
        });
    }

    ServoStateArray {
        timestamp_ns,
        servos,
    }
}

fn build_power_state(battery: &BatteryConfig) -> PowerState {
    let voltage = battery.nominal_voltage_v;
    let percent = if battery.nominal_voltage_v > battery.low_voltage_v {
        ((voltage - battery.low_voltage_v)
            / (battery.nominal_voltage_v - battery.low_voltage_v))
            .clamp(0.0, 1.0)
    } else {
        1.0
    };

    PowerState {
        timestamp_ns: now_nanos(),
        battery_voltage_v: voltage,
        battery_percent: percent,
        low_battery: voltage <= battery.low_voltage_v,
    }
}
