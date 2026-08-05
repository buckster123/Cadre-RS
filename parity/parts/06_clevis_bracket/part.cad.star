P = params(
    base_l=50.0,
    base_w=20.0,
    base_t=5.0,
    ear_h=30.0,
    ear_t=5.0,
    ear_gap=12.0,
    pin_d=6.0,
    light_d=8.0,
)

def gen_step():
    base = box(P.base_l, P.base_w, P.base_t, at=(P.base_l / 2.0, 0.0, P.base_t / 2.0))
    y0 = P.ear_gap / 2.0 + P.ear_t / 2.0
    ear_a = box(P.ear_t, P.base_w, P.ear_h, at=(P.base_l - P.ear_t / 2.0, y0, P.base_t + P.ear_h / 2.0))
    ear_b = box(P.ear_t, P.base_w, P.ear_h, at=(P.base_l - P.ear_t / 2.0, -y0, P.base_t + P.ear_h / 2.0))
    body = union(union(base, ear_a), ear_b)
    pin_z = P.base_t + P.ear_h * 0.7
    pin = box(P.ear_t + 2.0, P.pin_d, P.pin_d, at=(P.base_l - P.ear_t / 2.0, y0, pin_z))
    pin2 = box(P.ear_t + 2.0, P.pin_d, P.pin_d, at=(P.base_l - P.ear_t / 2.0, -y0, pin_z))
    body = cut(body, union(pin, pin2))
    for x in (12.0, 28.0):
        h = cylinder(P.light_d / 2.0, P.base_t + 2.0, at=(x, 0.0, -1.0))
        body = cut(body, h)
    return solid(body, label="clevis_bracket")
