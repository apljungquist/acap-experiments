ARG REPO=axisecp
ARG SDK=acap-native-sdk
ARG UBUNTU_VERSION=24.04
ARG VERSION=12.11.0
ARG BASE_IMAGE=debian:trixie-20260223

FROM ${REPO}/${SDK}:${VERSION}-aarch64-ubuntu${UBUNTU_VERSION} AS sdk-aarch64
FROM ${REPO}/${SDK}:${VERSION}-armv7hf-ubuntu${UBUNTU_VERSION} AS sdk-armv7hf
FROM ${BASE_IMAGE}

COPY --from=sdk-aarch64 /opt/axis/acapsdk/axis-acap-manifest-tools /opt/axis/acapsdk/axis-acap-manifest-tools
COPY --from=sdk-aarch64 /opt/axis/acapsdk/environment-setup-cortexa53-crypto-poky-linux /opt/axis/acapsdk/environment-setup-cortexa53-crypto-poky-linux
COPY --from=sdk-armv7hf /opt/axis/acapsdk/environment-setup-cortexa9hf-neon-poky-linux-gnueabi /opt/axis/acapsdk/environment-setup-cortexa9hf-neon-poky-linux-gnueabi
COPY --from=sdk-aarch64 /opt/axis/acapsdk/sysroots/aarch64 /opt/axis/acapsdk/sysroots/aarch64
COPY --from=sdk-armv7hf /opt/axis/acapsdk/sysroots/armv7hf /opt/axis/acapsdk/sysroots/armv7hf
COPY --from=sdk-aarch64 /opt/axis/acapsdk/sysroots/x86_64-pokysdk-linux /opt/axis/acapsdk/sysroots/x86_64-pokysdk-linux

# The linker and sysroot for each target come from `.cargo/config.toml`, not from `CARGO_TARGET_*`
# variables here, which would take precedence over it and drop the `-L` it adds.

COPY .devhost/install-system-packages.sh ./
RUN ./install-system-packages.sh \
 && rm install-system-packages.sh

ENV PATH=/usr/local/cargo/bin:$PATH \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup

# The toolchain is installed from `rust-toolchain.toml`, which `rustup show` picks up from the
# working directory.
#
# The build runs as the invoking user, so `CARGO_HOME` and `RUSTUP_HOME` must be writable by anyone.
#
# A named volume inherits the mode of the image directory it is mounted over; for a path missing
# from the image, Docker instead creates the mount point as root with mode 0755 and the build cannot
# write it. So the caches downloaded during the build are dropped but their directories are kept,
# and `/target`, which make-in-container mounts a volume over as `CARGO_TARGET_DIR`, is created here
# rather than chmodded from outside on every run.
RUN --mount=type=bind,target=/context \
    cd /context/.devhost \
 && ./install-rust.sh \
 && (cd /context && rustup show) \
 && ./install-tools.sh /usr/local \
 && rm -rf $CARGO_HOME/registry $CARGO_HOME/git \
 && mkdir $CARGO_HOME/registry $CARGO_HOME/git \
 && mkdir --mode=0777 /target \
 && find $CARGO_HOME $RUSTUP_HOME -type d -exec chmod a+rwx {} + \
 && find $CARGO_HOME $RUSTUP_HOME -type f -exec chmod a+rw {} +
