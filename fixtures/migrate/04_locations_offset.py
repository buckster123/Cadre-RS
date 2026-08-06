"""H2-7 fixture: Locations offset → translate (clean-room public API shape)."""
from build123d import *

with BuildPart() as part:
    Box(40, 20, 8)
    with Locations((25, 0, 0)):
        Cylinder(4, 12)
