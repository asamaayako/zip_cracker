# Archive Cracker

多线程压缩包密码破解工具，使用 Rust 编写。支持 ZIP、7z 格式。

## 特性

- **多格式支持**：支持 ZIP、7z 加密压缩包
- **多线程并行**：使用 rayon 库充分利用多核 CPU
- **智能攻击策略**：先尝试字典攻击，失败后自动进行暴力破解
- **密码记忆**：破解成功的密码自动保存到字典，下次优先尝试
- **内置字典**：首次运行自动创建包含 Top 1000 常用密码的字典
- **零内存预分配**：按需生成密码，内存占用仅 ~3MB
- **智能验证**：使用 infer 库通过文件魔数验证密码正确性
- **灵活字符集**：支持拼音声母、字母、数字、汉字等基础字符集

## 安装

### 下载预编译版本

从 [Releases](https://github.com/asamaayako/zip_cracker/releases) 下载对应平台的压缩包，包含：
- `archive_cracker` - 可执行文件
- `dictionary.txt` - 内置 Top 1000 常用密码字典
- `README.md` - 使用说明

### 从源码编译

```bash
git clone https://github.com/asamaayako/zip_cracker.git
cd zip_cracker
cargo build --release
```

## 使用

### 基本用法（推荐）

只需指定压缩包路径，程序会自动使用内置字典尝试破解：

```bash
./archive_cracker 文件.zip
./archive_cracker 文件.7z
```

### 字典 + 暴力破解组合

如果字典攻击失败，可以指定长度参数进行暴力破解：

```bash
# 先尝试字典，失败后暴力破解 4 位密码
./archive_cracker -l 4 文件.zip

# 先尝试字典，失败后尝试 1-6 位密码
./archive_cracker -m 6 文件.zip

# 先尝试字典，失败后尝试 3-8 位纯数字密码
./archive_cracker -c digit --min-length 3 -m 8 文件.zip
```

### 使用自定义字典

```bash
./archive_cracker -D rockyou.txt 文件.zip
```

### 跳过字典攻击

```bash
./archive_cracker --skip-dictionary -l 4 文件.zip
```

## 参数说明

| 参数 | 说明 |
|------|------|
| `-D, --dictionary <PATH>` | 字典文件路径（默认 `~/.archive_cracker/dictionary.txt`） |
| `-l, --length <N>` | 固定密码长度（暴力破解） |
| `-m, --max-length <N>` | 最大密码长度（递增模式） |
| `--min-length <N>` | 最小密码长度，默认为 1 |
| `-c, --charset <NAME>` | 字符集选择（可多选，用逗号分隔） |
| `--skip-dictionary` | 跳过字典攻击，直接暴力破解 |

## 字典文件格式

字典文件为纯文本，每行一个密码：

```
# 这是注释，会被忽略
123456
password
qwerty
abc123
```

- 空行会被忽略
- `#` 开头的行作为注释
- 自动去重，相同密码只尝试一次

默认字典位置：`~/.archive_cracker/dictionary.txt`

## 字符集选项

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

## 支持的格式

| 格式 | 扩展名 | 加密支持 |
|------|--------|----------|
| ZIP | .zip | ✅ ZipCrypto, AES |
| 7z | .7z | ✅ AES-256 |

## 性能参考

测试环境：8核 CPU，Apple M 系列

| 字符集组合 | 字符数 | 4位密码组合数 | 预计时间 |
|-----------|--------|--------------|---------|
| digit | 10 | 10,000 | <1秒 |
| pinyin | 20 | 160,000 | ~2秒 |
| lower | 26 | 456,976 | ~5秒 |
| lower,digit | 36 | 1,679,616 | ~17秒 |
| lower,upper,digit | 62 | 14,776,336 | ~4分钟 |

## 依赖

- [rayon](https://crates.io/crates/rayon) - 并行计算
- [zip](https://crates.io/crates/zip) - ZIP 文件处理
- [sevenz-rust](https://crates.io/crates/sevenz-rust) - 7z 文件处理
- [infer](https://crates.io/crates/infer) - 文件类型检测
- [clap](https://crates.io/crates/clap) - 命令行参数解析

## License

MIT
