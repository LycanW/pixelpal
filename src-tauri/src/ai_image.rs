use image::{DynamicImage, GenericImage, GenericImageView, RgbaImage};

/// Color distance in RGB space.
fn color_dist(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
  let dr = (r1 as i32 - r2 as i32).abs() as u32;
  let dg = (g1 as i32 - g2 as i32).abs() as u32;
  let db = (b1 as i32 - b2 as i32).abs() as u32;
  dr * dr + dg * dg + db * db
}

/// Make background transparent using flood-fill from image edges.
/// Only pixels connected to the edge AND matching the background color become transparent.
/// Internal pixels (like white eye highlights) are preserved.
pub fn make_background_transparent(img: &mut RgbaImage, threshold: u8) {
  let (w, h) = (img.width(), img.height());
  if w == 0 || h == 0 { return; }

  let threshold_sq = (threshold as u32) * (threshold as u32);

  // Sample background from four corners
  let corners = [
    img.get_pixel(0, 0),
    img.get_pixel(w - 1, 0),
    img.get_pixel(0, h - 1),
    img.get_pixel(w - 1, h - 1),
  ];
  let bg_r = (corners.iter().map(|p| p[0] as u32).sum::<u32>() / 4) as u8;
  let bg_g = (corners.iter().map(|p| p[1] as u32).sum::<u32>() / 4) as u8;
  let bg_b = (corners.iter().map(|p| p[2] as u32).sum::<u32>() / 4) as u8;

  let mut visited = vec![false; (w * h) as usize];
  let mut queue = Vec::new();

  // Seed from all edge pixels matching background color
  let mut seed = |x: u32, y: u32| {
    let idx = (y * w + x) as usize;
    if visited[idx] { return; }
    let p = img.get_pixel(x, y);
    if color_dist(p[0], p[1], p[2], bg_r, bg_g, bg_b) <= threshold_sq {
      visited[idx] = true;
      queue.push((x, y));
    }
  };

  for x in 0..w {
    seed(x, 0);
    seed(x, h - 1);
  }
  for y in 0..h {
    seed(0, y);
    seed(w - 1, y);
  }

  // BFS flood fill — only remove edge-connected background
  let mut head = 0;
  while head < queue.len() {
    let (x, y) = queue[head];
    head += 1;

    img.get_pixel_mut(x, y)[3] = 0;

    let neighbors = [
      (x.wrapping_sub(1), y), (x + 1, y),
      (x, y.wrapping_sub(1)), (x, y + 1),
    ];
    for (nx, ny) in neighbors {
      if nx < w && ny < h {
        let idx = (ny * w + nx) as usize;
        if !visited[idx] {
          let np = img.get_pixel(nx, ny);
          if color_dist(np[0], np[1], np[2], bg_r, bg_g, bg_b) <= threshold_sq {
            visited[idx] = true;
            queue.push((nx, ny));
          }
        }
      }
    }
  }

  // ── Post-pass: remove small isolated non-transparent artifacts ──
  // After flood fill, edge anti-aliasing may leave small colored fragments
  // near the silhouette.  We keep only the largest connected opaque blob.
  let mut label = vec![0u32; (w * h) as usize];
  let mut current_label = 0u32;
  let mut label_areas: Vec<u32> = Vec::new();

  for y in 0..h {
    for x in 0..w {
      let idx = (y * w + x) as usize;
      if img.get_pixel(x, y)[3] > 0 && label[idx] == 0 {
        current_label += 1;
        let mut area = 0u32;
        let mut q = vec![(x, y)];
        label[idx] = current_label;
        let mut qi = 0usize;
        while qi < q.len() {
          let (cx, cy) = q[qi];
          qi += 1;
          area += 1;
          let neighbors = [
            (cx.wrapping_sub(1), cy), (cx + 1, cy),
            (cx, cy.wrapping_sub(1)), (cx, cy + 1),
          ];
          for (nx, ny) in neighbors {
            if nx < w && ny < h {
              let nidx = (ny * w + nx) as usize;
              if img.get_pixel(nx, ny)[3] > 0 && label[nidx] == 0 {
                label[nidx] = current_label;
                q.push((nx, ny));
              }
            }
          }
        }
        label_areas.push(area);
      }
    }
  }

  if label_areas.is_empty() { return; }

  // Find the largest opaque connected component (the character)
  let mut max_area = 0u32;
  let mut max_label = 0u32;
  for (i, &area) in label_areas.iter().enumerate() {
    if area > max_area {
      max_area = area;
      max_label = (i + 1) as u32;
    }
  }

  // Remove everything except the largest component
  for y in 0..h {
    for x in 0..w {
      let idx = (y * w + x) as usize;
      if label[idx] != 0 && label[idx] != max_label {
        img.get_pixel_mut(x, y)[3] = 0;
      }
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

/// Split a horizontal row image into N equal-width frames.
pub fn split_row_into_frames(
  row_img: &mut DynamicImage,
  frame_count: u32,
) -> Result<Vec<DynamicImage>, String> {
  if frame_count == 0 {
    return Err("frame_count must be > 0".into());
  }
  let (w, h) = (row_img.width(), row_img.height());
  let slot_w = w / frame_count;
  if slot_w == 0 {
    return Err(format!("image width {} too small for {} frames", w, frame_count));
  }
  let mut frames = Vec::new();
  for i in 0..frame_count {
    let x = i * slot_w;
    let cw = if i == frame_count - 1 { w - x } else { slot_w };
    frames.push(row_img.crop(x, 0, cw, h));
  }
  Ok(frames)
}

/// Force an image into pixel-art style.
/// Steps: 1) auto-crop to content, 2) downscale to small pixel grid with nearest-neighbor
/// to get hard edges, 3) upscale back to target size, 4) optional color quantization.
pub fn pixelate(
  img: &DynamicImage,
  target_pixels: u32,
  quantize: Option<usize>,
) -> RgbaImage {
  let rgba = img.to_rgba8();
  let cropped = auto_crop_to_content(&rgba);
  let (cw, ch) = (cropped.width(), cropped.height());
  if cw == 0 || ch == 0 {
    return cropped;
  }

  // Determine downscale dimensions maintaining aspect ratio
  let ratio = cw as f32 / ch as f32;
  let (small_w, small_h) = if ratio >= 1.0 {
    (target_pixels, ((target_pixels as f32) / ratio).round().max(1.0) as u32)
  } else {
    (((target_pixels as f32) * ratio).round().max(1.0) as u32, target_pixels)
  };

  // Downscale with nearest neighbor (hard pixel edges)
  let small = DynamicImage::ImageRgba8(cropped)
    .resize(small_w, small_h, image::imageops::Nearest);

  // Upscale back with nearest neighbor (keep blocky pixels)
  let big = small.resize(cw, ch, image::imageops::Nearest);

  let mut result = big.to_rgba8();

  // Optional color quantization
  if let Some(colors) = quantize {
    quantize_colors(&mut result, colors);
  }

  result
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

#[cfg(test)]
mod tests {
  use super::*;
  use image::RgbaImage;

  #[test]
  fn test_auto_crop_skips_transparent_edges() {
    let mut img = RgbaImage::new(10, 10);
    // Paint a 4x4 red square in the center
    for y in 3..7 {
      for x in 3..7 {
        img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
      }
    }
    let cropped = auto_crop_to_content(&img);
    assert_eq!(cropped.width(), 4);
    assert_eq!(cropped.height(), 4);
  }

  #[test]
  fn test_compose_spritesheet_2x2() {
    let frames: Vec<DynamicImage> = (0..4)
      .map(|i| {
        let mut img = RgbaImage::new(32, 32);
        let color = match i {
          0 => image::Rgba([255, 0, 0, 255]),
          1 => image::Rgba([0, 255, 0, 255]),
          2 => image::Rgba([0, 0, 255, 255]),
          _ => image::Rgba([255, 255, 0, 255]),
        };
        for y in 0..32 { for x in 0..32 { img.put_pixel(x, y, color); } }
        DynamicImage::ImageRgba8(img)
      })
      .collect();

    let sheet = compose_spritesheet(&frames, 2).unwrap();
    assert_eq!(sheet.width(), 64);  // 2 * 32
    assert_eq!(sheet.height(), 64); // 2 * 32
    assert_eq!(sheet.get_pixel(0, 0), &image::Rgba([255, 0, 0, 255]));
    assert_eq!(sheet.get_pixel(32, 0), &image::Rgba([0, 255, 0, 255]));
    assert_eq!(sheet.get_pixel(0, 32), &image::Rgba([0, 0, 255, 255]));
    assert_eq!(sheet.get_pixel(32, 32), &image::Rgba([255, 255, 0, 255]));
  }

  #[test]
  fn test_compose_spritesheet_empty_fails() {
    let frames: Vec<DynamicImage> = vec![];
    assert!(compose_spritesheet(&frames, 2).is_err());
  }
}
