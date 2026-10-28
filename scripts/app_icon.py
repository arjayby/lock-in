# 1024px app icon: blue rounded square with a white padlock, following the macOS icon grid.
import sys, zlib, struct, math
S, SS = 1024, 2
N = S * SS
def lock(x, y):
    bx0, bx1, by0, by1, r = 0.18, 0.82, 0.44, 0.92, 0.10
    cx = min(max(x, bx0 + r), bx1 - r); cy = min(max(y, by0 + r), by1 - r)
    body = bx0 <= x <= bx1 and by0 <= y <= by1 and (x - cx) ** 2 + (y - cy) ** 2 <= r * r
    hole = (x - 0.5) ** 2 + (y - 0.63) ** 2 <= 0.065 ** 2 or (abs(x - 0.5) <= 0.03 and 0.63 <= y <= 0.78)
    d = math.hypot(x - 0.5, y - 0.30)
    ring = 0.17 <= d <= 0.25 and y <= 0.30
    legs = (0.25 <= x <= 0.33 or 0.67 <= x <= 0.75) and 0.30 <= y <= 0.46
    return (body and not hole) or ring or legs
def pixel(x, y):
    # 824px tile centered in 1024 with 185px corners
    m, r = 100 / 1024, 185 / 1024
    if not (m <= x <= 1 - m and m <= y <= 1 - m): return None
    cx = min(max(x, m + r), 1 - m - r); cy = min(max(y, m + r), 1 - m - r)
    if (x - cx) ** 2 + (y - cy) ** 2 > r * r: return None
    lx, ly = (x - 0.25) / 0.5, (y - 0.24) / 0.5
    if 0 <= lx <= 1 and 0 <= ly <= 1 and lock(lx, ly): return (255, 255, 255)
    t = (y - m) / (1 - 2 * m)  # vertical gradient
    return (round(66 - 20 * t), round(133 - 40 * t), round(244 - 30 * t))
rows = []
for py in range(S):
    row = bytearray([0])
    for px in range(S):
        acc = [0, 0, 0]; a = 0
        for i in range(SS):
            for j in range(SS):
                c = pixel((px * SS + i + 0.5) / N, (py * SS + j + 0.5) / N)
                if c: a += 1; acc = [acc[k] + c[k] for k in range(3)]
        row += bytes([*(round(v / a) if a else 0 for v in acc), round(255 * a / (SS * SS))])
    rows.append(bytes(row))
def chunk(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
open(sys.argv[1], "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b""))
