use base64::Engine;
use image::imageops::FilterType;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ImageItem {
    pub name: String,
    pub path: String,
    pub thumbnail: String,
}

fn is_supported_image(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
        Some(ext) => matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp"),
        None => false,
    }
}

pub fn get_images_from_folder(folder_path: &Path) -> Vec<ImageItem> {
    if !folder_path.exists() || !folder_path.is_dir() {
        return Vec::new();
    }

    let cache_dir = folder_path.join(".cache").join("thumbnails");
    let _ = fs::create_dir_all(&cache_dir);

    let entries = match fs::read_dir(folder_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_supported_image(&path) {
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let safe_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");

            let thumb_file_name = format!("{}_vthumb.jpg", safe_stem);
            let thumb_path = cache_dir.join(&thumb_file_name);

            let should_generate = match (fs::metadata(&path), fs::metadata(&thumb_path)) {
                (Ok(src_meta), Ok(thumb_meta)) => {
                    match (src_meta.modified(), thumb_meta.modified()) {
                        (Ok(src_time), Ok(thumb_time)) => src_time > thumb_time,
                        _ => false,
                    }
                }
                (Ok(_), Err(_)) => true,
                _ => false,
            };

            if should_generate {
                if let Ok(mut img) = image::open(&path) {
                    let (w, h) = (img.width(), img.height());
                    if w > 0 && h > 0 {
                        let target_aspect = 9.0 / 16.0;
                        let current_aspect = w as f64 / h as f64;

                        let cropped = if current_aspect > target_aspect {
                            let crop_w = (h as f64 * target_aspect).round() as u32;
                            let crop_x = (w - crop_w) / 2;
                            img.crop(crop_x, 0, crop_w, h)
                        } else {
                            let crop_h = (w as f64 / target_aspect).round() as u32;
                            let crop_y = (h - crop_h) / 2;
                            img.crop(0, crop_y, w, crop_h)
                        };

                        let resized = cropped.resize_exact(450, 800, FilterType::Lanczos3);
                        let _ = resized.save(&thumb_path);
                    }
                }
            }

            if thumb_path.exists() {
                if let Ok(bytes) = fs::read(&thumb_path) {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    items.push(ImageItem {
                        name: file_name,
                        path: path.to_string_lossy().to_string(),
                        thumbnail: format!("data:image/jpeg;base64,{}", encoded),
                    });
                }
            }
        }
    }

    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}
