P = params(
    width=80.0,
    depth=50.0,
    height=16.0,
    hole_d=10.0,
    fillet_r=2.0,
)

def gen_step():
    # OCCT-only parity: plate + center hole + all-edge fillet (mock Unsupported).
    blk = box(P.width, P.depth, P.height, at=CENTER)
    hole = cylinder(P.hole_d / 2.0, P.height + 2.0, at=(0.0, 0.0, -1.0))
    body = cut(blk, hole)
    body = fillet(body, radius=P.fillet_r)
    return solid(body, label="filleted_plate")
