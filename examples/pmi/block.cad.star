P = params(w=100.0, d=60.0, h=20.0)

def gen_step():
    return solid(box(P.w, P.d, P.h, at=CENTER), label="pmi_block")
