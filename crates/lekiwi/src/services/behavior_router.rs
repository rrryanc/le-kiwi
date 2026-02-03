use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::watch;

use crate::bus::Bus;
use crate::config::{LimitsConfig, SafetyConfig};
use crate::messages::{DiagnosticStatus, Diagnostics, VelocityCommand};
use crate::telemetry::Telemetry;
use crate::utils::now_nanos;

const TICK_HZ: u64 = 50;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    limits: LimitsConfig,
    safety: SafetyConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut cmd_rx = bus.cmd_in.subscribe();
    let mut estop_rx = bus.cmd_estop.subscribe();

    let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_HZ));
    let start = Instant::now();
    let mut last_cmd = VelocityCommand::zero("behavior_router", now_nanos());
    let mut last_seen = Instant::now();
    let mut last_output = last_cmd.clone();
    let mut last_update = Instant::now();
    let mut last_diag = Instant::now();
    let mut estop_active = false;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let now = Instant::now();
                let dt = (now - last_update).as_secs_f32().max(1e-3);
                last_update = now;

                let timed_out = now.duration_since(last_seen).as_millis() as u64 > safety.command_timeout_ms;
                let mut target = if timed_out {
                    VelocityCommand::zero("timeout", now_nanos())
                } else {
                    clamp_velocity(&last_cmd, &limits, now_nanos())
                };

                if estop_active && safety.estop_enabled {
                    target = VelocityCommand::zero("estop", now_nanos());
                }

                let output = apply_rate_limits(&last_output, &target, &limits, dt);
                last_output = output.clone();

                telemetry.log_cmd_velocity(&output);
                let _ = bus.cmd_out.send(output);

                if last_diag.elapsed() >= Duration::from_secs(1) {
                    last_diag = Instant::now();
                    let mut warnings = Vec::new();
                    if timed_out {
                        warnings.push("command_timeout".to_string());
                    }
                    if estop_active {
                        warnings.push("estop_active".to_string());
                    }

                    let status = if warnings.is_empty() {
                        DiagnosticStatus::Ok
                    } else {
                        DiagnosticStatus::Warn
                    };

                    let diag = Diagnostics {
                        timestamp_ns: now_nanos(),
                        status,
                        warnings,
                        last_error: None,
                        uptime_s: start.elapsed().as_secs_f64(),
                    };
                    telemetry.log_diagnostics(&diag);
                    let _ = bus.diagnostics.send(diag);
                }
            }
            Ok(cmd) = cmd_rx.recv() => {
                last_cmd = cmd;
                last_seen = Instant::now();
            }
            Ok(cmd) = estop_rx.recv() => {
                estop_active = cmd.enabled;
                telemetry.log_cmd_estop(&cmd);
            }
            _ = shutdown.changed() => {
                break;
            }
        }
    }

    Ok(())
}

fn clamp_velocity(cmd: &VelocityCommand, limits: &LimitsConfig, timestamp_ns: u64) -> VelocityCommand {
    let mut clamped = cmd.clone();
    clamped.timestamp_ns = timestamp_ns;
    clamped.vx_m_s = clamp(cmd.vx_m_s, limits.max_vx_m_s);
    clamped.vy_m_s = clamp(cmd.vy_m_s, limits.max_vy_m_s);
    clamped.omega_rad_s = clamp(cmd.omega_rad_s, limits.max_omega_rad_s);
    clamped.source = "behavior_router".to_string();
    clamped
}

fn apply_rate_limits(
    last: &VelocityCommand,
    target: &VelocityCommand,
    limits: &LimitsConfig,
    dt: f32,
) -> VelocityCommand {
    let max_dv = limits.max_accel_m_s2 * dt;
    let max_dw = limits.max_alpha_rad_s2 * dt;

    let mut output = target.clone();
    output.vx_m_s = limit_delta(last.vx_m_s, target.vx_m_s, max_dv);
    output.vy_m_s = limit_delta(last.vy_m_s, target.vy_m_s, max_dv);
    output.omega_rad_s = limit_delta(last.omega_rad_s, target.omega_rad_s, max_dw);
    output
}

fn clamp(value: f32, max: f32) -> f32 {
    if value > max {
        max
    } else if value < -max {
        -max
    } else {
        value
    }
}

fn limit_delta(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta > max_delta {
        current + max_delta
    } else if delta < -max_delta {
        current - max_delta
    } else {
        target
    }
}
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::watch;
use tokio::time::{interval, Instant};

use crate::bus::Bus;
use crate::config::{LimitsConfig, SafetyConfig};
use crate::messages::{DiagnosticStatus, Diagnostics, EstopCommand, VelocityCommand};
use crate::telemetry::Telemetry;
use crate::utils::now_nanos;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    limits: LimitsConfig,
    safety: SafetyConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut cmd_rx = bus.cmd_in.subscribe();
    let mut estop_rx = bus.cmd_estop.subscribe();
    let mut ticker = interval(Duration::from_millis(20));
    let mut last_cmd = VelocityCommand::zero("behavior_router", now_nanos());
    let mut last_seen = Instant::now();
    let mut last_output = last_cmd.clone();
    let mut estop_active = safety.estop_enabled && false;
    let mut last_diag = Instant::now();
    let start_time = Instant::now();

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = Instant::now();
                let timed_out = now.duration_since(last_seen).as_millis() as u64 > safety.command_timeout_ms;
                let mut target = if timed_out {
                    VelocityCommand::zero("timeout", now_nanos())
                } else {
                    clamp_velocity(&last_cmd, &limits, now_nanos())
                };

                if estop_active {
                    target = VelocityCommand::zero("estop", now_nanos());
                }

                let dt = 0.02;
                target.vx_m_s = rate_limit(target.vx_m_s, last_output.vx_m_s, limits.max_accel_m_s2 * dt);
                target.vy_m_s = rate_limit(target.vy_m_s, last_output.vy_m_s, limits.max_accel_m_s2 * dt);
                target.omega_rad_s = rate_limit(
                    target.omega_rad_s,
                    last_output.omega_rad_s,
                    limits.max_alpha_rad_s2 * dt,
                );

                last_output = target.clone();
                telemetry.log_cmd_velocity(&target);
                let _ = bus.cmd_out.send(target);

                if last_diag.elapsed() >= Duration::from_secs(1) {
                    let mut warnings = Vec::new();
                    let status = if estop_active {
                        warnings.push("estop_active".to_string());
                        DiagnosticStatus::Error
                    } else if timed_out {
                        warnings.push("command_timeout".to_string());
                        DiagnosticStatus::Warn
                    } else {
                        DiagnosticStatus::Ok
                    };
                    let diag = Diagnostics {
                        timestamp_ns: now_nanos(),
                        status,
                        warnings,
                        last_error: None,
                        uptime_s: start_time.elapsed().as_secs_f64(),
                    };
                    telemetry.log_diagnostics(&diag);
                    let _ = bus.diagnostics.send(diag);
                    last_diag = Instant::now();
                }
            }
            Ok(cmd) = cmd_rx.recv() => {
                last_cmd = cmd;
                last_seen = Instant::now();
            }
            Ok(estop) = estop_rx.recv() => {
                estop_active = estop.enabled;
                telemetry.log_cmd_estop(&estop);
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn clamp_velocity(cmd: &VelocityCommand, limits: &LimitsConfig, timestamp_ns: u64) -> VelocityCommand {
    let mut out = cmd.clone();
    out.timestamp_ns = timestamp_ns;
    out.vx_m_s = clamp(cmd.vx_m_s, limits.max_vx_m_s);
    out.vy_m_s = clamp(cmd.vy_m_s, limits.max_vy_m_s);
    out.omega_rad_s = clamp(cmd.omega_rad_s, limits.max_omega_rad_s);
    out
}

fn clamp(value: f32, max_abs: f32) -> f32 {
    value.max(-max_abs).min(max_abs)
}

fn rate_limit(target: f32, current: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() > max_delta {
        current + delta.signum() * max_delta
    } else {
        target
    }
}
