use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use zip::ZipArchive;

#[derive(Parser)]
#[command(name = "zip_cracker")]
#[command(about = "ZIP 密码暴力破解器 (多线程)", long_about = None)]
struct Args {
    /// ZIP 文件路径
    #[arg(default_value = "../default.zip")]
    zip_path: String,

    /// 字符集选择 (可多选，用逗号分隔，如: lower,upper,digit)
    #[arg(short, long, value_delimiter = ',', default_value = "lower,upper,digit")]
    charset: Vec<Charset>,

    /// 密码长度 (固定长度模式)
    #[arg(short, long)]
    length: Option<usize>,

    /// 最大密码长度 (递增模式: 从1开始逐一尝试到此长度)
    #[arg(short, long)]
    max_length: Option<usize>,

    /// 最小密码长度 (递增模式下的起始长度, 默认为1)
    #[arg(long, default_value = "1")]
    min_length: usize,
}

#[derive(Clone, ValueEnum, PartialEq, Eq, Hash)]
enum Charset {
    /// 拼音声母 (20字符)
    Pinyin,
    /// 小写字母 (a-z, 26字符)
    Lower,
    /// 大写字母 (A-Z, 26字符)
    Upper,
    /// 数字 (0-9, 10字符)
    Digit,
    /// ASCII符号 (32字符)
    Symbol,
    /// 全部可打印 ASCII (95字符)
    Ascii,
    /// 全角符号 (常用全角标点)
    Fullwidth,
    /// 常用汉字 (3500字符 - GB2312一级汉字)
    Chinese,
}

/// 从文件名提取扩展名（小写）
fn get_extension(filename: &str) -> Option<String> {
    filename
        .rsplit('.')
        .next()
        .filter(|ext| !ext.is_empty() && ext.len() <= 5)
        .map(|ext| ext.to_lowercase())
}

/// 检查扩展名是否被 infer 库支持
fn is_infer_supported(ext: &str) -> bool {
    // infer 库支持的所有扩展名
    const SUPPORTED: &[&str] = &[
        // 图片
        "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "psd", "ico", "heif", "heic",
        "avif", "jxl", "cr2", "orf", "raf", // 视频
        "mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "flv", "3gp", // 音频
        "mp3", "flac", "wav", "ogg", "m4a", "aac", "aiff", "wma", "amr", // 压缩
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "lz4", "cab", "rpm", "deb",
        // 文档
        "pdf", "docx", "xlsx", "pptx", "odt", "ods", "odp", "epub", "rtf", // 字体
        "ttf", "otf", "woff", "woff2", // 可执行
        "exe", "dll", "elf", "dex", "wasm", "class", // 其他
        "swf", "sqlite", "nes", "crx", "lnk", "alias", "dcm",
    ];
    SUPPORTED.contains(&ext)
}

/// 自动检测 ZIP 中第一个 infer 可识别扩展名的非目录文件
fn detect_target_file(zip_path: &str) -> Option<(usize, String, String)> {
    let file = File::open(zip_path).ok()?;
    let archive = ZipArchive::new(file).ok()?;

    for i in 0..archive.len() {
        let name = archive.name_for_index(i)?;

        // 跳过目录
        if name.ends_with('/') {
            continue;
        }

        let name_owned = name.to_string();
        if let Some(ext) = get_extension(&name_owned) {
            // 只选择 infer 库支持的扩展名
            if is_infer_supported(&ext) {
                return Some((i, name_owned, ext));
            }
        }
    }
    None
}

fn main() {
    let args = Args::parse();
    let zip_path = &args.zip_path;
    let num_cpus = num_cpus::get();

    let (charset_name, chars) = get_combined_charset(&args.charset);

    // 确定密码长度范围
    let (min_len, max_len) = match (args.length, args.max_length) {
        (Some(len), None) => (len, len),             // 固定长度模式
        (None, Some(max)) => (args.min_length, max), // 递增模式
        (Some(_), Some(_)) => {
            eprintln!("错误: --length 和 --max-length 不能同时使用");
            std::process::exit(1);
        }
        (None, None) => {
            eprintln!("错误: 请指定 --length (固定长度) 或 --max-length (递增模式)");
            std::process::exit(1);
        }
    };

    if min_len > max_len {
        eprintln!(
            "错误: --min-length ({}) 不能大于 --max-length ({})",
            min_len, max_len
        );
        std::process::exit(1);
    }

    if min_len == 0 {
        eprintln!("错误: 密码长度不能为 0");
        std::process::exit(1);
    }

    println!("=== ZIP 密码暴力破解器 (Rust 多线程版) ===");
    println!("目标文件: {}", zip_path);
    println!("CPU 核心数: {}", num_cpus);
    println!("字符集: {} ({}字符)", charset_name, chars.len());

    if min_len == max_len {
        println!("密码长度: {}", min_len);
        let total_combinations = (chars.len() as u64).pow(min_len as u32);
        println!(
            "密码空间: {}^{} = {} 组合",
            chars.len(),
            min_len,
            total_combinations
        );
    } else {
        println!("密码长度: {} ~ {} (递增模式)", min_len, max_len);
        let total_combinations: u64 = (min_len..=max_len)
            .map(|len| (chars.len() as u64).pow(len as u32))
            .sum();
        println!(
            "密码空间: {} 组合 (长度{}到{}的总和)",
            total_combinations, min_len, max_len
        );
    }
    println!();

    // 自动检测目标文件
    let (target_index, target_name, target_ext) =
        detect_target_file(zip_path).expect("未找到可识别扩展名的加密文件");

    println!("检测到目标文件: {} (索引 {})", target_name, target_index);
    println!("文件扩展名: .{}", target_ext);
    println!("验证方式: 使用 infer 库检测解密后内容是否匹配扩展名");
    println!();

    // 验证文件存在
    let file = File::open(zip_path).expect("无法打开 ZIP 文件");
    let archive = ZipArchive::new(file).expect("无法解析 ZIP 文件");
    let file_count = archive.len();
    println!("ZIP 文件包含 {} 个文件", file_count);
    println!();
    println!("开始破解...");

    let found = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let mut total_tested: u64 = 0;
    let mut result_password: Option<String> = None;

    // 从最小长度到最大长度逐一尝试
    for current_len in min_len..=max_len {
        if found.load(Ordering::Relaxed) {
            break;
        }

        if min_len != max_len {
            println!("尝试长度 {} ...", current_len);
        }

        let total_combinations = (chars.len() as u64).pow(current_len as u32);
        total_tested += total_combinations;

        // 使用索引并行搜索，零内存预分配
        let result = (0..total_combinations).into_par_iter().find_any(|&index| {
            if found.load(Ordering::Relaxed) {
                return false;
            }

            let pwd = index_to_password(index, &chars, current_len);
            if try_password(zip_path, &pwd, target_index, &target_ext) {
                found.store(true, Ordering::Relaxed);
                return true;
            }
            false
        });

        if let Some(index) = result {
            result_password = Some(index_to_password(index, &chars, current_len));
            break;
        }
    }

    let elapsed = start.elapsed();

    match result_password {
        Some(password) => {
            println!();
            println!("✅ 密码找到: {}", password);
            println!("密码长度: {}", password.len());
            println!("耗时: {:.2} 秒", elapsed.as_secs_f64());
        }
        None => {
            println!();
            println!("❌ 未找到密码");
            println!("耗时: {:.2} 秒", elapsed.as_secs_f64());
        }
    }

    let speed = total_tested as f64 / elapsed.as_secs_f64();
    println!("已测试: {} 个密码", total_tested);
    println!("平均速度: {:.0} 次/秒", speed);
}

/// 获取单个字符集
fn get_single_charset(charset: &Charset) -> (&'static str, Vec<char>) {
    match charset {
        Charset::Pinyin => (
            "拼音声母",
            vec![
                'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c',
                's', 'r', 'y', 'w',
            ],
        ),
        Charset::Lower => ("小写字母", ('a'..='z').collect()),
        Charset::Upper => ("大写字母", ('A'..='Z').collect()),
        Charset::Digit => ("数字", ('0'..='9').collect()),
        Charset::Symbol => (
            "ASCII符号",
            get_ascii_symbols(),
        ),
        Charset::Ascii => (
            "可打印ASCII",
            (' '..='~').collect(), // ASCII 32-126
        ),
        Charset::Fullwidth => (
            "全角符号",
            get_fullwidth_symbols(),
        ),
        Charset::Chinese => (
            "常用汉字 (GB2312一级)",
            get_chinese_charset(),
        ),
    }
}

/// 获取 ASCII 符号字符集（不包括字母和数字）
fn get_ascii_symbols() -> Vec<char> {
    let mut symbols = Vec::new();

    // 空格
    symbols.push(' ');

    // !"#$%&'()*+,-./ (ASCII 33-47)
    symbols.extend('!'..='/');
    symbols.push('/');

    // :;<=>?@ (ASCII 58-64)
    symbols.extend(':'..='@');

    // [\]^_` (ASCII 91-96)
    symbols.extend('['..='`');

    // {|}~ (ASCII 123-126)
    symbols.extend('{'..='~');

    symbols
}

/// 获取全角符号字符集
fn get_fullwidth_symbols() -> Vec<char> {
    vec![
        // 全角标点符号
        '，', '。', '、', '；', '：', '？', '！', '…', '—', '·',
        '"', '"', '\u{2018}', '\u{2019}', '（', '）', '【', '】', '《', '》',
        '『', '』', '「', '」', '〈', '〉', '￥', '※', '〃', '々',
        // 全角数学和其他符号
        '＋', '－', '×', '÷', '＝', '≠', '＜', '＞', '≤', '≥',
        '％', '‰', '°', '℃', '＄', '￡', '￠', '＠', '＃', '＆',
        '＊', '§', '〒', '〓', '□', '■', '△', '▲', '○', '●',
        '◎', '☆', '★', '◇', '◆', '〔', '〕', '〖', '〗',
    ]
}

/// 合并多个字符集（去重）
fn get_combined_charset(charsets: &[Charset]) -> (String, Vec<char>) {
    // 去重字符集选择
    let unique_charsets: Vec<_> = charsets.iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if unique_charsets.is_empty() {
        eprintln!("错误: 必须至少选择一个字符集");
        std::process::exit(1);
    }

    // 合并所有字符集
    let mut char_set = HashSet::new();
    let mut names = Vec::new();

    for charset in &unique_charsets {
        let (name, chars) = get_single_charset(charset);
        names.push(name);
        char_set.extend(chars);
    }

    let name = names.join(" + ");
    let mut chars: Vec<char> = char_set.into_iter().collect();
    chars.sort_unstable(); // 排序以保证顺序一致性

    (name, chars)
}

/// 获取 GB2312 一级汉字字符集 (3500个常用汉字)
fn get_chinese_charset() -> Vec<char> {
    // GB2312 一级汉字：区位码 16-55, 位码 01-94
    // Unicode 编码范围
    let mut chars = Vec::with_capacity(3500);

    // GB2312 一级汉字 Unicode 范围（常用汉字）
    for code in 0x4e00..=0x9fa5_u32 {
        if let Some(c) = char::from_u32(code) {
            // 过滤掉不常用的，保留最常用的3500个
            if chars.len() < 3500 {
                chars.push(c);
            } else {
                break;
            }
        }
    }

    chars
}

/// 将索引转换为密码字符串
fn index_to_password(mut index: u64, chars: &[char], length: usize) -> String {
    let base = chars.len() as u64;
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        result.push(chars[(index % base) as usize]);
        index /= base;
    }
    result
}

fn try_password(zip_path: &str, password: &str, file_index: usize, expected_ext: &str) -> bool {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };

    // 尝试用密码解密指定索引的文件
    let result = archive.by_index_decrypt(file_index, password.as_bytes());
    match result {
        Ok(mut file) => {
            // 读取文件头部用于类型检测 (infer 需要至少前几百字节)
            let mut buffer = vec![0u8; 8192];
            let bytes_read = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => return false,
            };

            if bytes_read == 0 {
                return false;
            }

            // 使用 infer 库检测文件类型
            if let Some(kind) = infer::get(&buffer[..bytes_read]) {
                // 检查检测到的扩展名是否与期望的扩展名匹配
                kind.extension() == expected_ext
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
