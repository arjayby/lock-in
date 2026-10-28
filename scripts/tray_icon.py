# Draws a padlock as a black template PNG (alpha only), supersampled for smooth edges.
import sys, zlib, struct, math
S = int(sys.argv[2]); SS = 4; N = S * SS
def inside(x, y):
    # coordinates in 0..1
    # body: rounded rect
    bx0, bx1, by0, by1, r = 0.18, 0.82, 0.44, 0.92, 0.10
    cx = min(max(x, bx0 + r), bx1 - r); cy = min(max(y, by0 + r), by1 - r)
    body = bx0 <= x <= bx1 and by0 <= y <= by1 and (x - cx) ** 2 + (y - cy) ** 2 <= r * r
    # keyhole cut out of the body
    hole = (x - 0.5) ** 2 + (y - 0.63) ** 2 <= 0.065 ** 2 or (abs(x - 0.5) <= 0.03 and 0.63 <= y <= 0.78)
    # shackle: half ring + legs
    d = math.hypot(x - 0.5, y - 0.30)
    ring = 0.17 <= d <= 0.25 and y <= 0.30
    legs = (0.25 <= x <= 0.33 or 0.67 <= x <= 0.75) and 0.30 <= y <= 0.46
    return (body and not hole) or ring or legs
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
