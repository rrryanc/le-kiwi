# Topics and Message Schema

This document defines the minimal, stable topics used across the LeKiwi control
stack. Messages are JSON objects with explicit units and timestamps in
nanoseconds.

## Timestamp Conventions

- Use `timestamp_ns` as a uint64 (monotonic or UNIX time; be consistent).
- Every published message must include a timestamp.

## Frames

- `base_link`: robot base frame
- `odom`: local odometry frame
- `map`: optional global frame
- Camera frames: `camera_base`, `camera_wrist`

## Command Topics

### /cmd/velocity

Low-level velocity command (teleop or planner).

Fields:
- timestamp_ns
- vx_m_s
- vy_m_s
- omega_rad_s
- source (string)
- priority (int)

Example:
```
{
  "timestamp_ns": 1738600000000000000,
  "vx_m_s": 0.2,
  "vy_m_s": 0.0,
  "omega_rad_s": 0.1,
  "source": "foxglove",
  "priority": 50
}
```

### /cmd/skill

High-level skill request.

Fields:
- request_id
- skill_name
- params (object)
- timeout_s
- priority (int)

Example:
```
{
  "request_id": "req-00042",
  "skill_name": "rotate_to_heading",
  "params": { "theta_rad": 1.57, "tolerance_rad": 0.1 },
  "timeout_s": 10.0,
  "priority": 30
}
```

### /cmd/estop

Emergency stop toggle.

Fields:
- timestamp_ns
- enabled (bool)
- reason (string)
- source (string)

## State Topics

### /state/odometry

Fields:
- timestamp_ns
- x_m
- y_m
- theta_rad
- vx_m_s
- vy_m_s
- omega_rad_s
- frame_id ("odom")

### /state/servos

Fields:
- timestamp_ns
- servos (array)

Servo object:
- id
- name
- position_rad
- velocity_rad_s
- load
- temperature_c
- voltage_v
- error_flags (bitfield)

### /state/power

Fields:
- timestamp_ns
- battery_voltage_v
- battery_percent
- low_battery (bool)

### /system/diagnostics

Fields:
- timestamp_ns
- status (ok|warn|error)
- warnings (array of strings)
- last_error (string)
- uptime_s

## Sensor Topics

### /sensors/camera/base
### /sensors/camera/wrist

Fields:
- timestamp_ns
- frame_id
- width
- height
- encoding (e.g. "jpeg")
- data (base64 or raw bytes, depending on SDK)

Note: use compressed JPEG to reduce bandwidth on the Pi.

## Logging Topics

### /log/control

Fields:
- timestamp_ns
- action (start|stop)
- topics (array of strings)
- session_name (string)

### /log/status

Fields:
- timestamp_ns
- active (bool)
- file_path
- size_bytes
- duration_s

## Priority Rules

Recommended priority order:

1. E-stop
2. Manual teleop (/cmd/velocity from Foxglove)
3. Model-driven commands (/cmd/skill)
4. Background scripts or tests
