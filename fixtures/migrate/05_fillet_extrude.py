"""H2-7 fixture: Rectangle+extrude + fillet note (clean-room public API shape)."""
from build123d import *

length = 50.0
width = 30.0
height = 10.0
fillet_r = 2.0

with BuildSketch() as sk:
    Rectangle(length, width)

with BuildPart() as part:
    extrude(amount=height)
    fillet(fillet_r)
