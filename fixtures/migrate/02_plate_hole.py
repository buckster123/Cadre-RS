"""Plate with hole — public-API-shaped (clean-room)."""
from build123d import *

length = 80.0
width = 50.0
height = 8.0
hole_r = 4.0

with BuildPart() as plate:
    Box(length, width, height)
    with Locations((0, 0, 0)):
        Cylinder(hole_r, height)
    # subtract-ish intent via mode would live here in real build123d
