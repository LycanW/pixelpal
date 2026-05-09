#!/usr/bin/env python3
from PIL import Image
import os

FRAME = 32
SHEET = FRAME * 2  # 64

P = {
    '.': (0, 0, 0, 0),
    'X': (26, 26, 46, 255),
    'S': (232, 184, 138, 255),
    'H': (200, 118, 58, 255),
    'h': (160, 88, 40, 255),
    'W': (255, 255, 255, 255),
    'B': (17, 17, 17, 255),
    'T': (74, 144, 217, 255),
    't': (42, 106, 154, 255),
    'P': (74, 122, 154, 255),
    'Q': (106, 58, 26, 255),
    'K': (244, 164, 196, 255),
}

def px(img, x, y, c):
    if c != '.' and 0 <= x < FRAME and 0 <= y < FRAME:
        img.putpixel((x, y), P[c])

def r(img, x, y, w, h, c):
    for dy in range(h):
        for dx in range(w):
            if c != '.':
                px(img, x+dx, y+dy, c)

def o(img, x, y, w, h, fill):
    for dx in range(-1, w+1):
        px(img, x+dx, y-1, 'X')
        px(img, x+dx, y+h, 'X')
    for dy in range(h):
        px(img, x-1, y+dy, 'X')
        px(img, x+w, y+dy, 'X')
    r(img, x, y, w, h, fill)

def make_frame():
    return Image.new('RGBA', (FRAME, FRAME), (0,0,0,0))

def compose_grid(frames):
    sheet = Image.new('RGBA', (SHEET, SHEET), (0,0,0,0))
    positions = [(0, 0), (FRAME, 0), (0, FRAME), (FRAME, FRAME)]
    for i, (fx, fy) in enumerate(positions):
        sheet.paste(frames[i], (fx, fy))
    return sheet

# ============================================================
# Character parts
# ============================================================

def draw_char(img, dx=0, dy=0, eyes='open', blink_override=False):
    # Hair
    o(img, 7+dx, 2+dy, 18, 2, 'H')
    r(img, 7+dx, 3+dy, 1, 11, 'H')
    r(img, 24+dx, 3+dy, 1, 11, 'H')

    # Head outline + skin
    o(img, 8+dx, 3+dy, 16, 13, 'S')

    # Eyes
    if eyes == 'closed' or blink_override:
        r(img, 11+dx, 7+dy, 3, 1, 'X')
        r(img, 18+dx, 7+dy, 3, 1, 'X')
    else:
        r(img, 11+dx, 7+dy, 3, 3, 'W')
        r(img, 18+dx, 7+dy, 3, 3, 'W')
        r(img, 12+dx, 8+dy, 1, 1, 'B')
        r(img, 19+dx, 8+dy, 1, 1, 'B')

    # Blush
    r(img, 8+dx, 9+dy, 2, 1, 'K')
    r(img, 22+dx, 9+dy, 2, 1, 'K')

    # Body
    o(img, 8+dx, 15+dy, 16, 9, 'T')
    r(img, 10+dx, 17+dy, 12, 4, 't')

    # Legs
    o(img, 9+dx, 24+dy, 6, 5, 'P')
    o(img, 17+dx, 24+dy, 6, 5, 'P')

    # Feet
    o(img, 9+dx, 29+dy, 6, 2, 'Q')
    o(img, 17+dx, 29+dy, 6, 2, 'Q')

def draw_sleep(img):
    # Character lying on side
    # Head at left
    o(img, 4, 14, 14, 11, 'S')
    r(img, 3, 13, 16, 2, 'H')
    r(img, 3, 15, 1, 9, 'H')

    # Eyes closed
    r(img, 7, 18, 3, 1, 'X')
    r(img, 12, 18, 3, 1, 'X')

    # Body to the right
    o(img, 16, 16, 14, 8, 'T')
    r(img, 18, 18, 10, 4, 't')

    # Legs at right
    o(img, 28, 19, 5, 4, 'P')
    o(img, 28, 23, 5, 2, 'Q')

    # Arms folded
    r(img, 16, 21, 4, 2, 'S')

# ============================================================
# Animation frames
# ============================================================

def build_idle():
    f = []
    for i in range(4):
        img = make_frame()
        d = 0
        if i in (1, 3):
            d = -1 if i == 1 else 1
        blink = (i == 3)
        draw_char(img, dy=d, blink_override=blink)
        f.append(img)
    return f

def build_walk():
    f = []
    # Frames: legs alternating, body bouncing
    configs = [
        (0, 0, 0, 0, 0),     # neutral
        (-1, -1, 1, -2, -1),  # left forward, right back, body up
        (0, 0, 0, 0, 0),     # crossing
        (1, -1, -1, -2, -1),  # right forward, left back, body up
    ]
    for ll, lr, rl, rr, by in configs:
        img = make_frame()
        draw_char(img, dy=by)
        # Override legs for walk
        o(img, 9+ll, 24+by+lr, 6, 5, 'P')
        o(img, 17+rl, 24+by+rr, 6, 5, 'P')
        o(img, 9+ll, 29+by+lr, 6, 2, 'Q')
        o(img, 17+rl, 29+by+rr, 6, 2, 'Q')
        f.append(img)
    return f

def build_click():
    f = []
    for i in range(4):
        img = make_frame()
        d = -2 if i < 2 else 0
        surprise = (i == 1 or i == 2)
        if surprise:
            # Bigger eyes
            r(img, 10, 7, 4, 4, 'W')
            r(img, 18, 7, 4, 4, 'W')
            draw_char(img, dy=d, eyes='open')
            r(img, 11, 8, 2, 2, 'B')
            r(img, 19, 8, 2, 2, 'B')
        else:
            draw_char(img, dy=d)
        f.append(img)
    return f

def build_sleep():
    f = []
    for i in range(4):
        img = make_frame()
        br = -1 if i in (1, 3) else 0
        draw_sleep(img)
        # zzz
        if i > 0:
            from PIL import ImageDraw
            d = ImageDraw.Draw(img)
            d.text((18 + i*3, 8 - i*4), 'z' * i, fill=P['B'])
        f.append(img)
    return f

# ============================================================
# Main
# ============================================================

OUT = os.path.join(os.path.dirname(__file__), '..', 'pets', 'default-cat')
os.makedirs(OUT, exist_ok=True)

actions = {
    'idle': build_idle,
    'walk': build_walk,
    'click': build_click,
    'sleep': build_sleep,
}

for name, builder in actions.items():
    frames = builder()
    sheet = compose_grid(frames)
    path = os.path.join(OUT, f'{name}.png')
    sheet.save(path)
    print(f'  ✓ {name}.png  ({os.path.getsize(path)} bytes)')

print(f'\nDone → {OUT}/')
