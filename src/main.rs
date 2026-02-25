use clap::Parser;
use archive_cracker::{crack_archive, Args, CrackError};

fn main() {
    let args = Args::parse();

    println!("=== Archive Cracker ===");
    println!("目标文件: {}", args.archive_path);
    println!();

    // 调用库函数执行破解
    let result = crack_archive(args);

    // 输出结果
    println!();
    println!("=== 最终结果 ===");

    match result {
        Ok(success) => {
            println!("✅ 密码找到: {}", success.password);
            println!("密码长度: {}", success.password.len());
            println!("总耗时: {:.2} 秒", success.elapsed_secs);
            println!("已测试: {} 个密码", success.total_tested);
            println!("平均速度: {:.0} 次/秒", success.speed());
        }
        Err(e) => {
            match e {
                CrackError::NotFound(failure) => {
                    println!("❌ 未找到密码");
                    println!("总耗时: {:.2} 秒", failure.elapsed_secs);
                    println!("已测试: {} 个密码", failure.total_tested);
                    if failure.elapsed_secs > 0.0 {
                        println!("平均速度: {:.0} 次/秒", failure.speed());
                    }
                }
                _ => {
                    println!("❌ 错误: {e}");
                }
            }
        }
    }
}
