pub mod common;
mod zip;
mod sevenz;

pub use self::zip::ZipHandler;
pub use self::sevenz::SevenZHandler;

use std::path::Path;

/// 压缩包格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
}

impl ArchiveFormat {
    /// 从文件路径自动检测格式
    pub fn detect(path: &str) -> Option<Self> {
        let path = Path::new(path);
        let ext = path.extension()?.to_str()?.to_lowercase();

        match ext.as_str() {
            "zip" => Some(ArchiveFormat::Zip),
            "7z" => Some(ArchiveFormat::SevenZ),
            _ => None,
        }
    }
}

/// 目标文件信息
#[derive(Debug, Clone)]
pub struct TargetFile {
    pub index: usize,
    pub name: String,
    pub extension: String,
}

/// 压缩包处理器 trait
pub trait ArchiveHandler: Send + Sync {
    /// 检测压缩包中的目标文件
    fn detect_target(&self, path: &str) -> Option<TargetFile>;

    /// 获取文件数量
    fn file_count(&self, path: &str) -> Result<usize, String>;

    /// 尝试密码
    fn try_password(&self, path: &str, password: &str, target: &TargetFile) -> bool;

    /// 格式名称
    fn format_name(&self) -> &'static str;
}

/// 获取对应格式的处理器
pub fn get_handler(format: ArchiveFormat) -> Box<dyn ArchiveHandler> {
    match format {
        ArchiveFormat::Zip => Box::new(ZipHandler),
        ArchiveFormat::SevenZ => Box::new(SevenZHandler),
    }
}
