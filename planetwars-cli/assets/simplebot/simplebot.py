import sys, json

def move(command):
    """ print a command record to stdout """
    moves = []
    if command is not None:
        moves.append(command)

    print(json.dumps({ 'moves': moves }))
    # flush the buffer, so that the gameserver can receive the json-encoded line.
    sys.stdout.flush()


for line in sys.stdin:
    state = json.loads(line)
    # you are always player 1.
    my_planets = [p for p in state['planets'] if p['owner'] == 1]
    other_planets = [p for p in state['planets'] if p['owner'] != 1]

    if not my_planets or not other_planets:
        # we don't own any planets, so we can't make any moves.
        move(None)
    else:
        # find my planet that has the most ships
        planet = max(my_planets, key=lambda p: p['ship_count'])
        # find enemy planet that has the least ships
        destination = min(other_planets, key=lambda p: p['ship_count'])
        # attack!
        move({
            'origin': planet['name'],
            'destination': destination['name'],
            'ship_count': planet['ship_count'] - 1
        })
