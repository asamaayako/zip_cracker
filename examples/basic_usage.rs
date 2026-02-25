use archive_cracker::{Args, CrackError, cli::Charset, crack_archive};

fn main() {
    // 示例 1: 只使用字典攻击 + 默认 1-5 位暴力破解
    let args = Args {
        archive_path: "test.zip".to_string(),
        dictionary: None,
        charset: vec![Charset::Lower, Charset::Upper, Charset::Digit],
        length: None,
        max_length: None,
        min_length: 1,
        skip_dictionary: false,
    };

    match crack_archive(&args) {
        Ok(success) => {
            println!("✅ 密码找到: {}", success.password);
            println!("   耗时: {:.2}秒", success.elapsed_secs);
            println!("   速度: {:.0} 次/秒", success.speed());
        }
        Err(CrackError::NotFound(failure)) => {
            println!("❌ 未找到密码");
            println!("   已测试: {} 个", failure.total_tested);
        }
        Err(e) => {
            eprintln!("❌ 错误: {e}");
        }
    }

    // 示例 2: 指定固定长度
    let args2 = Args {
        archive_path: "test.zip".to_string(),
        dictionary: None,
        charset: vec![Charset::Digit],
        length: Some(4), // 只破解 4 位数字
        max_length: None,
        min_length: 1,
        skip_dictionary: false,
    };

    if let Ok(success) = crack_archive(&args2) {
        println!("第二次尝试成功: {}", success.password);
    }

    // 示例 3: 指定范围
    let args3 = Args {
        archive_path: "test.zip".to_string(),
        dictionary: Some("/path/to/custom.txt".to_string()),
        charset: vec![Charset::Lower, Charset::Digit],
        length: None,
        max_length: Some(6), // 破解 1-6 位
        min_length: 1,
        skip_dictionary: false,
    };

    match crack_archive(&args3) {
        Ok(success) => println!("第三次尝试成功: {}", success.password),
        Err(e) => println!("第三次尝试失败: {e}"),
    }
}
