# Draws a bullseye as a black template PNG (alpha only), supersampled for smooth edges.
import sys, zlib, struct, math
S = int(sys.argv[2]); SS = 4; N = S * SS
def inside(x, y):
    # bullseye from shapes.gallery, scaled to leave a 4% margin
    d = math.hypot(x - 0.5, y - 0.5) / 0.46 * 128
    return 96 <= d <= 128 or 40 <= d <= 72 or d <= 16
rows = []
for py in range(S):
    row = bytearray([0])
    for px in range(S):
        n = sum(inside((px * SS + i + 0.5) / N, (py * SS + j + 0.5) / N) for i in range(SS) for j in range(SS))
        row += bytes([0, 0, 0, round(255 * n / (SS * SS))])
    rows.append(bytes(row))
def chunk(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b"")
open(sys.argv[1], "wb").write(png)
