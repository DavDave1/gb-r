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

## PPU Rendering

When PPU enters mode 3, it starts drawing pixels using FIFO and Pixel fetcher

Pixel fetcher pushes pixels in the FIFOs: 1 FIFO for BG/Win and 1 FIFO for OBJs

When FIFO is filled with 8 pixels, the pixel drawing happens: FIFOs are mixed based on
priority and color is retrieved using palettes.

### Mode 3

### Get tile index

For background: check LCD Control reg BG tile map area flag (3)
- If LCDControl.3 is true and X coord of scanline is <> viewport X cood
   - Read tilemap from $9C00
- Else
   - Read tilemap from $9800
- Compute tile X,Y coord as:
   - FetcherX = (ScrollX / 8 + FetcherX) &1F
   - FetcherY = (LY + ScrollY) & 0xFF
     - FetcherX is from 0 to 31 => 1 FetcherX == 8 ScanLineX
     - FetcherY is from 0 to 255
   
For window: check LCD Control reg Win tile map area flag (6)
- If LCDControl.6 is true and X coord of scanline is >< viewport X cood
   - Read tilemap from $9C00
- Else
   - Read tilemap from $9800

### Get tile data low

Get lower byte data for tile at index.

### Get tile data high

Get higher byte data for tile at index

This also pushes row of 8 BG/Win pixels to the FIFO

### Push

Push a row of 8 BW/Win pixels to the FIFO, only if FIFO is empty

- If Horizontal Flip == true
  - Push LSB of pixels first
- Else
  - Push MSB first


