import sys, json

def move(command):
    """ print a command record to stdout """
    moves = []
    if command is not None:
        moves.append(command)

    print(json.dumps({ 'moves': moves }))
    # flush the buffer, so that the gameserver can receive the line
    sys.stdout.flush()


for line in sys.stdin:
    state = json.loads(line)

    # you are always player 1.
    my_planets = [p for p in state['planets'] if p['owner'] == 1]
    other_planets = [p for p in state['planets'] if p['owner'] != 1]

    if not my_planets or not other_planets:
        # no valid moves can be made
        move(None)
    else:
        # send some ships!
        move({
            'origin': my_planets[0]['name'],
            'destination': other_planets[0]['name'],
            'ship_count': 1
        })
