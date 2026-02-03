use std::sync::Arc;

use anyhow::Result;
use tokio::sync::watch;

use crate::bus::Bus;

pub async fn run(_bus: Arc<Bus>, mut shutdown: watch::Receiver<bool>) -> Result<()> {
    tracing::info!("Kinematics service is a placeholder in this build");
    wait_for_shutdown(&mut shutdown).await;
    Ok(())
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
