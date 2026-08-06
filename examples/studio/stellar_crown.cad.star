P = params(
    hub_r=14.0,
    hub_h=10.0,
    ring_r=28.0,
    ring_t=4.0,
    ring_h=6.0,
    spike_l=16.0,
    spike_w=3.5,
    spike_h=18.0,
    n_spikes=8.0,
    gem_r=7.0,
    base_r=18.0,
    base_h=4.0,
)

def gen_step():
    # Stellar crown — polar spikes, gem sphere, cut through-hole, base pedestal
    base = cylinder(P.base_r, P.base_h, at=(0.0, 0.0, 0.0))
    hub = cylinder(P.hub_r, P.hub_h, at=(0.0, 0.0, P.base_h))
    ring = cylinder(P.ring_r, P.ring_h, at=(0.0, 0.0, P.base_h + 2.0))
    ring_core = cylinder(P.ring_r - P.ring_t, P.ring_h + 0.2, at=(0.0, 0.0, P.base_h + 1.9))
    ring = cut(ring, ring_core)
    spike = box(
        P.spike_l,
        P.spike_w,
        P.spike_h,
        at=(P.hub_r + P.spike_l / 2.0, 0.0, P.base_h + P.hub_h / 2.0 + 4.0),
    )
    spikes = polar_pattern(spike, P.n_spikes)
    gem = sphere(P.gem_r, at=(0.0, 0.0, P.base_h + P.hub_h + P.gem_r * 0.55))
    bore = cylinder(4.0, P.base_h + P.hub_h + 2.0, at=(0.0, 0.0, -0.5))
    body = union(union(union(base, hub), union(ring, spikes)), gem)
    body = cut(body, bore)
    return solid(body, label="stellar_crown")
