# LeKiwi Robot Stack

Lightweight control stack for a LeKiwi robot running on Raspberry Pi OS with
two USB webcams. The stack is designed for:

- Safe, deterministic motion control
- Foxglove SDK live telemetry + manual MCAP logging
- Model-based control via laptop now, with a path to run on Pi later

## Docs

- docs/system_overview.md
- docs/bringup.md
- docs/topics_and_schema.md
- docs/systemd_plan.md

## Config Templates

- configs/robot.yaml
- configs/cameras.yaml
- configs/logging.yaml

## Rust Stack

The control stack is implemented in Rust and uses the Foxglove SDK for live
visualization plus MCAP logging. Install Rust 1.83+ with rustup and build:

```bash
cargo build --release
```

Run the full stack (single process with multiple async services):

```bash
./target/release/lekiwi stack
```

Individual services are available as subcommands:

```bash
./target/release/lekiwi foxglove
./target/release/lekiwi mcap-logger
./target/release/lekiwi behavior-router
./target/release/lekiwi kinematics
./target/release/lekiwi motor-bus
./target/release/lekiwi state-estimator
./target/release/lekiwi cameras
```

## Development Sandbox

Run [Claude Code](https://docs.anthropic.com/en/docs/claude-code) in a sandboxed
Docker container. Your code is accessible, but your SSH keys, git credentials,
and filesystem are not.

## Why?

AI coding tools can execute arbitrary commands. This setup ensures:

- [x] AI can read/write your project files
- [x] AI can run builds, tests, dev servers
- [x] Claude credentials persist across sessions
- [ ] AI cannot access your SSH keys
- [ ] AI cannot access git credentials / push to GitHub
- [ ] AI cannot read files outside the project
- [ ] AI cannot access cloud credentials (AWS, GCP, etc.)

## Files

```
claude-sandbox/
├── README.md         # You are here
├── install.sh        # Symlinks files into place
├── claude-sandbox    # Launcher script
└── Dockerfile        # Container definition
```

## Install

```bash
# Clone your dotfiles, then:
./install.sh

# Build the Docker image (first time only, ~2 min)
claude-sandbox --build
```

If `~/.local/bin` isn't in your PATH, add to `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Usage

```bash
# Start Claude in current directory
cd ~/projects/my-app
claude-sandbox

# Start with an initial prompt
claude-sandbox "explain this codebase"

# Get a shell instead of Claude
claude-sandbox --shell

# Rebuild after Dockerfile changes
claude-sandbox --build
```

## First Run

On first launch, Claude will prompt you to authenticate in your browser.
Credentials are stored in a Docker volume (`claude-auth`) and persist across
sessions.

## What's Installed

| Tool | Version |
|------|---------|
| Node.js | 20.x LTS |
| Yarn | Latest (via corepack) |
| Rust | Stable |
| Python | 3.x |
| pipenv | Latest |
| Git | Latest |
| ripgrep, fd, jq | Latest |

## Customizing

Edit the `Dockerfile` and rebuild:

```bash
# Example: add Go
# Add to Dockerfile:
#   RUN wget -q https://go.dev/dl/go1.22.0.linux-amd64.tar.gz \
#       && tar -C /usr/local -xzf go1.22.0.linux-amd64.tar.gz \
#       && rm go1.22.0.linux-amd64.tar.gz
#   ENV PATH="/usr/local/go/bin:$PATH"

claude-sandbox --build
```

## Resource Limits

The container runs with:
- 8GB memory limit
- 4 CPU cores

Edit `claude-sandbox` script to adjust.

## Troubleshooting

**Reset Claude authentication:**
```bash
docker volume rm claude-auth
```

**Permission denied on project files:**

The container runs as uid 1000. If your Mac user has a different uid, edit the
`docker run` command in `claude-sandbox`:

```bash
--user $(id -u):$(id -g)
```

**Container can't access network:**

Network access is enabled by default. If you disabled it and need it back,
remove `--network none` from the script.

**Rebuild from scratch:**
```bash
docker rmi claude-sandbox
claude-sandbox --build
```

## Security Notes

This setup relies on Docker's container isolation. On macOS, Docker Desktop runs
containers inside a Linux VM, providing an additional layer of separation from
your host.

**Not mounted (safe):**
- `~/.ssh`
- `~/.git-credentials`
- `~/.aws`, `~/.kube`, `~/.config/gcloud`
- Docker socket

**Mounted (accessible to AI):**
- Current working directory only
- Claude auth (separate volume)

For stronger isolation (e.g., if your project needs Docker-in-Docker), consider
running inside a Lima VM.

## License

MIT - Do whatever you want with it.
