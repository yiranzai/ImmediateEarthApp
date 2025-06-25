// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use base64::{engine::general_purpose, Engine};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use std::io::Cursor;
use tauri::command;

#[tauri::command]
async fn fetch_and_stitch_tiles(
    urls: Vec<String>,
    tiles_per_row: u32,
    tile_size: u32,
) -> Result<String, String> {
    // 1. 并发下载所有瓦片
    let client = reqwest::Client::new();
    let mut handles = Vec::new();
    for (idx, url) in urls.iter().enumerate() {
        let client = client.clone();
        let url = url.clone();
        handles.push(tauri::async_runtime::spawn(async move {
            // 日志：开始下载瓦片
            println!("开始下载第{}个瓦片: {}", idx + 1, url);
            let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            // 判断字节数是否为 2834
            println!("第{}个瓦片字节数为{}", idx + 1, bytes.len());
            if bytes.len() == 2834 {
                println!(
                    "第{}个瓦片字节数为2834，判定为无效瓦片，终止本次拼接。",
                    idx + 1
                );
                return Err("无效瓦片数据".to_string());
            }
            let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
            // 日志：下载并解码完成，打印图片宽高
            println!(
                "第{}个瓦片下载并解码完成，宽度: {}，高度: {}",
                idx + 1,
                img.width(),
                img.height()
            );
            Ok::<DynamicImage, String>(img)
        }));
    }

    // 2. 收集所有图片
    let mut images = Vec::new();
    let mut has_invalid_tile = false;
    for (idx, handle) in handles.into_iter().enumerate() {
        let img = handle.await.map_err(|e| e.to_string())??;
        // 日志：收集到第N张图片
        println!("已收集第{}张图片", idx + 1);
        // 检查是否为无效瓦片
        if img.width() == 2834 && img.height() == 2834 {
            println!(
                "第{}张图片为无效瓦片，宽高均为2834，终止本次拼接。",
                idx + 1
            );
            has_invalid_tile = true;
        }
        images.push(img);
    }
    if has_invalid_tile {
        return Err("无效瓦片数据".to_string());
    }

    // 3. 拼接图片
    let tiles_per_col = (urls.len() as u32 + tiles_per_row - 1) / tiles_per_row;
    let mut stitched =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(tile_size * tiles_per_row, tile_size * tiles_per_col);

    for (i, img) in images.iter().enumerate() {
        let x = (i as u32 % tiles_per_row) * tile_size;
        let y = (i as u32 / tiles_per_row) * tile_size;
        // 日志：开始拼接第N张图片
        println!("开始拼接第{}张图片，目标位置: ({}, {})", i + 1, x, y);
        // 拷贝瓦片到目标图像
        for ty in 0..tile_size {
            for tx in 0..tile_size {
                // 直接获取像素
                let pixel = img.get_pixel(tx, ty);
                stitched.put_pixel(x + tx, y + ty, pixel);
            }
        }
    }

    // 4. 编码为 PNG 并转 base64
    let mut buf = Vec::new();
    DynamicImage::ImageRgba8(stitched)
        .write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;
    let encoded = general_purpose::STANDARD.encode(&buf);

    Ok(encoded)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|_app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = tauri::Manager::get_webview_window(_app, "main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(tauri::generate_handler![fetch_and_stitch_tiles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
