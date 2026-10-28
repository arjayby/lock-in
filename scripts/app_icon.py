# 1024px app icon: blue rounded square with a white bullseye, following the macOS icon grid.
import sys, zlib, struct, math
S, SS = 1024, 2
N = S * SS
def target(x, y):
    # bullseye from shapes.gallery: rings at 96..128 and 40..72 plus a 16 dot, on a 128 radius
    d = math.hypot(x - 0.5, y - 0.5) * 256
    return 96 <= d <= 128 or 40 <= d <= 72 or d <= 16
def pixel(x, y):
    # 824px tile centered in 1024 with 185px corners
    m, r = 100 / 1024, 185 / 1024
    if not (m <= x <= 1 - m and m <= y <= 1 - m): return None
    cx = min(max(x, m + r), 1 - m - r); cy = min(max(y, m + r), 1 - m - r)
    if (x - cx) ** 2 + (y - cy) ** 2 > r * r: return None
    lx, ly = (x - 0.27) / 0.46, (y - 0.27) / 0.46
    if 0 <= lx <= 1 and 0 <= ly <= 1 and target(lx, ly): return (255, 255, 255)
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
