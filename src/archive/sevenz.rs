use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::TempDir;

use super::common::{get_extension, is_infer_supported, verify_content};
use super::{ArchiveHandler, TargetFile};

pub struct SevenZHandler;

impl ArchiveHandler for SevenZHandler {
    fn detect_target(&self, path: &str) -> Option<TargetFile> {
        // 尝试用空密码打开来获取文件列表
        let mut file = File::open(path).ok()?;
        let len = file.metadata().ok()?.len();
        let archive = sevenz_rust::Archive::read(&mut file, len, &[]).ok()?;

        for (i, entry) in archive.files.iter().enumerate() {
            if entry.is_directory() {
                continue;
            }

            let name = entry.name();
            if let Some(ext) = get_extension(name) {
                if is_infer_supported(&ext) {
                    return Some(TargetFile {
                        index: i,
                        name: name.to_string(),
                        extension: ext,
                    });
                }
            }
        }
        None
    }

    fn file_count(&self, path: &str) -> Result<usize, String> {
        let mut file = File::open(path).map_err(|_| "无法打开 7z 文件")?;
        let len = file.metadata().map_err(|_| "无法获取文件大小")?.len();
        let archive = sevenz_rust::Archive::read(&mut file, len, &[])
            .map_err(|_| "无法解析 7z 文件（可能已加密）")?;
        Ok(archive.files.len())
    }

    fn try_password(&self, path: &str, password: &str, target: &TargetFile) -> bool {
        // 创建临时目录用于解压
        let temp_dir = match TempDir::new() {
            Ok(d) => d,
            Err(_) => return false,
        };

        // 尝试用密码解压
        let result = sevenz_rust::decompress_file_with_password(
            Path::new(path),
            temp_dir.path(),
            password.into(),
        );

        if result.is_err() {
            return false;
        }

        // 查找并验证目标文件
        let target_path = temp_dir.path().join(&target.name);
        if !target_path.exists() {
            // 可能文件名包含目录，尝试只用文件名
            let filename = Path::new(&target.name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&target.name);

            // 递归查找文件
            if let Some(found_path) = find_file_recursive(temp_dir.path(), filename) {
                return verify_file(&found_path, &target.extension);
            }
            return false;
        }

        verify_file(&target_path, &target.extension)
    }

    fn format_name(&self) -> &'static str {
        "7z"
    }
}

fn find_file_recursive(dir: &Path, filename: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, filename) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
                return Some(path);
            }
        }
    }
    None
}

fn verify_file(path: &Path, expected_ext: &str) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = vec![0u8; 8192];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    verify_content(&buffer[..bytes_read], expected_ext)
}
