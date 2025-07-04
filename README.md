# OpenSight OS Network API (Rust implementation)

Network management for the Greenbone OpenSight Operating System (implemented in Rust).

The OpenSight Network API provides a REST API that should be used to configure the network connectivity of OpenSight OS.

The REST API itself is made using `actix-web`, and the documention is generated by `utoipa` (`swagger`/`openapi`).

## Dependencies:
- `Netplan` is used to configure the network interfaces. The `netplan` configuration files are located in `/etc/netplan/`.
- `networkd` is used to manage the network interfaces.

## Installation:
The project itself can be cloned and run with
```
cargo run
```
The project can also be built with
```
cargo build --release
```
and the binary can be run from `target/release/opensight-network-api`.

The server is hosted at an _unspecified_ address on port `8080` (most likely `localhost:8080`), and the documentation itself is hosted at `/docs/`.

Simply run the binary and navigate to
```
localhost:8080/docs/
```
to see the documentation.
