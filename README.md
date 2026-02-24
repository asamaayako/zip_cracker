# ZIP Cracker

多线程 ZIP 密码暴力破解工具，使用 Rust 编写。

## 特性

- **多线程并行**：使用 rayon 库充分利用多核 CPU
- **零内存预分配**：按需生成密码，内存占用仅 ~3MB（支持超大字符集和密码空间）
- **自动检测**：自动识别 ZIP 内可验证的加密文件
- **智能验证**：使用 infer 库通过文件魔数验证密码正确性
- **灵活字符集**：支持拼音声母、字母、数字、汉字等基础字符集，可自由组合
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
./target/release/zip_cracker -c lower,digit -l 5 文件.zip
```

### 递增模式（未知密码位数）

当不确定密码长度时，使用 `-m` 指定最大长度，程序会从最小长度开始逐一尝试：

```bash
# 从1位尝试到6位
./target/release/zip_cracker -m 6 文件.zip

# 从3位尝试到8位
./target/release/zip_cracker --min-length 3 -m 8 文件.zip

# 配合字符集使用（仅数字）
./target/release/zip_cracker -c digit -m 6 文件.zip

# 组合多个字符集（大小写字母+数字）
./target/release/zip_cracker -c lower,upper,digit -m 6 文件.zip
```

### 参数说明

| 参数 | 说明 |
|------|------|
| `-l, --length <N>` | 固定密码长度 |
| `-m, --max-length <N>` | 最大密码长度（递增模式） |
| `--min-length <N>` | 最小密码长度，默认为 1 |
| `-c, --charset <NAME>` | 字符集选择（可多选，用逗号分隔） |

## 字符集选项

### 基础字符集

| 参数 | 名称 | 字符数 | 字符范围 |
|------|------|--------|----------|
| `pinyin` | 拼音声母 | 20 | bpmfdtnlgkhjqxzcsryw |
| `lower` | 小写字母 | 26 | a-z |
| `upper` | 大写字母 | 26 | A-Z |
| `digit` | 数字 | 10 | 0-9 |
| `symbol` | ASCII符号 | 32 | !"#$%&'()*+,-./:;<=>?@[\]^_`{\|}~ 和空格 |
| `ascii` | 全部可打印ASCII | 95 | 字母+数字+符号（空格到~） |
| `fullwidth` | 全角符号 | 60+ | 全角标点、数学符号等（，。？！等） |
| `chinese` | 常用汉字 | 3500 | GB2312一级汉字 |

默认字符集：`lower,upper,digit`（大小写字母+数字，62字符）

### 组合示例

```bash
# 仅小写字母
./target/release/zip_cracker -c lower -l 4 文件.zip

# 小写+数字
./target/release/zip_cracker -c lower,digit -l 4 文件.zip

# 大小写字母+数字（默认）
./target/release/zip_cracker -c lower,upper,digit -l 4 文件.zip

# 仅数字+符号（如：123!）
./target/release/zip_cracker -c digit,symbol -l 4 文件.zip

# 拼音声母+数字
./target/release/zip_cracker -c pinyin,digit -l 4 文件.zip

# 数字+全角符号
./target/release/zip_cracker -c digit,fullwidth -l 4 文件.zip

# 字母+数字+ASCII符号（全部ASCII）
./target/release/zip_cracker -c ascii -l 4 文件.zip
```

## 性能参考

测试环境：8核 CPU，Apple M 系列

### 速度基准

| 字符集组合 | 字符数 | 4位密码组合数 | 预计时间 | 平均速度 |
|-----------|--------|--------------|---------|----------|
| digit | 10 | 10,000 | <1秒 | ~30,000/秒 |
| pinyin | 20 | 160,000 | ~2秒 | ~60,000/秒 |
| lower | 26 | 456,976 | ~5秒 | ~70,000/秒 |
| lower,digit | 36 | 1,679,616 | ~17秒 | ~70,000/秒 |
| lower,upper,digit | 62 | 14,776,336 | ~4分钟 | ~64,000/秒 |
| ascii | 95 | 81,450,625 | ~14分钟 | ~90,000/秒 |

## 依赖

- [rayon](https://crates.io/crates/rayon) - 并行计算
- [zip](https://crates.io/crates/zip) - ZIP 文件处理
- [infer](https://crates.io/crates/infer) - 文件类型检测
- [clap](https://crates.io/crates/clap) - 命令行参数解析

## License

MIT
