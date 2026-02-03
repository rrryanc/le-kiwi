# LeKiwi Bring-Up Guide (Raspberry Pi OS)

This guide walks through a safe first bring-up for the 12V LeKiwi build with
Wonrabai servo bus and two USB webcams. It assumes the hardware is assembled
per the Assembly guide.

## 1) Preflight Hardware Checks

- Battery is fully charged (12V 5200mAh Li-ion).
- 12V rail powers the servo bus (DC barrel plug to motor controller).
- 5V rail powers the Pi via the 12V -> 5V converter (USB-C).
- Common ground between servo bus and Pi.
- Inline fuse and physical power switch are installed.

## 2) USB Connections

- Wonrabai serial bus board connected to the Pi via USB.
- Two USB webcams connected (powered hub recommended).

## 3) Set Device Paths

Use stable device paths, not /dev/ttyUSB0 or /dev/video0.

- Serial bus: /dev/serial/by-id/...
- Cameras: /dev/v4l/by-id/...

Record these paths in:
- configs/robot.yaml
- configs/cameras.yaml

## 4) Servo Bus Sanity Check

Before motion:
- Verify the board is detected.
- Confirm each STS3215 servo responds to ping.
- Confirm servo IDs match the config.

If any servo is missing, do not enable torque.

## 5) Camera Sanity Check

Start at low resolution:
- 640x480 or 720p
- 15-30 FPS
- MJPEG if supported

Verify both cameras stream reliably before increasing resolution.

## 6) Configure Wheel Directions

On first motion test:
- Command a slow forward velocity.
- Confirm all wheels rotate to move forward.
- If one wheel is inverted, flip its direction in config.

Repeat for:
- Strafe left/right
- Rotate in place

## 7) Start Services (Recommended Order)

Build the Rust binary:

```bash
# Use Rust 1.83+ (via rustup)
cargo build --release
```

Run the full stack (single process):

```bash
./target/release/lekiwi stack
```

Or run services individually:

```bash
./target/release/lekiwi motor-bus
./target/release/lekiwi kinematics
./target/release/lekiwi state-estimator
./target/release/lekiwi cameras
./target/release/lekiwi foxglove
./target/release/lekiwi behavior-router
```

Recommended startup order if running individually:

1. motor-bus
2. kinematics
3. state-estimator
4. cameras
5. foxglove
6. behavior-router

Confirm diagnostics show "READY" before enabling torque.

## 8) Foxglove UI Test

- Connect to the Foxglove WebSocket server on the Pi.
- Confirm live topics:
  - /state/odometry
  - /state/servos
  - /sensors/camera/base
  - /sensors/camera/wrist
- Send a low-speed /cmd/velocity to verify control.

## 9) Manual MCAP Logging

Logging is manual start/stop only.

- Start logging from the Foxglove UI or via CLI.
- Ensure logs are written under data/mcap.

## 10) First Field Test Checklist

- E-stop works and overrides all motion.
- Command timeout stops motors if input stops.
- Battery voltage remains above low-batt threshold under load.
- Cameras do not drop frames for more than 1-2 seconds.

## Troubleshooting Notes

- If the Pi reboots under load, verify the 5V converter capacity and wiring.
- If USB cameras disconnect, use a powered hub and lower resolution.
- If motors jitter, check servo bus baud rate and power stability.
