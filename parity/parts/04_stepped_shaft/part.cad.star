P = params(
    d1=20.0,
    h1=30.0,
    d2=15.0,
    h2=20.0,
    d3=10.0,
    h3=15.0,
    key_w=4.0,
    key_d=2.5,
    key_len=18.0,
)

def gen_step():
    # Stepped shaft: three coaxial cylinders + rectangular keyway on largest step.
    s1 = cylinder(P.d1 / 2.0, P.h1, at=(0.0, 0.0, 0.0))
    s2 = cylinder(P.d2 / 2.0, P.h2, at=(0.0, 0.0, P.h1))
    s3 = cylinder(P.d3 / 2.0, P.h3, at=(0.0, 0.0, P.h1 + P.h2))
    body = union(union(s1, s2), s3)
    # Keyway: axial slot cut into outer diameter of step 1
    key = box(
        P.key_len,
        P.key_w,
        P.key_d + 1.0,
        at=(0.0, P.d1 / 2.0 - P.key_d / 2.0, P.h1 / 2.0),
    )
    body = cut(body, key)
    return solid(body, label="stepped_shaft")
