use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

use super::{ArchiveHandler, TargetFile};
use crate::archive::common::{get_extension, is_infer_supported};

pub struct ZipHandler;

impl ArchiveHandler for ZipHandler {
    fn detect_target(&self, path: &str) -> Option<TargetFile> {
        let file = File::open(path).ok()?;
        let archive = ZipArchive::new(file).ok()?;

        for i in 0..archive.len() {
            let name = archive.name_for_index(i)?;

            // 跳过目录
            if name.ends_with('/') {
                continue;
            }

            let name_owned = name.to_string();
            if let Some(ext) = get_extension(&name_owned)
                && is_infer_supported(&ext)
            {
                return Some(TargetFile {
                    index: i,
                    name: name_owned,
                    extension: ext,
                });
            }
        }
        None
    }

    fn file_count(&self, path: &str) -> Result<usize, String> {
        let file = File::open(path).map_err(|_| "无法打开 ZIP 文件")?;
        let archive = ZipArchive::new(file).map_err(|_| "无法解析 ZIP 文件")?;
        Ok(archive.len())
    }

    fn try_password(&self, path: &str, password: &str, target: &TargetFile) -> bool {
        let Ok(file) = File::open(path) else {
            return false;
        };

        let Ok(mut archive) = ZipArchive::new(file) else {
            return false;
        };

        let result = archive.by_index_decrypt(target.index, password.as_bytes());
        match result {
            Ok(mut file) => {
                let mut buffer = vec![0u8; 8192];
                let Ok(bytes_read) = file.read(&mut buffer) else {
                    return false;
                };

                if bytes_read == 0 {
                    return false;
                }
                infer::get(&buffer[..bytes_read])
                    .is_some_and(|kind| kind.extension() == target.extension)
            }
            Err(_) => false,
        }
    }

    fn format_name(&self) -> &'static str {
        "ZIP"
    }
}
