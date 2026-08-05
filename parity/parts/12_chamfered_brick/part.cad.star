P = params(
    dx=40.0,
    dy=30.0,
    dz=20.0,
    cham=2.0,
)

def gen_step():
    # OCCT-only parity: chamfered brick (mock Unsupported).
    b = box(P.dx, P.dy, P.dz, at=CENTER)
    b = chamfer(b, distance=P.cham)
    return solid(b, label="chamfered_brick")
