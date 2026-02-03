use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::watch;

use crate::bus::Bus;
use crate::config::{CameraConfig, CamerasConfig};
use crate::messages::CameraFrame;
use crate::telemetry::Telemetry;
use crate::utils::now_nanos;

pub async fn run(
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    cameras: CamerasConfig,
    shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut handles = Vec::new();
    for camera in cameras.cameras {
        let bus = bus.clone();
        let telemetry = telemetry.clone();
        let mut shutdown = shutdown.clone();
        handles.push(tokio::spawn(async move {
            run_camera(camera, bus, telemetry, &mut shutdown).await
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn run_camera(
    camera: CameraConfig,
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<()> {
    let fps = camera.fps.max(1) as f64;
    let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / fps));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let frame = CameraFrame {
                    timestamp_ns: now_nanos(),
                    camera_name: camera.name.clone(),
                    frame_id: format!("camera_{}", camera.name),
                    width: camera.width,
                    height: camera.height,
                    encoding: camera.format.clone(),
                    data_base64: String::new(),
                };
                telemetry.log_camera_frame(&frame);
                let _ = bus.camera.send(frame);
            }
            _ = shutdown.changed() => {
                break;
            }
        }
    }

    Ok(())
}
