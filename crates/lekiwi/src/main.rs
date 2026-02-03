mod bus;
mod config;
mod messages;
mod services;
mod telemetry;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::sync::watch;
use tracing_subscriber::EnvFilter;

use crate::bus::Bus;
use crate::config::AppConfig;
use crate::services::{
    behavior_router, cameras, foxglove_server, kinematics, mcap_logger, motor_bus, state_estimator,
};
use crate::telemetry::Telemetry;

#[derive(Debug, Parser)]
#[command(name = "lekiwi", about = "LeKiwi robot stack")]
struct Cli {
    #[arg(long, default_value = "configs/robot.yaml")]
    robot_config: PathBuf,
    #[arg(long, default_value = "configs/cameras.yaml")]
    cameras_config: PathBuf,
    #[arg(long, default_value = "configs/logging.yaml")]
    logging_config: PathBuf,
    #[arg(long, default_value = "0.0.0.0")]
    foxglove_host: String,
    #[arg(long, default_value_t = 8765)]
    foxglove_port: u16,
    #[arg(long, default_value = "LeKiwi")]
    foxglove_name: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Stack,
    Foxglove,
    McapLogger,
    BehaviorRouter,
    Kinematics,
    MotorBus,
    StateEstimator,
    Cameras,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = Arc::new(AppConfig::load(
        &cli.robot_config,
        &cli.cameras_config,
        &cli.logging_config,
    )?);

    let ctx = foxglove::Context::get_default();
    let telemetry = Arc::new(Telemetry::new(&ctx)?);
    let bus = Arc::new(Bus::new());

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    spawn_ctrlc_handler(shutdown_tx.clone());

    let foxglove_cfg = foxglove_server::FoxgloveConfig {
        host: cli.foxglove_host,
        port: cli.foxglove_port,
        name: cli.foxglove_name,
    };

    match cli.command {
        Command::Stack => {
            run_stack(
                config,
                ctx,
                bus,
                telemetry,
                foxglove_cfg,
                shutdown_rx,
            )
            .await?;
        }
        Command::Foxglove => {
            foxglove_server::run(foxglove_cfg, ctx, bus, telemetry, shutdown_rx).await?;
        }
        Command::McapLogger => {
            mcap_logger::run(
                bus,
                telemetry,
                ctx,
                config.logging.logging,
                shutdown_rx,
            )
            .await?;
        }
        Command::BehaviorRouter => {
            behavior_router::run(
                bus,
                telemetry,
                config.robot.limits,
                config.robot.safety,
                shutdown_rx,
            )
            .await?;
        }
        Command::Kinematics => {
            kinematics::run(bus, shutdown_rx).await?;
        }
        Command::MotorBus => {
            motor_bus::run(bus, telemetry, config.robot, shutdown_rx).await?;
        }
        Command::StateEstimator => {
            state_estimator::run(bus, telemetry, config.robot.frames, shutdown_rx).await?;
        }
        Command::Cameras => {
            cameras::run(bus, telemetry, config.cameras, shutdown_rx).await?;
        }
    }

    Ok(())
}

async fn run_stack(
    config: Arc<AppConfig>,
    ctx: Arc<foxglove::Context>,
    bus: Arc<Bus>,
    telemetry: Arc<Telemetry>,
    foxglove_cfg: foxglove_server::FoxgloveConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let mut handles = Vec::new();

    handles.push(tokio::spawn(foxglove_server::run(
        foxglove_cfg,
        ctx.clone(),
        bus.clone(),
        telemetry.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(mcap_logger::run(
        bus.clone(),
        telemetry.clone(),
        ctx.clone(),
        config.logging.logging.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(behavior_router::run(
        bus.clone(),
        telemetry.clone(),
        config.robot.limits.clone(),
        config.robot.safety.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(kinematics::run(
        bus.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(state_estimator::run(
        bus.clone(),
        telemetry.clone(),
        config.robot.frames.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(motor_bus::run(
        bus.clone(),
        telemetry.clone(),
        config.robot.clone(),
        shutdown.clone(),
    )));

    handles.push(tokio::spawn(cameras::run(
        bus.clone(),
        telemetry.clone(),
        config.cameras.clone(),
        shutdown.clone(),
    )));

    wait_for_shutdown(&mut shutdown).await;

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

fn spawn_ctrlc_handler(tx: watch::Sender<bool>) {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = tx.send(true);
        }
    });
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
