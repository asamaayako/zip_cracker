use clap::Parser;
use archive_cracker::{crack_archive, Args};

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

    match result.password {
        Some(ref pwd) => {
            println!("✅ 密码找到: {}", pwd);
            println!("密码长度: {}", pwd.len());
        }
        None => {
            if let Some(ref err) = result.error {
                println!("❌ 错误: {}", err);
            } else {
                println!("❌ 未找到密码");
            }
        }
    }

    println!("总耗时: {:.2} 秒", result.elapsed_secs);
    println!("已测试: {} 个密码", result.total_tested);

    if result.elapsed_secs > 0.0 {
        println!("平均速度: {:.0} 次/秒", result.speed());
    }
}
