P = params(
    tread_w=40.0,
    tread_d=18.0,
    tread_t=3.0,
    rise=8.0,
    post_r=5.0,
    rail_r=2.0,
)

def gen_step():
    post_h = P.rise * 8.0 + P.tread_t
    post = cylinder(P.post_r, post_h, at=(0.0, 0.0, 0.0))
    treads = []
    rails = []
    for i in range(8):
        z = float(i) * P.rise
        ang = float(i) * (360.0 / 8.0)
        t = box(P.tread_w, P.tread_d, P.tread_t, at=(P.tread_w / 2.0 + P.post_r, 0.0, z + P.tread_t / 2.0))
        t = rotate_z(t, ang)
        treads.append(t)
        r = cylinder(P.rail_r, P.rise + 2.0, at=(P.post_r + P.tread_w - 4.0, 0.0, z))
        r = rotate_z(r, ang)
        rails.append(r)
    body = union(post, union_all(treads))
    body = union(body, union_all(rails))
    return solid(body, label="spiral_stair")
