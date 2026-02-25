use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::common::{get_extension, is_infer_supported, verify_content};
use super::{ArchiveHandler, TargetFile};

pub struct SevenZHandler;

impl ArchiveHandler for SevenZHandler {
    fn detect_target(&self, path: &str) -> Option<TargetFile> {
        let mut file = File::open(path).ok()?;
        let len = file.metadata().ok()?.len();
        let archive = sevenz_rust::Archive::read(&mut file, len, &[]).ok()?;

        // 收集所有可识别魔数的文件，选择最小的一个
        // 7z 解压时即使跳过写入，固实压缩仍需处理前面的数据
        // 选择最小文件可以减少验证时读取的数据量
        let mut candidates: Vec<(usize, String, String, u64)> = Vec::new();

        for (i, entry) in archive.files.iter().enumerate() {
            if entry.is_directory() {
                continue;
            }

            let name = entry.name();
            if let Some(ext) = get_extension(name)
                && is_infer_supported(&ext)
            {
                candidates.push((i, name.to_string(), ext, entry.size()));
            }
        }

        // 按文件大小排序，选择最小的
        candidates.sort_by_key(|(_, _, _, size)| *size);

        candidates
            .into_iter()
            .next()
            .map(|(index, name, extension, _)| TargetFile {
                index,
                name,
                extension,
            })
    }

    fn file_count(&self, path: &str) -> Result<usize, String> {
        let mut file = File::open(path).map_err(|_| "无法打开 7z 文件")?;
        let len = file.metadata().map_err(|_| "无法获取文件大小")?.len();
        let archive = sevenz_rust::Archive::read(&mut file, len, &[])
            .map_err(|_| "无法解析 7z 文件（可能已加密）")?;
        Ok(archive.files.len())
    }

    fn try_password(&self, path: &str, password: &str, target: &TargetFile) -> bool {
        let Ok(file) = File::open(path) else {
            return false;
        };

        let target_name = target.name.clone();
        let target_ext = target.extension.clone();
        let found = Arc::new(AtomicBool::new(false));
        let found_clone = Arc::clone(&found);

        // 使用自定义提取函数，只验证目标文件，不写入磁盘
        let result = sevenz_rust::decompress_with_extract_fn_and_password(
            file,
            std::path::Path::new("."), // 不会实际写入
            password.into(),
            move |entry, reader, _dest_path| {
                // 跳过目录
                if entry.is_directory() {
                    return Ok(true);
                }

                // 只处理目标文件
                if entry.name() == target_name {
                    // 读取文件内容到内存验证
                    let mut buffer = vec![0u8; 8192];
                    if let Ok(bytes_read) = reader.read(&mut buffer)
                        && bytes_read > 0
                        && verify_content(&buffer[..bytes_read], &target_ext)
                    {
                        found_clone.store(true, Ordering::Relaxed);
                    }
                    // 找到目标后可以停止（返回 false 停止遍历）
                    return Ok(false);
                }

                // 跳过其他文件（不写入磁盘）
                Ok(true)
            },
        );

        // 解压错误（密码错误）返回 false
        if result.is_err() {
            return false;
        }

        found.load(Ordering::Relaxed)
    }

    fn format_name(&self) -> &'static str {
        "7z"
    }
}
