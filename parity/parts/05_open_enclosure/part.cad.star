P = params(
    outer_w=80.0,
    outer_d=60.0,
    outer_h=40.0,
    wall=3.0,
    boss_d=8.0,
    boss_h=6.0,
)

def gen_step():
    outer = box(P.outer_w, P.outer_d, P.outer_h, at=(0.0, 0.0, P.outer_h / 2.0))
    iw = P.outer_w - 2.0 * P.wall
    id_ = P.outer_d - 2.0 * P.wall
    ih = P.outer_h - P.wall
    cavity = box(iw, id_, ih + 1.0, at=(0.0, 0.0, P.wall + (ih + 1.0) / 2.0))
    shell = cut(outer, cavity)
    bosses = [
        cylinder(P.boss_d / 2.0, P.boss_h, at=(x, y, P.wall))
        for x in (-20.0, 20.0)
        for y in (-15.0, 15.0)
    ]
    body = union(shell, union_all(bosses))
    return solid(body, label="open_enclosure")
