mod attack;
mod charset;
mod cli;
mod passwords;
mod zip_utils;

use clap::Parser;

use attack::{
    append_to_dictionary, bruteforce_attack, dictionary_attack, ensure_dictionary_exists,
    get_default_dictionary_path,
};
use cli::{Args, AttackMode};
use zip_utils::{detect_target_file, get_file_count};

fn main() {
    let args = Args::parse();
    let zip_path = &args.zip_path;

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
    let (target_index, target_name, target_ext) =
        detect_target_file(zip_path).expect("未找到可识别扩展名的加密文件");

    // 获取文件数量
    let file_count = get_file_count(zip_path).expect("无法读取 ZIP 文件");

    let result = match args.mode {
        AttackMode::Dictionary => {
            let result = dictionary_attack(
                zip_path,
                &dict_path.to_string_lossy(),
                target_index,
                &target_ext,
                &target_name,
                file_count,
            );
            (result.password, result.total_tested, result.elapsed_secs)
        }
        AttackMode::Bruteforce => {
            // 确定密码长度范围
            let (min_len, max_len) = match (args.length, args.max_length) {
                (Some(len), None) => (len, len),
                (None, Some(max)) => (args.min_length, max),
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

            let result = bruteforce_attack(attack::bruteforce::BruteforceParams {
                zip_path,
                charsets: &args.charset,
                min_len,
                max_len,
                target_index,
                target_ext: &target_ext,
                target_name: &target_name,
                file_count,
            });
            (result.password, result.total_tested, result.elapsed_secs)
        }
    };

    // 输出结果并保存密码
    let (password, total_tested, elapsed_secs) = result;
    print_result(&password, total_tested, elapsed_secs);

    // 如果找到密码，追加到默认字典
    if let Some(ref pwd) = password {
        match append_to_dictionary(&default_dict_path, pwd) {
            Ok(true) => {
                println!(
                    "📝 密码已保存到字典: {}",
                    default_dict_path.display()
                );
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
            println!();
            println!("✅ 密码找到: {}", pwd);
            println!("密码长度: {}", pwd.len());
            println!("耗时: {:.2} 秒", elapsed_secs);
        }
        None => {
            println!();
            println!("❌ 未找到密码");
            println!("耗时: {:.2} 秒", elapsed_secs);
        }
    }

    let speed = total_tested as f64 / elapsed_secs;
    println!("已测试: {} 个密码", total_tested);
    println!("平均速度: {:.0} 次/秒", speed);
}
