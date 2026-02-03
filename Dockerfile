FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    jq \
    ripgrep \
    fd-find \
    build-essential \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && corepack enable \
    && npm install -g @anthropic-ai/claude-code \
    && rm -rf /var/lib/apt/lists/*

RUN python3 -m pip install --no-cache-dir --upgrade pip \
    && python3 -m pip install --no-cache-dir pipenv

ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:$PATH

RUN mkdir -p "${CARGO_HOME}" "${RUSTUP_HOME}" \
    && curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal \
    && rustup default stable \
    && chmod -R a+w "${CARGO_HOME}" "${RUSTUP_HOME}"

RUN ln -s /usr/bin/fdfind /usr/local/bin/fd

RUN groupadd --gid 1000 sandbox \
    && useradd --uid 1000 --gid 1000 --create-home --home-dir /home/sandbox sandbox \
    && mkdir -p /home/sandbox/.config/claude \
    && chown -R sandbox:sandbox /home/sandbox

WORKDIR /workspace
USER sandbox
