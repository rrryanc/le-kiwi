use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use foxglove::{McapCompression, McapWriteOptions, McapWriterHandle};
use tokio::sync::watch;

use crate::bus::Bus;
use crate::config::LoggingSettings;
use crate::messages::{LogAction, LogStatus};
use crate::telemetry::{Telemetry, TOPIC_CAMERA_BASE, TOPIC_CAMERA_WRIST};
use crate::utils::now_nanos;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    ctx: Arc<foxglove::Context>,
    logging: LoggingSettings,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut rx = bus.log_control.subscribe();
    let mut active: Option<McapWriterHandle<std::io::BufWriter<std::fs::File>>> = None;
    let mut active_path: Option<PathBuf> = None;
    let mut start_time: Option<Instant> = None;

    loop {
        tokio::select! {
            Ok(cmd) = rx.recv() => {
                match cmd.action {
                    LogAction::Start => {
                        if active.is_some() {
                            tracing::warn!("MCAP logging already active");
                            continue;
                        }
                        let path = build_log_path(&logging.directory, cmd.session_name.as_deref())?;
                        let topics = resolve_topics(&logging, cmd.topics);
                        let handle = create_writer(&ctx, &path, topics)?;

                        start_time = Some(Instant::now());
                        active_path = Some(path.clone());
                        active = Some(handle);

                        let status = LogStatus {
                            timestamp_ns: now_nanos(),
                            active: true,
                            file_path: Some(path.display().to_string()),
                            size_bytes: None,
                            duration_s: None,
                        };
                        telemetry.log_log_status(&status);
                        let _ = bus.log_status.send(status);
                    }
                    LogAction::Stop => {
                        if let Some(handle) = active.take() {
                            let _ = handle.close();
                        }
                        let duration_s = start_time.take().map(|t| t.elapsed().as_secs_f64());
                        let status = LogStatus {
                            timestamp_ns: now_nanos(),
                            active: false,
                            file_path: active_path.take().map(|p| p.display().to_string()),
                            size_bytes: None,
                            duration_s,
                        };
                        telemetry.log_log_status(&status);
                        let _ = bus.log_status.send(status);
                    }
                }
            }
            _ = shutdown.changed() => {
                break;
            }
        }
    }

    if let Some(handle) = active {
        let _ = handle.close();
    }

    Ok(())
}

fn resolve_topics(logging: &LoggingSettings, requested: Option<Vec<String>>) -> HashSet<String> {
    let mut topics: HashSet<String> = requested
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| logging.default_topics.clone())
        .into_iter()
        .collect();

    if logging.include_cameras {
        topics.insert(TOPIC_CAMERA_BASE.to_string());
        topics.insert(TOPIC_CAMERA_WRIST.to_string());
    }

    topics
}

fn build_log_path(dir: &str, session: Option<&str>) -> Result<PathBuf> {
    let name = session
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("session-{}", now_nanos()));
    let filename = format!("{name}.mcap");

    fs::create_dir_all(dir).with_context(|| format!("unable to create {}", dir))?;
    Ok(Path::new(dir).join(filename))
}

fn create_writer(
    ctx: &Arc<foxglove::Context>,
    path: &Path,
    topics: HashSet<String>,
) -> Result<McapWriterHandle<std::io::BufWriter<std::fs::File>>> {
    let options = McapWriteOptions::default()
        .chunk_size(Some(1024 * 1024))
        .compression(Some(McapCompression::Zstd));

    let topics = Arc::new(topics);
    let writer = ctx
        .mcap_writer_with_options(options)
        .channel_filter_fn(move |desc| topics.contains(desc.topic()))
        .create_new_buffered_file(path)?;
    Ok(writer)
}
