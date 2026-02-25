/// 从文件名提取扩展名（小写）
#[must_use] 
pub fn get_extension(filename: &str) -> Option<String> {
    filename
        .rsplit('.')
        .next()
        .filter(|ext| !ext.is_empty() && ext.len() <= 5)
        .map(str::to_lowercase)
}

/// 检查扩展名是否被 infer 库支持
#[must_use] 
pub fn is_infer_supported(ext: &str) -> bool {
    const SUPPORTED: &[&str] = &[
        // 图片
        "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "psd", "ico", "heif", "heic",
        "avif", "jxl", "cr2", "orf", "raf",
        // 视频
        "mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "flv", "3gp",
        // 音频
        "mp3", "flac", "wav", "ogg", "m4a", "aac", "aiff", "wma", "amr",
        // 压缩
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst", "lz4", "cab", "rpm", "deb",
        // 文档
        "pdf", "docx", "xlsx", "pptx", "odt", "ods", "odp", "epub", "rtf",
        // 字体
        "ttf", "otf", "woff", "woff2",
        // 可执行
        "exe", "dll", "elf", "dex", "wasm", "class",
        // 其他
        "swf", "sqlite", "nes", "crx", "lnk", "alias", "dcm",
    ];
    SUPPORTED.contains(&ext)
}

/// 使用 infer 库验证文件内容是否匹配期望的扩展名
#[must_use] 
pub fn verify_content(data: &[u8], expected_ext: &str) -> bool {
    if data.is_empty() {
        return false;
    }

    if let Some(kind) = infer::get(data) {
        kind.extension() == expected_ext
    } else {
        false
    }
}
