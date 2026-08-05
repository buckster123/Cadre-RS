P = params(w=40.0, d=40.0, h=5.0)
def gen_step():
    return solid(box(P.w, P.d, P.h, at=CENTER), label="plate")
