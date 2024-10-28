# easybox

简体中文 | [English](./README.en.md)

easyBox 是一个基础命令行的项目，该项目使用 Rust 语言重写 Linux 下的基础命令，支持服务器场景以及嵌入式场景。该项目优先会支持当前还未进行 Rust 重构的基础软件，并借助 Rust 的安全能力，提供更为安全的操作系统基础命令。当前项目处于启动阶段，欢迎社区开发者参与 ISSUE 讨论以及命令的开发。

## 环境要求

Rust (`cargo`, `rustc`) >= 1.65.0

## 构建方法

我们使用 Cargo 来构建 easybox 二进制文件。

我们首先需要拉取仓库：

```shell
git clone https://gitee.com/openeuler/easybox
cd easybox
```

然后我们可以使用 Cargo 构建 easybox，该流程与其他 Rust 程序相同：

```shell
cargo build --release
```

此命令将 easybox 构建为名为 “easybox” 的多调用（BusyBox-type）二进制文件。

如果你不想将每个工具都构建到最终二进制文件中，你可以手动指定需要构建的工具。例如：

```shell
cargo build --features "base32 sysctl" --no-default-features
```

如果您不想构建多调用二进制文件，也可以将每个工具构建为单独的二进制文件。每个工具都包含在主仓库中的自己的包中，名为 “oe_UTILNAME”。要构建单独的二进制文件，可使用 Cargo 仅构建特定包（使用 `--package` [又名 `-p`] 选项）。例如：

```shell
cargo build -p oe_base32 -p oe_sysctl
```

## 安装方法

使用 Cargo 安装 easybox：

```shell
cargo install --path . --locked
```

此命令将 easybox 安装到 Cargo 的 _bin_ 文件夹中（例如 `$HOME/.cargo/bin`）。之后，可以通过 `$HOME/.cargo/bin/easybox [util] [util options]` 使用 easybox。

## 卸载方法

使用 Cargo 卸载 easybox：

```shell
cargo uninstall easybox
```

## 参与贡献

参与 easybox 贡献，请参阅 [CONTRIBUTING](CONTRIBUTING.md) 文件。

## LICENSE

easybox 使用 MulanPSL-2.0 许可证，详细信息请参阅 [LICENSE](LICENSE) 文件。
