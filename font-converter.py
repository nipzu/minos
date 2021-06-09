from PIL import Image

input_file = input("input file: ")
output_file = input("output file: ")
font_width = int(input("font width: "))
font_height = int(input("font height: "))

with Image.open(input_file) as font_in:
  with open(output_file,"w") as font_out:
    b = 0
    i = 0
    font_out.write("[")
    for i in range(font_width * font_height * 96):
      if font_in.getpixel((i % (font_width * 16), i // (font_width * 16))) == 1:
        b += 2**(i % 8)
      if (i + 1) % 8 == 0:
        font_out.write(str(b) + ",")
        b = 0
    font_out.write("]")
