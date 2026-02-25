pub mod archive;
pub mod attack;
pub mod charset;
pub mod cli;
pub mod passwords;

use archive::{get_handler, ArchiveFormat};
use attack::{
    append_to_dictionary, bruteforce_attack, dictionary_attack, ensure_dictionary_exists,
    get_default_dictionary_path,
};
pub use cli::Args;

/// 密码破解结果
#[derive(Debug, Clone)]
pub struct CrackResult {
    /// 找到的密码（如果成功）
    pub password: Option<String>,
    /// 总共测试的密码数量
    pub total_tested: u64,
    /// 总耗时（秒）
    pub elapsed_secs: f64,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

impl CrackResult {
    /// 是否成功找到密码
    pub fn is_success(&self) -> bool {
        self.password.is_some()
    }

    /// 计算平均速度（密码/秒）
    pub fn speed(&self) -> f64 {
        if self.elapsed_secs > 0.0 {
            self.total_tested as f64 / self.elapsed_secs
        } else {
            0.0
        }
    }
}

/// 执行密码破解
///
/// # 参数
/// - `args`: CLI 参数配置
///
/// # 返回
/// 返回破解结果，包含密码、测试次数、耗时等信息
///
/// # 示例
/// ```no_run
/// use archive_cracker::{crack_archive, Args};
///
/// let args = Args {
///     archive_path: "file.zip".to_string(),
///     dictionary: None,
///     charset: vec![],
///     length: Some(4),
///     max_length: None,
///     min_length: 1,
///     skip_dictionary: false,
/// };
///
/// let result = crack_archive(args);
/// if result.is_success() {
///     println!("密码: {}", result.password.unwrap());
/// }
/// ```
pub fn crack_archive(args: Args) -> CrackResult {
    let archive_path = &args.archive_path;

    // 检测压缩包格式
    let format = match ArchiveFormat::detect(archive_path) {
        Some(f) => f,
        None => {
            return CrackResult {
                password: None,
                total_tested: 0,
                elapsed_secs: 0.0,
                error: Some("不支持的压缩包格式（支持: ZIP, 7z）".to_string()),
            };
        }
    };
    let handler = get_handler(format);

    // 获取字典路径
    let default_dict_path = get_default_dictionary_path();
    let dict_path = args
        .dictionary
        .as_ref()
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| default_dict_path.clone());

    // 确保默认字典存在
    let _ = ensure_dictionary_exists(&default_dict_path);

    // 检测目标文件
    let target = match handler.detect_target(archive_path) {
        Some(t) => t,
        None => {
            return CrackResult {
                password: None,
                total_tested: 0,
                elapsed_secs: 0.0,
                error: Some("未找到可识别扩展名的加密文件".to_string()),
            };
        }
    };

    let file_count = handler.file_count(archive_path).unwrap_or(0);

    let mut found_password: Option<String> = None;
    let mut total_tested: u64 = 0;
    let mut total_elapsed: f64 = 0.0;

    // 第一阶段：字典攻击
    if !args.skip_dictionary && dict_path.exists() {
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

    // 第二阶段：暴力破解
    if found_password.is_none() {
        let (min_len, max_len) = match (args.length, args.max_length) {
            (Some(len), None) => (len, len),
            (None, Some(max)) => (args.min_length, max),
            (Some(_), Some(_)) => {
                return CrackResult {
                    password: None,
                    total_tested,
                    elapsed_secs: total_elapsed,
                    error: Some("--length 和 --max-length 不能同时使用".to_string()),
                };
            }
            (None, None) => (1, 5),
        };

        if min_len > max_len {
            return CrackResult {
                password: None,
                total_tested,
                elapsed_secs: total_elapsed,
                error: Some(format!(
                    "--min-length ({}) 不能大于 --max-length ({})",
                    min_len, max_len
                )),
            };
        }

        if min_len == 0 {
            return CrackResult {
                password: None,
                total_tested,
                elapsed_secs: total_elapsed,
                error: Some("密码长度不能为 0".to_string()),
            };
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
    }

    // 如果找到密码，保存到默认字典
    if let Some(ref pwd) = found_password {
        let _ = append_to_dictionary(&default_dict_path, pwd);
    }

    CrackResult {
        password: found_password,
        total_tested,
        elapsed_secs: total_elapsed,
        error: None,
    }
}
