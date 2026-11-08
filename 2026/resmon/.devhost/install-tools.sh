#!/usr/bin/env sh
# Install the Rust programs that the `Makefile` uses into `ROOT`/bin.
#
# Usage: install-tools.sh <ROOT>
#
# Requires `cargo`, e.g. from `install-rust.sh`.
set -eux

ROOT="${1}"

# Keep the build out of the source tree and remove it afterwards; it is large and is not needed to
# run the programs.
TARGET_DIR="$(mktemp -d)"

# Pin `acap-rs` to a commit ID rather than a tag to protect against supply chain attacks.
ACAP_RS_URL="https://github.com/AxisCommunications/acap-rs.git"
ACAP_RS_REV="37294a76dc24369b1d971504c41da37fe34394f2"

cargo install --locked --root "${ROOT}" --target-dir "${TARGET_DIR}" mkhelp@0.2.3
cargo install --locked --root "${ROOT}" --target-dir "${TARGET_DIR}" --git "${ACAP_RS_URL}" --rev "${ACAP_RS_REV}" cargo-acap-build
cargo install --locked --root "${ROOT}" --target-dir "${TARGET_DIR}" --git "${ACAP_RS_URL}" --rev "${ACAP_RS_REV}" cargo-acap-sdk

# `cargo-acap-build` shells out to this to generate the LICENSE file that every EAP carries. Pinned
# to the version acap-rs itself uses, because 0.9 rejects the `about.toml` that cargo-acap-build
# embeds: it reads `no-clearly-defined` as a table rather than a boolean.
cargo install --locked --root "${ROOT}" --target-dir "${TARGET_DIR}" cargo-about@0.6.2

rm -r "${TARGET_DIR}"
