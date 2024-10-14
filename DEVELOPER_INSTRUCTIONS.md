# 配置本地开发环境

关于贡献规则和最佳实践，请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 开始之前

在本指南中，我们假设你已经有一个 Gitee 账户，并且已经安装并配置了 `git` 和你喜欢的代码编辑器或 IDE。

在开始 easybox 项目之前，请按以下步骤操作：

1. 将 [easybox 仓库](https://gitee.com/openeuler/easybox) fork 到你的 Gitee 账户。
***注：*** 请参阅 [Gitee 指南](https://help.gitee.com/base/pullrequest/Fork+Pull#%E5%A6%82%E4%BD%95-fork-%E4%BB%93%E5%BA%93) 了解此步骤的更多信息。
2. 将 fork 的仓库克隆到本地开发环境：

    ```shell
    git clone https://gitee.com/YOUR-GITEE-ACCOUNT/easybox
    cd easybox
    ```

## 必选工具

你需要本节中提到的工具来在本地构建和测试代码更改。本节将解释如何安装和配置这些工具。该仓库已配置 CI，它将使用这些工具检查你的代码。下一节 [测试](#测试) 将解释如何在本地运行这些检查，以避免等待 CI。

**该节涉及的必选工具的安装与使用已集成至测试脚本，若无特殊需要，你无需手动安装这些工具，仅需根据 [测试](#测试) 直接进行测试即可。**

### Rust 工具链

[安装 Rust](https://www.rust-lang.org/tools/install)

如果你使用 rustup 来安装和管理 Rust 工具链，通常 clippy 和 rustfmt 已被安装。如果你使用其他方法安装 Rust，请确保手动安装它们。

### pre-commit hooks

[安装 pre-commit](https://pre-commit.com/#install)

仓库中提供了 `pre-commit` 的配置文件 [.pre-commit-config.yaml](.pre-commit-config.yaml) 以进行自动测试。

***注：*** 若使用 `pip` 安装后仍提示 `command not found`，可使用 `export PATH="$HOME/.local/bin:$PATH"` 将 `pip` 的二进制文件目录添加到 PATH 变量，`codespell` 同理。

***注：*** 可通过在仓库目录中运行 `pre-commit install` 使 git 提交自动进行检查，如果某个检查失败，会显示错误消息解释原因，提交也会被取消，你可以根据建议进行修改，然后再次运行 `git commit ...`。但由于 cargo-test 中部分测试不适用于非 CI 环境等问题，cargo-test 通常会失败，因此可按照下一节 [测试](#测试) 中的步骤手动分为两部分进行测试。

### Spell checker

[安装 codespell](https://github.com/codespell-project/codespell?tab=readme-ov-file#installation)

我们使用 `codespell` 作为项目中所有文件的拼写检查工具。如果你希望让拼写检查工具忽略某个单词，可以在 [codespell_ignore_words](ci/codespell_ignore_words) 添加想要忽略的单词。

## 非必选工具

### Markdown linter

我们推荐使用 `markdownlint` 来检查仓库中的 Markdown 文件。如果你使用的是 VS Code，可以安装
[markdownlint](https://marketplace.visualstudio.com/items?itemName=DavidAnson.vscode-markdownlint)
扩展，在编辑器中进行语法检查。否则，你可以单独 [安装 markdownlint](https://github.com/DavidAnson/markdownlint)。

## 测试

所有测试已集成至脚本 [00-pre.sh](ci/00-pre.sh)、[01-pre-commit.sh](ci/01-pre-commit.sh) 和 [02-musl-build.sh](ci/02-musl-build.sh)，可在提交修改后进行测试：

```shell
sh +x ci/00-pre.sh
sh +x ci/01-pre-commit.sh
sh +x ci/02-musl-build.sh
```

由于 [前文](#pre-commit-hooks) 中存在的问题，以上过程中未对 cargo-test 进行测试，建议手动使用 `cargo` 对涉及的特定工具进行 cargo-test 测试：

```shell
# 若被测试工具不需要 root 权限，以 base32 为例
RUST_BACKTRACE=full cargo test base32 -- --nocapture --test-threads=1

# 若被测试工具需要 root 权限，以 which 为例
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' RUST_BACKTRACE=full cargo test which -- --nocapture --test-threads=1
# or
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' RUST_BACKTRACE=full cargo test which -- --nocapture --test-threads=1
```

### 00-pre.sh

包含一些依赖和 Rust 的安装。

### 01-pre-commit.sh

安装 `pre-commit` 和 `codespell`，并使用 `pre-commit` 进行测试。

测试包含在 [.pre-commit-config.yaml](.pre-commit-config.yaml) 中，包括：

- [check-byte-order-marker](https://gitee.com/overweight/pre-commit-hooks#fix-byte-order-marker)
- [check-case-conflict](https://gitee.com/overweight/pre-commit-hooks#check-case-conflict)
- [check-merge-conflict](https://gitee.com/overweight/pre-commit-hooks#check-merge-conflict)
- [check-symlinks](https://gitee.com/overweight/pre-commit-hooks#check-symlinks)
- [check-toml](https://gitee.com/overweight/pre-commit-hooks#check-toml)
- [end-of-file-fixer](https://gitee.com/overweight/pre-commit-hooks#end-of-file-fixer)
- [mixed-line-ending](https://gitee.com/overweight/pre-commit-hooks#mixed-line-ending)
- [trailing-whitespace](https://gitee.com/overweight/pre-commit-hooks#trailing-whitespace)
- [detect-private-key](https://gitee.com/overweight/pre-commit-hooks#detect-private-key)
- codespell
- commit-msg
- cargo-fmt
- cargo-fix
- cargo-build
- cargo-test

### 02-musl-build.sh

测试使用 musl 进行编译。
