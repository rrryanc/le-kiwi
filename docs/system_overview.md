# LeKiwi Control Stack Overview

This repository defines a lightweight, custom control stack for a LeKiwi robot
running on Raspberry Pi OS with two USB webcams. The design prioritizes:

- Deterministic motion control and safety on-device (Pi)
- Clear message boundaries for logging and replay
- Foxglove SDK for live telemetry + manual MCAP logging
- Laptop-based model inference now, with a clean path to run on Pi later

The initial implementation is a Rust-based stack (`lekiwi` binary) with
subcommands for each service.

## Hardware Assumptions

- Battery: 12V 5200mAh Li-ion
- Motor controller: Wonrabai Serial Bus Servo Driver Board
- Drive servos: Feetech STS3215
- Compute: Raspberry Pi 5, Raspberry Pi OS
- Cameras: Two USB webcams (FMV 1080p)

## Runtime Split

Pi (always-on, safety critical):
- motor_bus
- kinematics
- state_estimator
- cameras
- behavior_router
- foxglove_server (SDK)
- mcap_logger (manual start/stop)

Laptop (higher compute, optional):
- LLM for natural language parsing
- Foundation model for perception + policy
- Sends skill requests to the Pi

## High-Level Data Flow

User (natural language)
  -> Laptop: LLM + foundation model
  -> Pi: behavior_router (safety gate)
  -> kinematics -> motor_bus -> servos

Cameras + state + telemetry
  -> foxglove_server -> live UI
  -> mcap_logger -> MCAP files

## Core Services

### motor_bus
- Connects to the Wonrabai servo bus via USB serial
- Sends wheel speed targets at 50-100 Hz
- Reads back servo telemetry (temp, voltage, load, errors)

### kinematics
- Converts (vx, vy, omega) into three wheel speeds (kiwi drive)
- Applies limits and ramping

### state_estimator
- Tracks pose using commanded velocities (open-loop initially)
- Can incorporate servo feedback later for odometry

### cameras
- Streams base and wrist webcams
- Publishes compressed frames with timestamps

### behavior_router
- Accepts commands from laptop and Foxglove
- Arbitrates priorities and enforces safety
- Emits low-level velocity commands

### foxglove_server
- Runs the Foxglove SDK WebSocket server
- Publishes topics for state, cameras, and diagnostics
- Accepts teleop commands from Foxglove UI

### mcap_logger
- Subscribes to selected topics
- Manual start/stop only (UI or CLI)
- Writes logs to data/mcap

## Safety Model

- E-stop overrides all motion
- Motor command timeout (stop if silent for N ms)
- Speed and acceleration limits
- Servo temperature and error checks
- Low-battery cutoff

## Model Integration (Now and Later)

Now:
- Run LLM + foundation model on a laptop
- Send high-level skill requests to the Pi

Later:
- Move the model process onto the Pi without changing the message interface

## Related Docs

- docs/topics_and_schema.md
- docs/bringup.md
- docs/systemd_plan.md
