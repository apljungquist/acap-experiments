The only thing the host needs is Docker:

    ./.devcontainer/make-in-container.sh build

The `.eap` files end up in `target-container/acap/`. Other `Makefile` targets, such as `install`,
run the same way; the script's header says which work.

The image has the ACAP Native SDK 12 sysroots and manifest tools in `/opt/axis/acapsdk`, the Debian
cross compilers, and `cargo-acap-build` from acap-rs. Outside it, `cargo-acap-build` needs the same.

`.cargo/config.toml` puts the sysroot's lib directory ahead with `-L`, because libaxparameter's glib
needs the sysroot's glibc and the cross compiler searches its own first. The image sets no
`CARGO_TARGET_*` variables, since they would override that file.
