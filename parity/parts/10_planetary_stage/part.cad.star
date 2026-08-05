P = params(
    sun_r=10.0,
    sun_h=8.0,
    planet_r=6.0,
    planet_h=8.0,
    ring_ro=28.0,
    ring_ri=22.0,
    carrier_t=4.0,
)

def gen_step():
    sun = cylinder(P.sun_r, P.sun_h, at=(0.0, 0.0, 0.0))
    ring_o = cylinder(P.ring_ro, P.sun_h, at=(0.0, 0.0, 0.0))
    ring_i = cylinder(P.ring_ri, P.sun_h + 2.0, at=(0.0, 0.0, -1.0))
    ring = cut(ring_o, ring_i)
    planets = []
    orbit = P.sun_r + P.planet_r + 1.0
    for i in range(3):
        ang = 360.0 * float(i) / 3.0
        p = cylinder(P.planet_r, P.planet_h, at=(orbit, 0.0, 0.0))
        p = rotate_z(p, ang)
        planets.append(p)
    carrier = cylinder(orbit + P.planet_r, P.carrier_t, at=(0.0, 0.0, P.sun_h))
    body = union(union(sun, ring), union_all(planets))
    body = union(body, carrier)
    return solid(body, label="planetary_stage")
