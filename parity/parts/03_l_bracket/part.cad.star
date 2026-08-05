P = params(
    leg=40.0,
    width=30.0,
    thick=5.0,
    hole_d=6.0,
    hole_off=15.0,
)

def gen_step():
    # Gusseted L-bracket: horizontal + vertical plates + triangular gusset, holes in two dirs.
    horiz = box(P.leg, P.width, P.thick, at=(P.leg / 2.0, 0.0, P.thick / 2.0))
    vert = box(P.thick, P.width, P.leg, at=(P.thick / 2.0, 0.0, P.leg / 2.0))
    # Simple block gusset in the corner (true triangle needs loft — later)
    gusset = box(P.leg * 0.4, P.thick, P.leg * 0.4, at=(P.leg * 0.2 + P.thick / 2.0, 0.0, P.leg * 0.2 + P.thick / 2.0))
    body = union(union(horiz, vert), gusset)
    # Holes through horizontal plate (+Z)
    h1 = cylinder(P.hole_d / 2.0, P.thick + 2.0, at=(P.hole_off, 0.0, -1.0))
    h2 = cylinder(P.hole_d / 2.0, P.thick + 2.0, at=(P.leg - P.hole_off, 0.0, -1.0))
    body = cut(body, union(h1, h2))
    # Clearance through vertical plate (+X direction) via slot cut
    slot = box(P.thick + 2.0, P.hole_d, P.hole_d, at=(P.thick / 2.0, 0.0, P.leg - P.hole_off))
    body = cut(body, slot)
    return solid(body, label="l_bracket")
