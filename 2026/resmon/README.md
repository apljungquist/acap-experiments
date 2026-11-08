# resmon

*A Rust ACAP app that uses no-std and fun hacks to do something useful while staying tiny*

The app:
- uses `axparameter` to allow users to configure it and
- abuses the web server folowing symlinks when serving static content to serve data.

Installed on a P8815-2 (`armv7hf`) the app uses only 4KiB.

## Building

See [building](docs/building.md) for how to build it.

## Preview

![gui](img/resmon_index.jpg)
