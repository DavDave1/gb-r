## PPU State machine

1 dot == 1 CPU cycle

154 scan lines

each scan line is 456 dots

Initial state
dot count = 0
LY = 0
Mode2

Next cylce

update dot count
if LY > 144
  set VBLANK
  if LY == 153 && DOT >=456
     render
     LY = 0
     DOT = 456 - DOT
else if DOT <= 80
   Mode2
else if DOT  < 289 && !DRAWING DONE
   Mode3
else if DOT <= 456
   Mode0
else
   LY++
   DOT = 456 - DOT

Mode2:
  Read tiles
Mode3:
  Draw pixels
  set drawing done
Mode0
  set HBLANK


