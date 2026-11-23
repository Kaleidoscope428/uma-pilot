use image::{GrayImage, RgbaImage};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)] pub struct Template { pub path: PathBuf, pub image: RgbaImage }
#[derive(Clone, Debug, PartialEq)] pub struct Match { pub x: u32, pub y: u32, pub width: u32, pub height: u32, pub confidence: f32 }
#[derive(Debug, thiserror::Error)] pub enum VisionError { #[error("image: {0}")] Image(#[from] image::ImageError), #[error("template is larger than screenshot")] TemplateTooLarge, #[error("template has no visual variation")] FlatTemplate }
pub fn load_templates(directory: &Path) -> Result<Vec<Template>, VisionError> {
    let entries = match std::fs::read_dir(directory) { Ok(x) => x, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]), Err(_) => return Ok(vec![]) };
    Ok(entries.filter_map(Result::ok).map(|x| x.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))).filter_map(|path| image::open(&path).ok().map(|x| Template { path, image: x.to_rgba8() })).collect())
}
pub fn find(screenshot: &RgbaImage, template: &Template, threshold: f32) -> Result<Option<Match>, VisionError> {
    if template.image.width() > screenshot.width() || template.image.height() > screenshot.height() { return Err(VisionError::TemplateTooLarge); }
    let hay = grayscale(screenshot); let needle = grayscale(&template.image); let n = (needle.width()*needle.height()) as f64;
    let mean_n = needle.pixels().map(|p| p[0] as f64).sum::<f64>() / n; let denom_n = needle.pixels().map(|p| { let d=p[0] as f64-mean_n; d*d }).sum::<f64>().sqrt();
    if denom_n < 0.001 { return Err(VisionError::FlatTemplate); }
    let mut best = (f32::NEG_INFINITY, 0, 0);
    for y in 0..=hay.height()-needle.height() { for x in 0..=hay.width()-needle.width() {
        let mut sum=0.; for ty in 0..needle.height() { for tx in 0..needle.width() { sum += hay.get_pixel(x+tx,y+ty)[0] as f64; }}
        let mean = sum/n; let mut dot=0.; let mut denom=0.; for ty in 0..needle.height() { for tx in 0..needle.width() { let a=hay.get_pixel(x+tx,y+ty)[0] as f64-mean; let b=needle.get_pixel(tx,ty)[0] as f64-mean_n; dot+=a*b; denom+=a*a; }}
        let score = if denom > 0. { (dot/(denom.sqrt()*denom_n)) as f32 } else { -1. }; if score > best.0 { best=(score,x,y); }
    }}
    let confidence = ((best.0 + 1.0) / 2.0).clamp(0.0, 1.0); Ok((confidence >= threshold).then_some(Match { x:best.1,y:best.2,width:needle.width(),height:needle.height(),confidence }))
}
fn grayscale(image: &RgbaImage) -> GrayImage { image::imageops::grayscale(image) }

#[cfg(test)] mod tests { use super::*; use image::Rgba;
 fn img(w:u32,h:u32)->RgbaImage { RgbaImage::from_fn(w,h,|x,y| Rgba([(x*13+y*7) as u8,(x*3+y*17) as u8,(x*y) as u8,255])) }
 #[test] fn finds_embedded_template() { let shot=img(30,30); let crop=image::imageops::crop_imm(&shot,7,9,8,6).to_image(); let t=Template{path:"x.png".into(),image:crop}; let found=find(&shot,&t,.99).expect("valid").expect("matched"); assert_eq!((found.x,found.y),(7,9)); }
 #[test] fn rejects_different_image() { let shot=RgbaImage::from_pixel(20,20,Rgba([0,0,0,255])); let t=Template{path:"x.png".into(),image:img(4,4)}; assert!(find(&shot,&t,.85).expect("valid").is_none()); }
}
