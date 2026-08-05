P = params(
    outer_d=80.0,
    thickness=10.0,
    bore_d=25.0,
    bolt_circle_d=55.0,
    bolt_d=6.5,
    n_bolts=6,
)

def gen_step():
    # Bolt-circle flange: disc + center bore + N bolt holes on a circle.
    r_out = P.outer_d / 2.0
    disc = cylinder(r_out, P.thickness, at=(0.0, 0.0, 0.0))
    bore = cylinder(P.bore_d / 2.0, P.thickness + 2.0, at=(0.0, 0.0, -1.0))
    body = cut(disc, bore)
    r_bc = P.bolt_circle_d / 2.0
    # 6 bolts at 0,60,...,300 deg — explicit coords (no math module yet)
    bolts = [
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * 1.0, 0.0, -1.0)),
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * 0.5, r_bc * 0.86602540378, -1.0)),
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * -0.5, r_bc * 0.86602540378, -1.0)),
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * -1.0, 0.0, -1.0)),
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * -0.5, r_bc * -0.86602540378, -1.0)),
        cylinder(P.bolt_d / 2.0, P.thickness + 2.0, at=(r_bc * 0.5, r_bc * -0.86602540378, -1.0)),
    ]
    body = cut(body, union_all(bolts))
    return solid(body, label="bolt_circle_flange")
