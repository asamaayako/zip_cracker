use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::charset::{get_combined_charset, index_to_password};
use crate::cli::Charset;
use crate::zip_utils::try_password;

/// 暴力破解结果
pub struct BruteforceResult {
    pub password: Option<String>,
    pub total_tested: u64,
    pub elapsed_secs: f64,
}

/// 暴力破解参数
pub struct BruteforceParams<'a> {
    pub zip_path: &'a str,
    pub charsets: &'a [Charset],
    pub min_len: usize,
    pub max_len: usize,
    pub target_index: usize,
    pub target_ext: &'a str,
    pub target_name: &'a str,
    pub file_count: usize,
}

/// 执行暴力破解攻击
pub fn bruteforce_attack(params: BruteforceParams) -> BruteforceResult {
    let num_cpus = num_cpus::get();
    let (charset_name, chars) = get_combined_charset(params.charsets);

    println!("=== ZIP 密码暴力破解器 (Rust 多线程版) ===");
    println!("目标文件: {}", params.zip_path);
    println!("CPU 核心数: {}", num_cpus);
    println!("字符集: {} ({}字符)", charset_name, chars.len());

    if params.min_len == params.max_len {
        println!("密码长度: {}", params.min_len);
        let total_combinations = (chars.len() as u64).pow(params.min_len as u32);
        println!(
            "密码空间: {}^{} = {} 组合",
            chars.len(),
            params.min_len,
            total_combinations
        );
    } else {
        println!(
            "密码长度: {} ~ {} (递增模式)",
            params.min_len, params.max_len
        );
        let total_combinations: u64 = (params.min_len..=params.max_len)
            .map(|len| (chars.len() as u64).pow(len as u32))
            .sum();
        println!(
            "密码空间: {} 组合 (长度{}到{}的总和)",
            total_combinations, params.min_len, params.max_len
        );
    }
    println!();

    println!(
        "检测到目标文件: {} (索引 {})",
        params.target_name, params.target_index
    );
    println!("文件扩展名: .{}", params.target_ext);
    println!("验证方式: 使用 infer 库检测解密后内容是否匹配扩展名");
    println!();

    println!("ZIP 文件包含 {} 个文件", params.file_count);
    println!();
    println!("开始破解...");

    let found = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let mut total_tested: u64 = 0;
    let mut result_password: Option<String> = None;

    // 从最小长度到最大长度逐一尝试
    for current_len in params.min_len..=params.max_len {
        if found.load(Ordering::Relaxed) {
            break;
        }

        if params.min_len != params.max_len {
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
            if try_password(params.zip_path, &pwd, params.target_index, params.target_ext) {
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

    BruteforceResult {
        password: result_password,
        total_tested,
        elapsed_secs: elapsed.as_secs_f64(),
    }
}
