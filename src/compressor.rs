use anyhow::Result;
use image::{imageops::FilterType::Gaussian, GenericImageView};
use libwebp_sys::WebPEncodeRGBA;
use tracing::{info, debug};
use std::path::Path;

pub fn encode_webp(input_image: &[u8], width: u32, height: u32, quality: i32) -> Result<Vec<u8>> {
    unsafe {
        let mut out_buf = std::ptr::null_mut();
        let stride = width as i32 * 4;
        debug!("start encoding");
        let len = WebPEncodeRGBA(
            input_image.as_ptr(),
            width as i32,
            height as i32,
            stride,
            quality as f32,
            &mut out_buf,
        );
        Ok(std::slice::from_raw_parts(out_buf, len as usize).into())
    }
}

pub async fn compress(input: impl AsRef<Path>, output: impl AsRef<Path>, scale: f32) -> Result<()> {
    let input_data = tokio::fs::read(input).await?;
    let input_img = image::load_from_memory(&input_data)?;
    let (w, h) = input_img.dimensions();
    let w = (w as f32 * scale) as u32;
    let h = (h as f32 * scale) as u32;
    if scale != 1.0 {
        input_img.resize(w, h, Gaussian);
    }

    let mut raw_data = input_img.as_bytes();
    let tmp;
    if input_img.color() != image::ColorType::Rgba8 {
        tmp = input_img.to_rgba8();
        raw_data = tmp.as_ref();
    }

    let output_data = encode_webp(raw_data, w, h, 85)?;
    tokio::fs::write(output, &output_data).await?;

    Ok(())
}
