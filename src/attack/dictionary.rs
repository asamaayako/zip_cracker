use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::archive::{ArchiveHandler, TargetFile};
use crate::passwords::TOP_1000_PASSWORDS;

/// 字典攻击结果
pub struct DictionaryResult {
    pub password: Option<String>,
    pub total_tested: u64,
    pub elapsed_secs: f64,
}

/// 获取默认字典路径 (~/.archive_cracker/dictionary.txt)
pub fn get_default_dictionary_path() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户主目录");
    home.join(".archive_cracker").join("dictionary.txt")
}

/// 确保字典目录和文件存在，首次创建时写入内置 Top1000 密码
pub fn ensure_dictionary_exists(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        // 创建字典文件，带注释头和内置密码
        let mut file = File::create(path)?;
        writeln!(file, "# Archive Cracker 密码字典")?;
        writeln!(file, "# 每行一个密码，# 开头为注释")?;
        writeln!(file, "# 破解成功的密码会自动追加到此文件")?;
        writeln!(file, "# 你也可以主动记录密码到这个文件中以加速破解")?;
        writeln!(file)?;
        writeln!(file, "# === 内置 Top 1000 常用密码 ===")?;
        for password in TOP_1000_PASSWORDS {
            writeln!(file, "{}", password)?;
        }
        writeln!(file)?;
        writeln!(file, "# === 用户密码（破解成功自动添加）===")?;
    }
    Ok(())
}

/// 追加密码到字典（去重）
pub fn append_to_dictionary(dict_path: &PathBuf, password: &str) -> std::io::Result<bool> {
    // 先检查是否已存在
    if dict_path.exists() {
        let existing = load_dictionary(&dict_path.to_string_lossy())?;
        if existing.contains(&password.to_string()) {
            return Ok(false); // 密码已存在
        }
    }

    // 追加密码
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dict_path)?;
    writeln!(file, "{}", password)?;
    Ok(true)
}

/// 从字典文件加载密码列表
pub fn load_dictionary(path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let passwords: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    Ok(passwords)
}

/// 从字典文件加载密码列表（去重）
pub fn load_dictionary_unique(path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut seen = HashSet::new();
    let passwords: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| seen.insert(line.clone()))
        .collect();

    Ok(passwords)
}

/// 执行字典攻击
pub fn dictionary_attack(
    archive_path: &str,
    dict_path: &str,
    target: &TargetFile,
    file_count: usize,
    handler: &dyn ArchiveHandler,
) -> DictionaryResult {
    let num_cpus = num_cpus::get();

    // 加载字典（去重）
    let passwords = load_dictionary_unique(dict_path).expect("无法加载字典文件");

    println!(
        "=== {} 密码字典攻击器 (Rust 多线程版) ===",
        handler.format_name()
    );
    println!("目标文件: {}", archive_path);
    println!("CPU 核心数: {}", num_cpus);
    println!("字典文件: {}", dict_path);
    println!("字典条目: {} 个密码", passwords.len());
    println!();
    println!("检测到目标文件: {} (索引 {})", target.name, target.index);
    println!("文件扩展名: .{}", target.extension);
    println!("验证方式: 使用 infer 库检测解密后内容是否匹配扩展名");
    println!();
    println!("压缩包包含 {} 个文件", file_count);
    println!();
    println!("开始破解...");

    let start = Instant::now();
    let total_tested = passwords.len() as u64;

    let found = Arc::new(AtomicBool::new(false));
    let password = passwords
        .par_iter()
        .find_any(|password| {
            if found.load(Ordering::Relaxed) {
                return false;
            }
            if handler.try_password(archive_path, password, target) {
                found.store(true, Ordering::Relaxed);
                return true;
            }
            false
        })
        .cloned();

    let elapsed = start.elapsed();

    DictionaryResult {
        password,
        total_tested,
        elapsed_secs: elapsed.as_secs_f64(),
    }
}
