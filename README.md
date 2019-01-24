# Yota
[![Build Status](https://travis-ci.org/artemiy312/yota.svg?branch=master)](https://travis-ci.org/artemiy312/yota)
[![Build status](https://ci.appveyor.com/api/projects/status/8aoo0p2aj0s7jl0m/branch/master?svg=true)](https://ci.appveyor.com/project/artemiy312/yota/branch/master)

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

```bash
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git artemiy312/yota --to /usr/local/bin
```

Or get the [latest version](https://github.com/artemiy312/yota/releases/latest) for your platform manually.

## Running the tests

```bash
cargo test
```

## Generating the docs

```bash
cargo doc --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
