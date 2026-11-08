## Configuration
## =============

# Parameters
# ----------

# Name of package containing the app to be built.
# Keep in sync with `name` in `Cargo.toml`.
export AXIS_PACKAGE ?= resmon

# The architecture that will be assumed when interacting with the device.
export AXIS_DEVICE_ARCH ?= aarch64

# Reproducible and stable results by default
export SOURCE_DATE_EPOCH ?= 0

# Other
# -----

.DEFAULT_GOAL := help
.DELETE_ON_ERROR: ;
.SECONDARY:
.SUFFIXES: ;

## Verbs
## =====

help:
	@mkhelp $(firstword $(MAKEFILE_LIST))
.PHONY: help

## Build <AXIS_PACKAGE> for aarch64 and armv7hf
##
## The EAPs are placed in ${CARGO_TARGET_DIR}/acap/, which is target-container/acap/ when built
## with .devcontainer/make-in-container.sh.
build:
	cargo-acap-build \
		-- \
		--locked \
		--package $(AXIS_PACKAGE)
.PHONY: build

## Install <AXIS_PACKAGE> on <AXIS_DEVICE_IP> using password <AXIS_DEVICE_PASS> and assuming architecture <AXIS_DEVICE_ARCH>
install:
	cargo-acap-sdk install \
		-- \
		--locked \
		--package $(AXIS_PACKAGE)
.PHONY: install

## Remove <AXIS_PACKAGE> from <AXIS_DEVICE_IP> using password <AXIS_DEVICE_PASS>
remove:
	cargo-acap-sdk remove
.PHONY: remove

## Start <AXIS_PACKAGE> on <AXIS_DEVICE_IP> using password <AXIS_DEVICE_PASS>
start:
	cargo-acap-sdk start
.PHONY: start

## Stop <AXIS_PACKAGE> on <AXIS_DEVICE_IP> using password <AXIS_DEVICE_PASS>
stop:
	cargo-acap-sdk stop
.PHONY: stop

## Restart <AXIS_PACKAGE> on <AXIS_DEVICE_IP> using password <AXIS_DEVICE_PASS>
restart:
	cargo-acap-sdk restart
.PHONY: restart
