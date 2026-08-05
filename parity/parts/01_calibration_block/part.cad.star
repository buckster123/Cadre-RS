P = params(
    width=100.0,
    depth=60.0,
    height=20.0,
    hole_d=8.0,
)

def gen_step():
    # Calibration block: plate with 2×2 hole pattern (M1 part 1, mock-safe — no chamfer).
    blk = box(P.width, P.depth, P.height, at=CENTER)
    holes = [
        cylinder(P.hole_d / 2.0, P.height + 2.0, at=(x, y, -1.0))
        for x in (-30.0, 30.0)
        for y in (-15.0, 15.0)
    ]
    body = cut(blk, union_all(holes))
    return solid(body, label="calibration_block")
