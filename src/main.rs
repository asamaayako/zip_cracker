use clap::{Parser, ValueEnum};
use rayon::prelude::*;
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

    /// 字符集选择
    #[arg(short, long, value_enum, default_value = "base64")]
    charset: Charset,

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

#[derive(Clone, ValueEnum)]
enum Charset {
    /// Base64 字符 (A-Za-z0-9, 62字符)
    Base64,
    /// 拼音声母 (20字符)
    Pinyin,
    /// 小写字母 (a-z, 26字符)
    Lower,
    /// 大写字母 (A-Z, 26字符)
    Upper,
    /// 数字 (0-9, 10字符)
    Digit,
    /// 小写+数字 (a-z0-9, 36字符)
    Alnum,
    /// 全部可打印 ASCII (95字符)
    Ascii,
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

    let (charset_name, chars) = get_charset(&args.charset);

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
    let mut total_tested: usize = 0;
    let mut result_password: Option<String> = None;

    // 从最小长度到最大长度逐一尝试
    for current_len in min_len..=max_len {
        if found.load(Ordering::Relaxed) {
            break;
        }

        if min_len != max_len {
            println!("尝试长度 {} ...", current_len);
        }

        let passwords: Vec<String> = generate_all_passwords(&chars, current_len);
        total_tested += passwords.len();

        // 使用 rayon 并行搜索
        let result = passwords.par_iter().find_any(|pwd| {
            if found.load(Ordering::Relaxed) {
                return false;
            }

            if try_password(zip_path, pwd, target_index, &target_ext) {
                found.store(true, Ordering::Relaxed);
                return true;
            }
            false
        });

        if let Some(pwd) = result {
            result_password = Some(pwd.clone());
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

/// 获取字符集
fn get_charset(charset: &Charset) -> (&'static str, Vec<char>) {
    match charset {
        Charset::Base64 => (
            "Base64 (A-Za-z0-9)",
            ('A'..='Z').chain('a'..='z').chain('0'..='9').collect(),
        ),
        Charset::Pinyin => (
            "拼音声母",
            vec![
                'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c',
                's', 'r', 'y', 'w',
            ],
        ),
        Charset::Lower => ("小写字母 (a-z)", ('a'..='z').collect()),
        Charset::Upper => ("大写字母 (A-Z)", ('A'..='Z').collect()),
        Charset::Digit => ("数字 (0-9)", ('0'..='9').collect()),
        Charset::Alnum => ("小写+数字 (a-z0-9)", ('a'..='z').chain('0'..='9').collect()),
        Charset::Ascii => (
            "可打印ASCII",
            (' '..='~').collect(), // ASCII 32-126
        ),
    }
}

/// 生成所有指定长度的密码组合
fn generate_all_passwords(chars: &[char], length: usize) -> Vec<String> {
    let n = chars.len();
    let total = n.pow(length as u32);
    let mut passwords = Vec::with_capacity(total);

    // 使用迭代方式生成，支持任意长度
    fn generate_recursive(
        chars: &[char],
        current: &mut String,
        length: usize,
        passwords: &mut Vec<String>,
    ) {
        if current.len() == length {
            passwords.push(current.clone());
            return;
        }
        for &c in chars {
            current.push(c);
            generate_recursive(chars, current, length, passwords);
            current.pop();
        }
    }

    let mut current = String::with_capacity(length);
    generate_recursive(chars, &mut current, length, &mut passwords);
    passwords
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
