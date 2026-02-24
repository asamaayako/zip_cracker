use unrar::Archive;

use super::common::{get_extension, is_infer_supported};
use super::{ArchiveHandler, TargetFile};

pub struct RarHandler;

impl ArchiveHandler for RarHandler {
    fn detect_target(&self, path: &str) -> Option<TargetFile> {
        let archive = Archive::new(path).open_for_listing().ok()?;

        for (i, entry) in archive.enumerate() {
            let entry = entry.ok()?;

            if entry.is_directory() {
                continue;
            }

            let name = entry.filename.to_string_lossy().to_string();
            if let Some(ext) = get_extension(&name) {
                if is_infer_supported(&ext) {
                    return Some(TargetFile {
                        index: i,
                        name,
                        extension: ext,
                    });
                }
            }
        }
        None
    }

    fn file_count(&self, path: &str) -> Result<usize, String> {
        let archive = Archive::new(path)
            .open_for_listing()
            .map_err(|_| "无法打开 RAR 文件")?;

        let count = archive.count();
        Ok(count)
    }

    fn try_password(&self, path: &str, password: &str, _target: &TargetFile) -> bool {
        // 尝试用密码打开 RAR 文件
        // 如果密码错误，迭代时会返回错误
        let archive = match Archive::with_password(path, password).open_for_listing() {
            Ok(a) => a,
            Err(_) => return false,
        };

        // 尝试读取所有条目，如果密码错误会失败
        for entry in archive {
            if entry.is_err() {
                return false;
            }
        }

        // 能成功读取所有条目，密码正确
        true
    }

    fn format_name(&self) -> &'static str {
        "RAR"
    }
}
