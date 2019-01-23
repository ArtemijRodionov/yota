# Yota

Change a device's speed of Yota provider.
Supports accounts with multiple devices.

```bash
yota 0.1.0
Artemiy Rodionov <wertins71@gmail.com>
Manage an user account of Yota provider

USAGE:
    yota [OPTIONS] <SPEED>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <PATH>    Sets a custom config file path. Default: $HOME/.yota/default.json

ARGS:
    <SPEED>
```

## Getting Started

Create config file `config.json`:
```json
{
  "name": "example@email.com",
  "password": "pass",
  "iccid": "11111111111"
}
```

You may find ICCID of the device at "yota.ru/selfcare/devices".

Change the speed:
```bash
yota 7.1
```

### Installing

```rust
cargo build --release
sudo cp target/release/yota /usr/local/bin
```

## Running the tests

```rust
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
