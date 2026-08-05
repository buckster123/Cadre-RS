P = params(
    hub_r=12.0,
    hub_h=40.0,
    fin_t=2.0,
    fin_w=18.0,
    fin_h=30.0,
    boss_r=4.0,
    boss_h=10.0,
)

def gen_step():
    hub = cylinder(P.hub_r, P.hub_h, at=(0.0, 0.0, 0.0))
    fins = []
    for i in range(6):
        ang = 360.0 * float(i) / 6.0
        fin = box(P.fin_w, P.fin_t, P.fin_h, at=(P.hub_r + P.fin_w / 2.0, 0.0, P.hub_h / 2.0))
        fin = rotate_z(fin, ang)
        fins.append(fin)
    body = union(hub, union_all(fins))
    boss = cylinder(P.boss_r, P.boss_h, at=(0.0, 0.0, 0.0))
    boss = translate(boss, 0.0, 0.0, -P.boss_h / 2.0)
    boss = rotate(boss, "y", 90.0)
    boss = translate(boss, P.hub_r, 0.0, P.hub_h * 0.75)
    body = union(body, boss)
    return solid(body, label="finned_cylinder")
