import { invoke } from '@tauri-apps/api/core';
import { parseGIF, decompressFrames } from 'gifuct-js';
import { FRAMES_PER_ACTION } from './config';
import type { GifFrameData } from './types';

export async function loadAnimation(petId: string, filename: string): Promise<HTMLImageElement> {
  const dataUrl = await invoke<string>('read_pet_sprite', { id: petId, filename });
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`failed to load sprite: ${petId}/${filename}`));
    img.src = dataUrl;
  });
}

export function getFrameMetrics(
  img: HTMLImageElement,
  frameCount: number = FRAMES_PER_ACTION,
  framesPerRow: number = 2,
) {
  const columns = Math.max(1, framesPerRow);
  const rows = Math.max(1, Math.ceil(frameCount / columns));
  return {
    frameWidth: img.naturalWidth / columns,
    frameHeight: img.naturalHeight / rows,
  };
}

export function isGifSprite(data: HTMLImageElement | GifFrameData): data is GifFrameData {
  return 'frames' in data && Array.isArray((data as GifFrameData).frames);
}

export async function loadGifAnimation(petId: string, filename: string): Promise<GifFrameData> {
  const dataUrl = await invoke<string>('read_pet_sprite', { id: petId, filename });
  const response = await fetch(dataUrl);
  const buffer = await response.arrayBuffer();
  const gif = parseGIF(buffer);
  const frames = decompressFrames(gif, true);

  const imageBitmaps: ImageBitmap[] = [];
  const delays: number[] = [];

  for (const frame of frames) {
    const imageData = new ImageData(
      new Uint8ClampedArray(frame.patch),
      frame.dims.width,
      frame.dims.height,
    );
    const bitmap = await createImageBitmap(imageData);
    imageBitmaps.push(bitmap);
    delays.push(frame.delay || 100);
  }

  return { frames: imageBitmaps, delays };
}

export function drawFrame(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  frameIndex: number,
  sourceFrameWidth: number,
  sourceFrameHeight: number,
  dx: number,
  dy: number,
  drawWidth: number,
  drawHeight: number,
  framesPerRow: number = 2,
) {
  const sx = (frameIndex % framesPerRow) * sourceFrameWidth;
  const sy = Math.floor(frameIndex / framesPerRow) * sourceFrameHeight;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    img,
    sx, sy, sourceFrameWidth, sourceFrameHeight,
    dx, dy, drawWidth, drawHeight,
  );
}
