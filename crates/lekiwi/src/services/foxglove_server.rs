use std::sync::Arc;

use anyhow::Result;
use foxglove::websocket::{Capability, Client, ClientChannel, ServerListener};
use tokio::sync::watch;

use crate::bus::Bus;
use crate::messages::{EstopCommand, LogControl, LogStatus, SkillCommand, VelocityCommand};
use crate::telemetry::{
    Telemetry, TOPIC_CMD_ESTOP, TOPIC_CMD_SKILL, TOPIC_CMD_VELOCITY, TOPIC_LOG_CONTROL,
};
use crate::utils::now_nanos;

#[derive(Debug, Clone)]
pub struct FoxgloveConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
}

pub async fn run(
    config: FoxgloveConfig,
    ctx: Arc<foxglove::Context>,
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let listener = Arc::new(FoxgloveListener { bus, telemetry });
    let server = ctx
        .websocket_server()
        .name(config.name)
        .bind(config.host, config.port)
        .capabilities([Capability::ClientPublish])
        .supported_encodings(["json"])
        .listener(listener);

    let handle = server.start().await?;
    tracing::info!("Foxglove server ready: {}", handle.app_url());

    wait_for_shutdown(&mut shutdown).await;
    let shutdown_handle = handle.stop();
    shutdown_handle.wait().await;
    Ok(())
}

struct FoxgloveListener {
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
}

impl ServerListener for FoxgloveListener {
    fn on_message_data(&self, _client: Client, channel: &ClientChannel, payload: &[u8]) {
        if channel.encoding != "json" {
            tracing::warn!(
                "Unsupported client encoding '{}' on {}",
                channel.encoding,
                channel.topic
            );
            return;
        }

        match channel.topic.as_str() {
            TOPIC_CMD_VELOCITY => match serde_json::from_slice::<VelocityCommand>(payload) {
                Ok(mut cmd) => {
                    if cmd.timestamp_ns == 0 {
                        cmd.timestamp_ns = now_nanos();
                    }
                    let _ = self.bus.cmd_in.send(cmd);
                }
                Err(err) => tracing::warn!("Invalid /cmd/velocity payload: {err}"),
            },
            TOPIC_CMD_SKILL => match serde_json::from_slice::<SkillCommand>(payload) {
                Ok(mut cmd) => {
                    if cmd.timestamp_ns == 0 {
                        cmd.timestamp_ns = now_nanos();
                    }
                    self.telemetry.log_cmd_skill(&cmd);
                    let _ = self.bus.cmd_skill.send(cmd);
                }
                Err(err) => tracing::warn!("Invalid /cmd/skill payload: {err}"),
            },
            TOPIC_CMD_ESTOP => match serde_json::from_slice::<EstopCommand>(payload) {
                Ok(mut cmd) => {
                    if cmd.timestamp_ns == 0 {
                        cmd.timestamp_ns = now_nanos();
                    }
                    let _ = self.bus.cmd_estop.send(cmd);
                }
                Err(err) => tracing::warn!("Invalid /cmd/estop payload: {err}"),
            },
            TOPIC_LOG_CONTROL => match serde_json::from_slice::<LogControl>(payload) {
                Ok(mut cmd) => {
                    if cmd.timestamp_ns == 0 {
                        cmd.timestamp_ns = now_nanos();
                    }
                    self.telemetry.log_log_control(&cmd);
                    let _ = self.bus.log_control.send(cmd);
                }
                Err(err) => tracing::warn!("Invalid /log/control payload: {err}"),
            },
            _ => {
                tracing::debug!("Ignoring client message on {}", channel.topic);
            }
        }
    }

    fn on_client_advertise(&self, _client: Client, channel: &ClientChannel) {
        tracing::info!(
            "Client advertised channel {} ({})",
            channel.topic,
            channel.encoding
        );
    }

    fn on_client_connect(&self) {
        let status = LogStatus {
            timestamp_ns: now_nanos(),
            active: false,
            file_path: None,
            size_bytes: None,
            duration_s: None,
        };
        self.telemetry.log_log_status(&status);
    }
}

async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow() {
            break;
        }
        if shutdown.changed().await.is_err() {
            break;
        }
    }
}
