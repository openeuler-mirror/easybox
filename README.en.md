# easybox

[简体中文](./README.md) | English

This is a basic command line project. It uses the Rust language to rewrite the basic command lines used in Linux. It is applicable to server scenarios and embedded scenarios. This project implements the basic commands that have not been implemented in Rust.

## Requirements

Rust (`cargo`, `rustc`) >= 1.65.0

## Building

We use Cargo to build the easybox binaries.

We first need to fetch the repository:

```shell
git clone https://gitee.com/openeuler/easybox
cd easybox
```

Then we can build easybox using Cargo with the same process for every other Rust program:

```shell
cargo build --release
```

This command builds easybox into a multicall (BusyBox-type) binary, named 'easybox'.

If you don't want to build every utility into the final binary, you can also specify which ones you want to build manually. For example:

```shell
cargo build --features "base32 sysctl" --no-default-features
```

If you don't want to build the multicall binary and would prefer to build the utilities as individual binaries, that is also possible. Each utility is contained in its own package within the main repository, named "oe_UTILNAME". To build individual utilities, use Cargo to build just the specific packages (using the `--package` [aka `-p`] option). For example:

```shell
cargo build -p oe_base32 -p oe_sysctl
```

## Installation

To install easybox using Cargo:

```shell
cargo install --path . --locked
```

This command will install easybox into Cargo's _bin_ folder (_e.g._ `$HOME/.cargo/bin`). After that, easybox can be used by `$HOME/.cargo/bin/easybox [util] [util options]`.

## Un-installation

To uninstall easybox using Cargo:

```shell
cargo uninstall easybox
```

## Contribution

To contribute to easybox, please see [CONTRIBUTING](CONTRIBUTING.md).

## License

easybox is licensed under the MulanPSL-2.0 License - see the [LICENSE](LICENSE) file for details.
