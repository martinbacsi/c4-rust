# Configuration class to hold settings
import sys
class Config:
    def __init__(self, self_play, iterations, cpuct):
        self.self_play = self_play
        self.iterations = iterations
        self.cpuct = cpuct

CONF = Config(self_play=True, iterations=1000, cpuct=4.0)
DIRICHLET_EPS = 0.3
W = 9
H = 7
POLICY_SIZE = W
ACTION_SIZE=POLICY_SIZE
INPUT_SIZE = H * W * 2


WIDTH = 9
HEIGHT = 7
FAB_COL = 0b1111111
FAB_ROW = (1 << (7 * 0)) | (1 << (7 * 1)) | (1 << (7 * 2)) | (1 << (7 * 3)) | (1 << (7 * 4)) | (1 << (7 * 5)) | (1 << (7 * 6)) | (1 << (7 * 7)) | (1 << (7 * 8))

COLS = [
    FAB_COL << (7 * 0),
    FAB_COL << (7 * 1),
    FAB_COL << (7 * 2),
    FAB_COL << (7 * 3),
    FAB_COL << (7 * 4),
    FAB_COL << (7 * 5),
    FAB_COL << (7 * 6),
    FAB_COL << (7 * 7),
    FAB_COL << (7 * 8),
]

ROWS = [
    FAB_ROW << 0,
    FAB_ROW << 1,
    FAB_ROW << 2,
    FAB_ROW << 3,
    FAB_ROW << 4,
    FAB_ROW << 5,
    FAB_ROW << 6,
]

D1_MASK = (COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5]) & (ROWS[3] | ROWS[4] | ROWS[5] | ROWS[6])
D2_MASK = (COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5]) & (ROWS[0] | ROWS[1] | ROWS[2] | ROWS[3])
H_MASK = COLS[0] | COLS[1] | COLS[2] | COLS[3] | COLS[4] | COLS[5]
V_MASK = ROWS[0] | ROWS[1] | ROWS[2] | ROWS[3]

def won(bitboard):
    d1 = bitboard & (bitboard >> 6) & (bitboard >> 12) & (bitboard >> 18) & D1_MASK
    d2 = bitboard & (bitboard >> 8) & (bitboard >> 16) & (bitboard >> 24) & D2_MASK
    h = bitboard & (bitboard >> 7) & (bitboard >> 14) & (bitboard >> 21) & H_MASK
    v = bitboard & (bitboard >> 1) & (bitboard >> 2) & (bitboard >> 3) & V_MASK
    return v + h + d1 + d2 > 0

class Outcome:
    Win = "Win"
    Draw = "Draw"
    NULL = "NULL"

class Connect4:
    def __init__(self):
        self.my_bitboard = 0
        self.opponent_bitboard = 0
        self.height = [0] * WIDTH
        self.player = 0
        self.outcome = Outcome.NULL
        self.last_move = 255  # equivalent to u8::max_value()

    def hash(self):
        return hash((self.my_bitboard, self.opponent_bitboard))

    def full(self):
        return self.turn() == (WIDTH * HEIGHT)

    def turn(self):
        return bin(self.my_bitboard | self.opponent_bitboard).count('1')

    def step(self, action):
        self.my_bitboard ^= 1 << (self.height[action] + HEIGHT * action)
        self.height[action] += 1
        self.my_bitboard, self.opponent_bitboard = self.opponent_bitboard, self.my_bitboard
        self.player = 1 - self.player
        self.last_move = action
        if won(self.opponent_bitboard):
            self.outcome = Outcome.Win
        elif self.full():
            self.outcome = Outcome.Draw

    def on_set_indices(self, func):
        maps = [self.my_bitboard, self.opponent_bitboard]

        for i in range(2):
            while maps[i] != 0:
                r = (maps[i] & -maps[i]).bit_length() - 1  # equivalent to trailing_zeros
                maps[i] ^= 1 << r
                nn_index = r * 2 + i
                func(nn_index)


    def print_board(self):
        for row in range(HEIGHT - 1, -1, -1):
            for col in range(WIDTH):
                index = 1 << (row + HEIGHT * col)
                print("O" if self.my_bitboard & index else "X" if self.opponent_bitboard & index else ".", end=" ")
            print()
        print("0 1 2 3 4 5 6 7 8")

class Game(Connect4):
    pass