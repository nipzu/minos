with open("E:\\font8bit.bmp","rb") as f:
  data = list(f.read()[0x3e:])
  data.reverse()
  b = 0
  i = 0
  for c in "".join(["".join(["#" if data[96*y+95-x] == 1 else " " for x in range(96)]) for y in range(48)]):
    b += 2**(i % 8) * (0 if c == " " else 1)
    i += 1
    if i % 8 == 0:
      print(b, ",")
      b = 0
