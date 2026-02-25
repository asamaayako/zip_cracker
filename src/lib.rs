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

/// 密码破解成功结果
#[derive(Debug, Clone)]
pub struct CrackSuccess {
    /// 找到的密码
    pub password: String,
    /// 总共测试的密码数量
    pub total_tested: u64,
    /// 总耗时（秒）
    pub elapsed_secs: f64,
}

impl CrackSuccess {
    /// 计算平均速度（密码/秒）
    pub fn speed(&self) -> f64 {
        if self.elapsed_secs > 0.0 {
            self.total_tested as f64 / self.elapsed_secs
        } else {
            0.0
        }
    }
}

/// 密码破解失败结果
#[derive(Debug, Clone)]
pub struct CrackFailure {
    /// 总共测试的密码数量
    pub total_tested: u64,
    /// 总耗时（秒）
    pub elapsed_secs: f64,
}

impl CrackFailure {
    /// 计算平均速度（密码/秒）
    pub fn speed(&self) -> f64 {
        if self.elapsed_secs > 0.0 {
            self.total_tested as f64 / self.elapsed_secs
        } else {
            0.0
        }
    }
}

/// 密码破解错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum CrackError {
    #[error("不支持的压缩包格式（支持: ZIP, 7z）")]
    UnsupportedFormat,

    #[error("未找到可识别扩展名的加密文件")]
    NoRecognizableFile,

    #[error("--length 和 --max-length 不能同时使用")]
    ConflictingLengthParams,

    #[error("--min-length ({0}) 不能大于 --max-length ({1})")]
    InvalidLengthRange(usize, usize),

    #[error("密码长度不能为 0")]
    ZeroLength,

    #[error("未找到密码")]
    NotFound(CrackFailure),
}

/// 密码破解结果类型
pub type CrackResult = Result<CrackSuccess, CrackError>;

/// 执行密码破解
///
/// # 参数
/// - `args`: CLI 参数配置
///
/// # 返回
/// 成功返回 `Ok(CrackSuccess)` 包含密码和统计信息
/// 失败返回 `Err(CrackError)` 包含错误类型和统计信息（如适用）
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
/// match crack_archive(args) {
///     Ok(success) => {
///         println!("密码: {}", success.password);
///         println!("速度: {:.0} 次/秒", success.speed());
///     }
///     Err(e) => eprintln!("错误: {}", e),
/// }
/// ```
pub fn crack_archive(args: Args) -> CrackResult {
    let archive_path = &args.archive_path;

    // 检测压缩包格式
    let format = ArchiveFormat::detect(archive_path)
        .ok_or(CrackError::UnsupportedFormat)?;
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
    let target = handler
        .detect_target(archive_path)
        .ok_or(CrackError::NoRecognizableFile)?;

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
            (Some(_), Some(_)) => return Err(CrackError::ConflictingLengthParams),
            (None, None) => (1, 5),
        };

        if min_len > max_len {
            return Err(CrackError::InvalidLengthRange(min_len, max_len));
        }

        if min_len == 0 {
            return Err(CrackError::ZeroLength);
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

    // 如果找到密码，保存到默认字典并返回成功
    if let Some(password) = found_password {
        let _ = append_to_dictionary(&default_dict_path, &password);

        Ok(CrackSuccess {
            password,
            total_tested,
            elapsed_secs: total_elapsed,
        })
    } else {
        Err(CrackError::NotFound(CrackFailure {
            total_tested,
            elapsed_secs: total_elapsed,
        }))
    }
}
