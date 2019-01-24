# Yota
[![Build Status](https://travis-ci.org/artemiy312/yota.svg?branch=master)](https://travis-ci.org/artemiy312/yota)
[![Build status](https://ci.appveyor.com/api/projects/status/8aoo0p2aj0s7jl0m/branch/master?svg=true)](https://ci.appveyor.com/project/artemiy312/yota/branch/master)

Changes a device's speed of Yota provider.
Supports accounts with multiple devices. Runs on Windows, macOS, Linux.

```bash
$ yota --help
yota 0.1.2
Artemiy Rodionov <wertins71@gmail.com>
Changes a device's speed of Yota provider.

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

## Getting Start

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
$ yota -c config.json 7.1
```

### Get the binary

See the [Releases](https://github.com/artemiy312/yota/releases/latest) section.

### Or install with cargo

```bash
$ cargo install --git https://github.com/artemiy312/yota
```

### Requirements

On Linux:

    OpenSSL 1.0.1, 1.0.2, or 1.1.0 with headers

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
