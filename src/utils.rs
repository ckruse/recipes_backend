use std::ffi::OsStr;
use std::path::Path;

use anyhow::Result;
use exif::{Exif, In, Tag};
use image::DynamicImage;

pub fn image_base_path() -> String {
    std::env::var("PICTURE_DIR").expect("env variable PICTURE_DIR not set")
}

pub fn avatar_base_path() -> String {
    std::env::var("AVATAR_DIR").expect("env variable AVATAR_DIR not set")
}

pub fn read_exif(path: &str) -> Result<Exif> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    Ok(exifreader.read_from_container(&mut bufreader)?)
}

pub fn correct_orientation(mut img: DynamicImage, orientation: u32) -> DynamicImage {
    if orientation <= 1 || orientation > 8 {
        return img;
    }

    if orientation >= 5 {
        img = img.rotate90().fliph();
    }

    if orientation == 3 || orientation == 4 || orientation == 7 || orientation == 8 {
        img = img.rotate180();
    }

    if orientation % 2 == 0 {
        img = img.fliph();
    }

    img
}

pub fn get_orientation(exif: &Exif) -> u32 {
    let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) else {
        return 0;
    };

    match orientation.value.get_uint(0).unwrap_or_default() {
        v @ 1..=8 => v,
        _ => 0,
    }
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}
