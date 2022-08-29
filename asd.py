def dothing(w, h):

    def get_value(x, y):
        return x + w * y + 1

    x, y, d = 0, 0, 1

    for _ in range(w * h):
        yield get_value(x, y)
        if d == 1 and x == w - 1:
            y += 1
            d = -1
        elif d == 1 and y == 0:
            x += 1
            d = -1
        elif d == -1 and y == h - 1:
            x += 1
            d = 1
        elif d == -1 and x == 0:
            y += 1
            d = 1
        else:
            x += d
            y -= d

print(list(dothing(5, 4)))