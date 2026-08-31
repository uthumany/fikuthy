import os
import sys
import pyte
from PIL import Image, ImageDraw, ImageFont

COLORS = {
    'default': (232, 237, 243), 'black': (10, 13, 18), 'red': (255, 93, 115),
    'green': (77, 219, 138), 'yellow': (245, 197, 66), 'blue': (73, 215, 255),
    'magenta': (169, 112, 255), 'cyan': (65, 217, 255), 'white': (232, 237, 243),
    'darkgray': (129, 144, 163), 'lightgray': (232, 237, 243)
}

def color(value, default):
    text = str(value).lower()
    if text == 'default': return default
    if len(text) == 6:
        try: return tuple(int(text[i:i+2], 16) for i in (0, 2, 4))
        except ValueError: pass
    return COLORS.get(text, default)

def render(path):
    name = os.path.basename(path)
    stem = os.path.splitext(name)[0]
    cols, rows = [int(value) for value in stem.rsplit('-', 1)[1].split('x')]
    screen = pyte.Screen(cols, rows)
    pyte.Stream(screen).feed(open(path, 'rb').read().decode('utf-8', errors='replace'))
    font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf', 14)
    cw, ch = 9, 19
    image = Image.new('RGB', (cols * cw, rows * ch), (10, 13, 18))
    draw = ImageDraw.Draw(image)
    for y in range(rows):
        for x, cell in screen.buffer[y].items():
            bg = color(cell.bg, (10, 13, 18))
            fg = color(cell.fg, (232, 237, 243))
            draw.rectangle((x*cw, y*ch, (x+1)*cw, (y+1)*ch), fill=bg)
            if cell.data != ' ':
                draw.text((x*cw, y*ch-1), cell.data, font=font, fill=fg)
    out = os.path.join(os.path.dirname(path), stem + '.png')
    image.save(out)
    print(out)

for path in sys.argv[1:]: render(path)
