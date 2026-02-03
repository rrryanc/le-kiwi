# Systemd Boot Plan

This document defines a systemd layout for the LeKiwi stack. The goal is a
predictable boot sequence with explicit dependencies and a manual-only logger.

## Service Names (suggested)

- lekiwi-motor-bus.service
- lekiwi-kinematics.service
- lekiwi-state-estimator.service
- lekiwi-cameras.service
- lekiwi-foxglove.service
- lekiwi-behavior-router.service
- lekiwi-mcap-logger.service (manual start/stop)
- lekiwi-stack.target (aggregate)

## Boot Order

1. lekiwi-motor-bus
2. lekiwi-kinematics
3. lekiwi-state-estimator
4. lekiwi-cameras
5. lekiwi-foxglove
6. lekiwi-behavior-router

The behavior_router should not enable torque until motor_bus reports healthy.

## Dependencies (conceptual)

Example dependencies to encode in unit files:

- motor_bus:
  - After=network-online.target
  - Wants=network-online.target

- kinematics:
  - After=lekiwi-motor-bus.service
  - Wants=lekiwi-motor-bus.service

- state_estimator:
  - After=lekiwi-kinematics.service
  - Wants=lekiwi-kinematics.service

- cameras:
  - After=multi-user.target

- foxglove:
  - After=lekiwi-cameras.service lekiwi-state-estimator.service
  - Wants=lekiwi-cameras.service lekiwi-state-estimator.service

- behavior_router:
  - After=lekiwi-foxglove.service lekiwi-motor-bus.service
  - Wants=lekiwi-foxglove.service lekiwi-motor-bus.service

## Manual MCAP Logger

The logger should not auto-start at boot. Keep it disabled by default.

Options:

1) Start/stop via Foxglove UI:
   - The logger process is always running but idle.
   - /log/control toggles recording.

2) Start/stop via systemd:
   - lekiwi-mcap-logger.service is disabled
   - start it only when needed

## Suggested Unit Settings

- Restart=on-failure
- RestartSec=1
- StandardOutput=journal
- StandardError=journal
- EnvironmentFile=/etc/lekiwi/lekiwi.env

## Aggregate Target

Create lekiwi-stack.target and enable it at boot.

Concept:
- lekiwi-stack.target Wants all core services
- A single systemctl command starts the stack:
  systemctl start lekiwi-stack.target

## Suggested Enablement

Enable:
- lekiwi-motor-bus.service
- lekiwi-kinematics.service
- lekiwi-state-estimator.service
- lekiwi-cameras.service
- lekiwi-foxglove.service
- lekiwi-behavior-router.service
- lekiwi-stack.target

Disable:
- lekiwi-mcap-logger.service (manual only)
