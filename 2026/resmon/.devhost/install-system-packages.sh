#!/usr/bin/env sh
# Install the system packages needed to build and install the app.
# `libssl-dev` is for `cargo-acap-sdk`, not for the app.
set -eux

apt-get update

apt-get install \
  --assume-yes \
  --no-install-recommends \
  build-essential \
  ca-certificates \
  curl \
  gcc-aarch64-linux-gnu \
  gcc-arm-linux-gnueabihf \
  libssl-dev \
  pkg-config \
  python3-jsonschema

rm -rf /var/lib/apt/lists/*
