mod archive;
mod attack;
mod charset;
mod cli;
mod passwords;

use clap::Parser;

use archive::{get_handler, ArchiveFormat};
use attack::{
    append_to_dictionary, bruteforce_attack, dictionary_attack, ensure_dictionary_exists,
    get_default_dictionary_path,
};
use cli::Args;

fn main() {
    let args = Args::parse();
    let archive_path = &args.archive_path;

    // 检测压缩包格式
    let format = ArchiveFormat::detect(archive_path)
        .expect("不支持的压缩包格式（支持: ZIP, 7z, RAR）");
    let handler = get_handler(format);

    // 获取字典路径（默认或用户指定）
    let default_dict_path = get_default_dictionary_path();
    let dict_path = args
        .dictionary
        .as_ref()
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| default_dict_path.clone());

    // 确保默认字典存在
    if let Err(e) = ensure_dictionary_exists(&default_dict_path) {
        eprintln!("警告: 无法创建默认字典文件: {}", e);
    }

    // 自动检测目标文件
    let target = handler
        .detect_target(archive_path)
        .expect("未找到可识别扩展名的加密文件");

    // 获取文件数量
    let file_count = handler.file_count(archive_path).unwrap_or(0);

    let mut found_password: Option<String> = None;
    let mut total_tested: u64 = 0;
    let mut total_elapsed: f64 = 0.0;

    // 第一阶段：字典攻击（优先）
    if !args.skip_dictionary && dict_path.exists() {
        println!("=== 阶段 1: 字典攻击 ===");
        let result = dictionary_attack(
            archive_path,
            &dict_path.to_string_lossy(),
            &target,
            file_count,
            handler.as_ref(),
        );

        total_tested += result.total_tested;
        total_elapsed += result.elapsed_secs;

        if let Some(pwd) = result.password {
            found_password = Some(pwd);
        }
    }

    // 第二阶段：暴力破解（如果字典失败且指定了长度参数）
    if found_password.is_none() {
        let has_length_params = args.length.is_some() || args.max_length.is_some();

        if has_length_params {
            println!();
            println!("=== 阶段 2: 暴力破解 ===");

            // 确定密码长度范围
            let (min_len, max_len) = match (args.length, args.max_length) {
                (Some(len), None) => (len, len),
                (None, Some(max)) => (args.min_length, max),
                (Some(_), Some(_)) => {
                    eprintln!("错误: --length 和 --max-length 不能同时使用");
                    std::process::exit(1);
                }
                (None, None) => unreachable!(),
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

            let result = bruteforce_attack(attack::bruteforce::BruteforceParams {
                archive_path,
                charsets: &args.charset,
                min_len,
                max_len,
                target: &target,
                file_count,
                handler: handler.as_ref(),
            });

            total_tested += result.total_tested;
            total_elapsed += result.elapsed_secs;

            if let Some(pwd) = result.password {
                found_password = Some(pwd);
            }
        } else if args.skip_dictionary {
            eprintln!("错误: 跳过字典攻击时必须指定 --length 或 --max-length 参数");
            std::process::exit(1);
        }
    }

    // 输出最终结果
    println!();
    println!("=== 最终结果 ===");
    print_result(&found_password, total_tested, total_elapsed);

    // 如果找到密码，追加到默认字典
    if let Some(ref pwd) = found_password {
        match append_to_dictionary(&default_dict_path, pwd) {
            Ok(true) => {
                println!("📝 密码已保存到字典: {}", default_dict_path.display());
            }
            Ok(false) => {
                // 密码已存在，不需要提示
            }
            Err(e) => {
                eprintln!("警告: 无法保存密码到字典: {}", e);
            }
        }
    }
}

fn print_result(password: &Option<String>, total_tested: u64, elapsed_secs: f64) {
    match password {
        Some(pwd) => {
            println!("✅ 密码找到: {}", pwd);
            println!("密码长度: {}", pwd.len());
        }
        None => {
            println!("❌ 未找到密码");
        }
    }

    println!("总耗时: {:.2} 秒", elapsed_secs);
    println!("已测试: {} 个密码", total_tested);

    if elapsed_secs > 0.0 {
        let speed = total_tested as f64 / elapsed_secs;
        println!("平均速度: {:.0} 次/秒", speed);
    }
}
