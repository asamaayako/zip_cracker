use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

/// 从文件名提取扩展名（小写）
fn get_extension(filename: &str) -> Option<String> {
    filename
        .rsplit('.')
        .next()
        .filter(|ext| !ext.is_empty() && ext.len() <= 5)
        .map(|ext| ext.to_lowercase())
}

/// 检查扩展名是否被 infer 库支持
fn is_infer_supported(ext: &str) -> bool {
    const SUPPORTED: &[&str] = &[
        // 图片
        "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "psd", "ico", "heif", "heic",
        "avif", "jxl", "cr2", "orf", "raf", // 视频
        "mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "flv", "3gp", // 音频
        "mp3", "flac", "wav", "ogg", "m4a", "aac", "aiff", "wma", "amr", // 压缩
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "lz4", "cab", "rpm", "deb",
        // 文档
        "pdf", "docx", "xlsx", "pptx", "odt", "ods", "odp", "epub", "rtf", // 字体
        "ttf", "otf", "woff", "woff2", // 可执行
        "exe", "dll", "elf", "dex", "wasm", "class", // 其他
        "swf", "sqlite", "nes", "crx", "lnk", "alias", "dcm",
    ];
    SUPPORTED.contains(&ext)
}

/// 自动检测 ZIP 中第一个 infer 可识别扩展名的非目录文件
pub fn detect_target_file(zip_path: &str) -> Option<(usize, String, String)> {
    let file = File::open(zip_path).ok()?;
    let archive = ZipArchive::new(file).ok()?;

    for i in 0..archive.len() {
        let name = archive.name_for_index(i)?;

        // 跳过目录
        if name.ends_with('/') {
            continue;
        }

        let name_owned = name.to_string();
        if let Some(ext) = get_extension(&name_owned) {
            // 只选择 infer 库支持的扩展名
            if is_infer_supported(&ext) {
                return Some((i, name_owned, ext));
            }
        }
    }
    None
}

/// 尝试用指定密码解密 ZIP 文件
pub fn try_password(zip_path: &str, password: &str, file_index: usize, expected_ext: &str) -> bool {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };

    // 尝试用密码解密指定索引的文件
    let result = archive.by_index_decrypt(file_index, password.as_bytes());
    match result {
        Ok(mut file) => {
            // 读取文件头部用于类型检测 (infer 需要至少前几百字节)
            let mut buffer = vec![0u8; 8192];
            let bytes_read = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => return false,
            };

            if bytes_read == 0 {
                return false;
            }

            // 使用 infer 库检测文件类型
            if let Some(kind) = infer::get(&buffer[..bytes_read]) {
                // 检查检测到的扩展名是否与期望的扩展名匹配
                kind.extension() == expected_ext
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// 获取 ZIP 文件中的文件数量
pub fn get_file_count(zip_path: &str) -> Result<usize, String> {
    let file = File::open(zip_path).map_err(|_| "无法打开 ZIP 文件")?;
    let archive = ZipArchive::new(file).map_err(|_| "无法解析 ZIP 文件")?;
    Ok(archive.len())
}
