import { drawFrame, getFrameMetrics, isGifSprite } from './SpriteLoader';
import type { AnimationDef, GifFrameData } from './types';

export function render(
  ctx: CanvasRenderingContext2D,
  sprite: HTMLImageElement | GifFrameData,
  frameIndex: number,
  animDef: AnimationDef | undefined,
) {
  ctx.imageSmoothingEnabled = false;

  if (isGifSprite(sprite)) {
    const frameCount = animDef?.frameCount ?? sprite.frames.length;
    if (frameIndex >= frameCount) return;
    const bitmap = sprite.frames[frameIndex];
    const fitScale = Math.min(ctx.canvas.width / bitmap.width, ctx.canvas.height / bitmap.height);
    const drawWidth = Math.round(bitmap.width * fitScale);
    const drawHeight = Math.round(bitmap.height * fitScale);
    const cx = Math.round((ctx.canvas.width - drawWidth) / 2);
    const cy = Math.round((ctx.canvas.height - drawHeight) / 2);
    ctx.drawImage(bitmap, cx, cy, drawWidth, drawHeight);
  } else {
    const frameCount = animDef?.frameCount ?? 4;
    const framesPerRow = animDef?.framesPerRow ?? 2;
    const { frameWidth, frameHeight } = getFrameMetrics(sprite, frameCount, framesPerRow);
    const fitScale = Math.min(ctx.canvas.width / frameWidth, ctx.canvas.height / frameHeight);
    const drawWidth = Math.round(frameWidth * fitScale);
    const drawHeight = Math.round(frameHeight * fitScale);
    const cx = Math.round((ctx.canvas.width - drawWidth) / 2);
    const cy = Math.round((ctx.canvas.height - drawHeight) / 2);
    drawFrame(ctx, sprite, frameIndex, frameWidth, frameHeight, cx, cy, drawWidth, drawHeight, framesPerRow);
  }
}
