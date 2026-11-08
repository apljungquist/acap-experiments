#!/bin/sh
# Print the disk usage on a device before and after installing an ACAP.
#
# Usage: measure-install.sh <app.eap>
#
# The device comes from the `AXIS_DEVICE_*` variables, as from `device-inventory for-each`. Apps
# land in `/usr/local/packages`, so the line to watch is the one mounted on `/usr/local`.
#
# Only the happy path is handled: the device must be reachable, accept the app, and allow SSH as
# `AXIS_DEVICE_USER`, which AXIS OS 11 and later do not for `root` until SSH is enabled on it.
set -eu

df_on_device() {
    # The host key is accepted without being checked and is never saved, so neither an unknown
    # device nor a reflashed one stops the script, and `known_hosts` is left alone. The log level
    # keeps ssh from reporting the key it just accepted.
    sshpass -eAXIS_DEVICE_PASS \
        ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR \
        "${AXIS_DEVICE_USER}@${AXIS_DEVICE_IP}" df /usr/local
}

# Uninstall first, so the first measurement is of a device without the app. The remove API wants
# the app's name, which is in the manifest inside the EAP rather than in its file name.
curl --silent --insecure --anyauth \
    --user "${AXIS_DEVICE_USER}:${AXIS_DEVICE_PASS}" \
    "https://${AXIS_DEVICE_IP}/axis-cgi/applications/control.cgi?action=remove&package=$(
        tar xzOf "${1}" manifest.json | sed -n 's/.*"appName": *"\([^"]*\)".*/\1/p')"

echo 'Before:'
df_on_device

curl --silent --insecure --anyauth \
    --user "${AXIS_DEVICE_USER}:${AXIS_DEVICE_PASS}" \
    --form "packfil=@${1}" \
    "https://${AXIS_DEVICE_IP}/axis-cgi/applications/upload.cgi"

echo 'After:'
df_on_device
