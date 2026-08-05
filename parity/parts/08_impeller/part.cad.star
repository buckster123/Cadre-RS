P = params(
    hub_r=8.0,
    hub_h=12.0,
    blade_l=25.0,
    blade_t=2.0,
    blade_h=10.0,
    shroud_t=2.0,
)

def gen_step():
    hub = cylinder(P.hub_r, P.hub_h, at=(0.0, 0.0, 0.0))
    blades = []
    for i in range(8):
        ang = 360.0 * float(i) / 8.0
        b = box(P.blade_l, P.blade_t, P.blade_h, at=(P.hub_r + P.blade_l / 2.0, 0.0, P.hub_h / 2.0))
        b = rotate_z(b, ang)
        blades.append(b)
    body = union(hub, union_all(blades))
    outer_r = P.hub_r + P.blade_l
    shroud = cylinder(outer_r, P.shroud_t, at=(0.0, 0.0, P.hub_h))
    body = union(body, shroud)
    return solid(body, label="impeller")
