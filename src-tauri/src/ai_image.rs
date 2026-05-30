use image::{DynamicImage, GenericImage, GenericImageView, RgbaImage};

pub fn make_background_transparent(img: &mut RgbaImage, threshold: u8) {
  let corners = [
    img.get_pixel(0, 0),
    img.get_pixel(img.width() - 1, 0),
    img.get_pixel(0, img.height() - 1),
    img.get_pixel(img.width() - 1, img.height() - 1),
  ];
  let bg_r = corners.iter().map(|p| p[0] as u32).sum::<u32>() / 4;
  let bg_g = corners.iter().map(|p| p[1] as u32).sum::<u32>() / 4;
  let bg_b = corners.iter().map(|p| p[2] as u32).sum::<u32>() / 4;

  for pixel in img.pixels_mut() {
    let dr = (pixel[0] as i32 - bg_r as i32).abs() as u32;
    let dg = (pixel[1] as i32 - bg_g as i32).abs() as u32;
    let db = (pixel[2] as i32 - bg_b as i32).abs() as u32;
    let dist = ((dr * dr + dg * dg + db * db) as f64).sqrt() as u8;
    if dist < threshold {
      pixel[3] = 0;
    }
  }
}

pub fn auto_crop_to_content(img: &RgbaImage) -> RgbaImage {
  let (w, h) = (img.width(), img.height());
  let mut min_x = w;
  let mut min_y = h;
  let mut max_x = 0u32;
  let mut max_y = 0u32;

  for y in 0..h {
    for x in 0..w {
      let pixel = img.get_pixel(x, y);
      if pixel[3] > 0 {
        if x < min_x { min_x = x; }
        if y < min_y { min_y = y; }
        if x > max_x { max_x = x; }
        if y > max_y { max_y = y; }
      }
    }
  }

  if min_x > max_x {
    return img.clone();
  }

  let crop_w = max_x - min_x + 1;
  let crop_h = max_y - min_y + 1;
  img.view(min_x, min_y, crop_w, crop_h).to_image()
}

pub fn quantize_colors(img: &mut RgbaImage, colors: usize) {
  let bits = (colors as f64).log2().ceil() as u32;
  let levels = 2u32.pow(bits.min(8));
  let step = 255.0 / (levels - 1) as f64;

  for pixel in img.pixels_mut() {
    for i in 0..3 {
      let v = pixel[i] as f64;
      let quantized = ((v / step).round() * step) as u8;
      pixel[i] = quantized;
    }
  }
}

pub fn compose_spritesheet(frames: &[DynamicImage], frames_per_row: u32) -> Result<RgbaImage, String> {
  if frames.is_empty() {
    return Err("no frames to compose".into());
  }

  let frame_w = frames.iter().map(|f| f.width()).max().unwrap_or(1);
  let frame_h = frames.iter().map(|f| f.height()).max().unwrap_or(1);
  let rows = ((frames.len() as f32) / (frames_per_row as f32)).ceil() as u32;
  let canvas_w = frame_w * frames_per_row;
  let canvas_h = frame_h * rows;

  let mut canvas = RgbaImage::new(canvas_w, canvas_h);

  for (idx, frame) in frames.iter().enumerate() {
    let col = (idx as u32) % frames_per_row;
    let row = (idx as u32) / frames_per_row;
    let x = col * frame_w;
    let y = row * frame_h;

    let rgba = frame.to_rgba8();
    canvas.copy_from(&rgba, x, y)
      .map_err(|e| format!("failed to paste frame {}: {}", idx, e))?;
  }

  Ok(canvas)
}
