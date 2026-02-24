# ZIP Cracker

多线程 ZIP 密码暴力破解工具，使用 Rust 编写。

## 特性

- **多线程并行**：使用 rayon 库充分利用多核 CPU
- **自动检测**：自动识别 ZIP 内可验证的加密文件
- **智能验证**：使用 infer 库通过文件魔数验证密码正确性
- **多种字符集**：支持 Base64、拼音声母、字母、数字等多种字符集
- **可配置长度**：支持固定长度或递增模式（未知密码位数时自动尝试）

## 安装

```bash
git clone https://github.com/asamaayako/zip_cracker.git
cd zip_cracker
cargo build --release
```

## 使用

### 固定长度模式

当已知密码长度时使用 `-l` 参数：

```bash
# 指定4位密码
./target/release/zip_cracker -l 4 文件.zip

# 指定字符集 + 密码长度
./target/release/zip_cracker -c alnum -l 5 文件.zip
```

### 递增模式（未知密码位数）

当不确定密码长度时，使用 `-m` 指定最大长度，程序会从最小长度开始逐一尝试：

```bash
# 从1位尝试到6位
./target/release/zip_cracker -m 6 文件.zip

# 从3位尝试到8位
./target/release/zip_cracker --min-length 3 -m 8 文件.zip

# 配合字符集使用
./target/release/zip_cracker -c digit -m 6 文件.zip
```

### 参数说明

| 参数 | 说明 |
|------|------|
| `-l, --length <N>` | 固定密码长度 |
| `-m, --max-length <N>` | 最大密码长度（递增模式） |
| `--min-length <N>` | 最小密码长度，默认为 1 |
| `-c, --charset <NAME>` | 字符集选择 |

## 字符集选项

| 参数 | 名称 | 字符数 | 字符范围 |
|------|------|--------|----------|
| `base64` | Base64 (默认) | 62 | A-Za-z0-9 |
| `pinyin` | 拼音声母 | 20 | bpmfdtnlgkhjqxzcsryw |
| `lower` | 小写字母 | 26 | a-z |
| `upper` | 大写字母 | 26 | A-Z |
| `digit` | 数字 | 10 | 0-9 |
| `alnum` | 小写+数字 | 36 | a-z0-9 |
| `ascii` | 可打印ASCII | 95 | 空格到~ |

## 性能参考

4位密码，8核 CPU (~100,000次/秒):

| 字符集 | 组合数 | 预计时间 |
|--------|--------|----------|
| digit | 10,000 | <1秒 |
| pinyin | 160,000 | ~2秒 |
| lower | 456,976 | ~5秒 |
| alnum | 1,679,616 | ~17秒 |
| base64 | 14,776,336 | ~2.5分钟 |
| ascii | 81,450,625 | ~14分钟 |

## 依赖

- [rayon](https://crates.io/crates/rayon) - 并行计算
- [zip](https://crates.io/crates/zip) - ZIP 文件处理
- [infer](https://crates.io/crates/infer) - 文件类型检测
- [clap](https://crates.io/crates/clap) - 命令行参数解析

## License

MIT
