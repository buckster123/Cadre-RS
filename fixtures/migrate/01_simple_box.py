"""Hand-written public-API-shaped fixture (clean-room).

Looks like build123d tutorial style — not copied from any private repo.
"""
from build123d import *

length = 100.0
width = 60.0
height = 20.0

with BuildPart() as part:
    Box(length, width, height)
