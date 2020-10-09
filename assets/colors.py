import cairo

WIDTH, HEIGHT = 256, 32

themes = [
    ("scarlet", [(241, 9, 6), (254, 222, 18), (54, 178, 52), (59, 99, 172)]),
    (
        "blue-and-yellow",
        [(22, 157, 215), (22, 157, 215), (255, 211, 47), (255, 211, 47)],
    ),
    ("hyacinth", [(74, 95, 188), (74, 95, 188), (74, 95, 188), (74, 95, 188)]),
    ("military", [(109, 207, 60), (109, 207, 60), (109, 207, 60), (42, 200, 255)]),
    (
        "gray",
        [(177, 176, 194), (177, 176, 194), (177, 176, 194), (177, 176, 194)],
    ),
    (
        "yellow-crested",
        [(177, 176, 194), (177, 176, 194), (177, 176, 194), (235, 226, 95)],
    ),
]


def from_rgb(rgb):
    r, g, b = rgb
    return r / 255, g / 255, b / 255


for theme in themes:
    name, colors = theme
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, WIDTH, HEIGHT)
    ctx = cairo.Context(surface)
    ctx.scale(WIDTH, HEIGHT)

    ctx.set_source_rgb(*from_rgb(colors[3]))
    ctx.rectangle(0, 0, 1, 1)
    ctx.fill()
    ctx.set_source_rgb(*from_rgb(colors[2]))
    ctx.rectangle(1 / 18, 0, 1 - 2 / 18, 1)
    ctx.fill()
    ctx.set_source_rgb(*from_rgb(colors[1]))
    ctx.rectangle(1 / 9 + 1 / 18, 0, 1 - 2 / 9 - 2 / 18, 1)
    ctx.fill()
    ctx.set_source_rgb(*from_rgb(colors[0]))
    ctx.rectangle(1 / 6 + 1 / 9 + 1 / 18, 0, 2 / 6, 1)
    ctx.fill()

    surface.write_to_png("{}.png".format(name))
