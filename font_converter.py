from PIL import Image

with open("font_data.txt","w") as font_out:
  font_in = Image.open("font.png")
  b = 0
  i = 0
  font_out.write("[")
  for i in range(48 * 96):
    if font_in.getpixel((i % 96, i // 96)) == 1:
      b += 2**(i % 8)
    if (i + 1) % 8 == 0:
      font_out.write(str(b) + ",")
      b = 0
  font_out.write("]")