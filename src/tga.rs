use anyhow;
use bevy::{
    core::AsBytes,
    math::Vec2,
    render::texture::{ Texture, TextureFormat },
};
use image;


pub fn from_bytes(bytes: &[u8]) -> Result<Texture, anyhow::Error> {
    let img_format = image::ImageFormat::Tga;
    
    let dyn_img = image::load_from_memory_with_format(bytes, img_format)?;

    let width;
    let height;

    let data: Vec<u8>;
    let format: TextureFormat;

    match dyn_img {
        image::DynamicImage::ImageLuma8(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::R8Unorm;

            data = i.into_raw();
        }
        image::DynamicImage::ImageLumaA8(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rg8Unorm;

            data = i.into_raw();
        }
        image::DynamicImage::ImageRgb8(i) => {
            let i = image::DynamicImage::ImageRgb8(i).into_rgba();
            width = i.width();
            height = i.height();
            format = TextureFormat::Rgba8UnormSrgb;

            data = i.into_raw();
        }
        image::DynamicImage::ImageRgba8(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rgba8UnormSrgb;

            data = i.into_raw();
        }
        image::DynamicImage::ImageBgr8(i) => {
            let i = image::DynamicImage::ImageBgr8(i).into_bgra();

            width = i.width();
            height = i.height();
            format = TextureFormat::Bgra8UnormSrgb;

            data = i.into_raw();
        }
        image::DynamicImage::ImageBgra8(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Bgra8UnormSrgb;

            data = i.into_raw();
        }
        image::DynamicImage::ImageLuma16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::R16Uint;

            let raw_data = i.into_raw();

            data = raw_data.as_slice().as_bytes().to_owned();
        }
        image::DynamicImage::ImageLumaA16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rg16Uint;

            let raw_data = i.into_raw();

            data = raw_data.as_slice().as_bytes().to_owned();
        }

        image::DynamicImage::ImageRgb16(image) => {
            width = image.width();
            height = image.height();
            format = TextureFormat::Rgba16Uint;

            let mut local_data =
                Vec::with_capacity(width as usize * height as usize * format.pixel_size());

            for pixel in image.into_raw().chunks_exact(3) {
                // TODO unsafe_get in release builds?
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = u16::max_value();

                local_data.extend_from_slice(&r.to_ne_bytes());
                local_data.extend_from_slice(&g.to_ne_bytes());
                local_data.extend_from_slice(&b.to_ne_bytes());
                local_data.extend_from_slice(&a.to_ne_bytes());
            }

            data = local_data;
        }
        image::DynamicImage::ImageRgba16(i) => {
            width = i.width();
            height = i.height();
            format = TextureFormat::Rgba16Uint;

            let raw_data = i.into_raw();

            data = raw_data.as_slice().as_bytes().to_owned();
        }
    }

    let texture = Texture::new(Vec2::new(width as f32, height as f32), data, format);
    Ok(texture)
}
