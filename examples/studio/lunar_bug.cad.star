P = params(
    body_l=48.0,
    body_w=28.0,
    body_h=16.0,
    wall=3.0,
    leg_l=10.0,
    leg_w=6.0,
    leg_h=14.0,
    dome_r=9.0,
    eye_r=2.2,
    eye_z=18.0,
    antenna_h=12.0,
    antenna_r=1.2,
    n_legs=6.0,
)

def gen_step():
    # Lunar rover bug — hollow-ish chassis, polar legs, dome head, twin antennae
    chassis = box(P.body_l, P.body_w, P.body_h, at=(0.0, 0.0, P.leg_h + P.body_h / 2.0))
    cavity = box(
        P.body_l - 2.0 * P.wall,
        P.body_w - 2.0 * P.wall,
        P.body_h - P.wall,
        at=(0.0, 0.0, P.leg_h + P.body_h / 2.0 + P.wall * 0.25),
    )
    shell = cut(chassis, cavity)

    leg = box(P.leg_l, P.leg_w, P.leg_h, at=(P.body_l * 0.35, 0.0, P.leg_h / 2.0))
    legs = polar_pattern(leg, P.n_legs)

    dome = sphere(P.dome_r, at=(0.0, 0.0, P.leg_h + P.body_h + P.dome_r * 0.35))
    eye_l = sphere(P.eye_r, at=(-4.0, 5.5, P.leg_h + P.body_h + 6.0))
    eye_r = sphere(P.eye_r, at=(4.0, 5.5, P.leg_h + P.body_h + 6.0))
    eyes = union(eye_l, eye_r)

    ant_l = cylinder(P.antenna_r, P.antenna_h, at=(-5.0, -2.0, P.leg_h + P.body_h + 2.0))
    ant_r = cylinder(P.antenna_r, P.antenna_h, at=(5.0, -2.0, P.leg_h + P.body_h + 2.0))
    tip_l = sphere(2.0, at=(-5.0, -2.0, P.leg_h + P.body_h + P.antenna_h + 1.5))
    tip_r = sphere(2.0, at=(5.0, -2.0, P.leg_h + P.body_h + P.antenna_h + 1.5))
    ants = union(union(ant_l, ant_r), union(tip_l, tip_r))

    # little cargo crate on the back
    crate = box(10.0, 8.0, 6.0, at=(0.0, -P.body_w * 0.15, P.leg_h + P.body_h + 3.0))

    body = union(union(shell, legs), union(union(dome, eyes), union(ants, crate)))
    return solid(body, label="lunar_bug")
