#!/usr/bin/env sh
# Run a Makefile target inside an ephemeral build container.
#
# The image is built from the dev container `Dockerfile`, which provides the ACAP SDK, the Rust
# toolchain, and the cross-compilation dependencies needed to build for AXIS devices, so the only
# thing the host needs is Docker.
#
# Usage:
#
#   .devcontainer/make-in-container.sh                  # make build
#   .devcontainer/make-in-container.sh <target> [...]   # make <target> ...
#
# The EAP files from `build` end up in `target-container/acap/` on the host.
#
# Targets that reach a device over VAPIX, such as `install`, `start` and `stop`, work too; the
# `AXIS_DEVICE_*` variables are forwarded from the invoking environment.
# Targets that reach it over SSH do not: the image has no SSH client, and the container has no known
# hosts and no terminal to accept one from.
set -eux

REPO_ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"

# A repository-specific tag, so it is not clobbered by an unrelated image of the same name.
TAG='acap-experiments-resmon'

# Mount the source at a fixed path so cached build fingerprints, which record absolute source
# paths, stay valid across runs.
WORKSPACE="/workspaces/$(basename -- "${REPO_ROOT}")"

if [ "$#" -eq 0 ]; then
  set -- build
fi

docker build \
  --tag "${TAG}" \
  --file "${REPO_ROOT}/.devcontainer/Dockerfile" \
  "${REPO_ROOT}"

# Named volumes keep cargo's caches and target directory on a native filesystem across runs, each of
# which removes its container. This persists downloaded dependencies and, crucially, keeps the build
# fingerprints on a filesystem with stable mtimes; the bind-mounted host filesystem does not
# preserve mtimes across runs on every host, so cargo would otherwise treat everything as stale and
# recompile it.

# The volumes mounted below inherit write permission from the empty, world-writable directories the
# image keeps at those paths.

# The build writes EAP files to CARGO_TARGET_DIR/acap; bind-mount that directory back onto the host
# so the artifacts are available outside the container. The rest of the target directory stays in
# the volume; this nested bind mount only shadows the output directory.
mkdir -p "${REPO_ROOT}/target-container/acap"

# `--user` keeps files written to the host owned by the invoking user.
#
# The `--env NAME` form takes the value from this environment, and leaves the variable unset in the
# container when it is unset here, so the device variables can be listed unconditionally.
docker run \
  --rm \
  --user "$(id -u):$(id -g)" \
  --volume "${REPO_ROOT}:${WORKSPACE}" \
  --volume acap-experiments-resmon-cargo-registry:/usr/local/cargo/registry \
  --volume acap-experiments-resmon-cargo-git:/usr/local/cargo/git \
  --volume acap-experiments-resmon-target:/target \
  --volume "${REPO_ROOT}/target-container/acap:/target/acap" \
  --workdir "${WORKSPACE}" \
  --env CARGO_TARGET_DIR=/target \
  --env AXIS_DEVICE_ARCH \
  --env AXIS_DEVICE_HTTP_PORT \
  --env AXIS_DEVICE_HTTPS_PORT \
  --env AXIS_DEVICE_IP \
  --env AXIS_DEVICE_PASS \
  --env AXIS_DEVICE_USER \
  --env AXIS_PACKAGE \
  "${TAG}" \
  make "$@"
