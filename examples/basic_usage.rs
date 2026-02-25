use archive_cracker::{crack_archive, Args, cli::Charset};

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

    let result = crack_archive(args);

    if result.is_success() {
        println!("密码找到: {}", result.password.as_ref().unwrap());
        println!("耗时: {:.2}秒", result.elapsed_secs);
        println!("速度: {:.0} 次/秒", result.speed());
    } else if let Some(err) = result.error {
        eprintln!("错误: {}", err);
    } else {
        println!("未找到密码");
    }

    // 示例 2: 指定固定长度
    let args2 = Args {
        archive_path: "test.zip".to_string(),
        dictionary: None,
        charset: vec![Charset::Digit],
        length: Some(4),  // 只破解 4 位数字
        max_length: None,
        min_length: 1,
        skip_dictionary: false,
    };

    let result2 = crack_archive(args2);
    println!("第二次尝试: {:?}", result2.password);

    // 示例 3: 指定范围
    let args3 = Args {
        archive_path: "test.zip".to_string(),
        dictionary: Some("/path/to/custom.txt".to_string()),
        charset: vec![Charset::Lower, Charset::Digit],
        length: None,
        max_length: Some(6),  // 破解 1-6 位
        min_length: 1,
        skip_dictionary: false,
    };

    let result3 = crack_archive(args3);
    println!("第三次尝试: {:?}", result3.password);
}
