P = params(
    hub_r=8.0,
    hub_h=12.0,
    fin_l=18.0,
    fin_t=2.0,
    fin_h=10.0,
    n_fins=6.0,
    boss_r=5.0,
)

def gen_step():
    # H2 demo: polar_pattern fins + sphere boss + mirrored support pad
    hub = cylinder(P.hub_r, P.hub_h, at=(0.0, 0.0, 0.0))
    fin = box(P.fin_l, P.fin_t, P.fin_h, at=(P.hub_r + P.fin_l / 2.0, 0.0, P.hub_h / 2.0))
    fins = polar_pattern(fin, P.n_fins)
    boss = sphere(P.boss_r, at=(0.0, 0.0, P.hub_h + P.boss_r * 0.5))
    pad = box(12.0, 8.0, 3.0, at=(20.0, 0.0, 1.5))
    pads = union(pad, mirror(pad, "yz"))
    body = union(union(hub, fins), union(boss, pads))
    return solid(body, label="pattern_hub")
