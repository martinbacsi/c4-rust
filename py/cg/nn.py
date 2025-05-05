import numpy as np
from .game import *
from .code_dec import DecodeBase16K
POLICY_SIZE = 10  # You can change this value as necessary
DIRICHLET_EPS = 0.3
def dirichlet_noise(random_generator):
    gamma_shape = 0.3
    gamma_scale = 1.0
    dirichlet_array = np.zeros(POLICY_SIZE)
    
    for i in range(POLICY_SIZE):
        dirichlet_array[i] = np.random.gamma(gamma_shape, gamma_scale)
    
    return dirichlet_array

class NnOutput:
    def __init__(self):
        self.p = np.ones(POLICY_SIZE, dtype=np.float32)
        self.v = 0.0

class DenseLayer:
    def __init__(self, input_size):
        self.input = np.zeros(input_size, dtype=np.float32)
        self.weights = np.zeros((input_size, 0), dtype=np.float32)
        self.bias = np.zeros(0, dtype=np.float32)

    def forward(self, output):
        np.copyto(output, self.bias)
        out_size = output.size
        for j in range(self.input.size):
            val = self.input[j]
            if val != 0.0:
                for i in range(out_size):
                    output[i] += val * self.weights[j, i]

    def forward_game(self, game:Connect4, output):
        np.copyto(output, self.bias)
        out_size = output.size
        def util(nn_ind):
            for j in range(out_size):
                output[j] += self.weights[nn_ind, j]
        game.on_set_indices(util)


class NN:
    def __init__(self):
        self.path = [
            DenseLayer(INPUT_SIZE),
            DenseLayer(128),
            DenseLayer(64),
            DenseLayer(64),
        ]

    def forward(self, game):
        res_raw = np.zeros(POLICY_SIZE + 1, dtype=np.float32)
        res = NnOutput()
        for i in range(len(self.path) - 1):
            a, b = self.path[:i + 1], self.path[i + 1:]
            if i == 0:
                a[-1].forward_game(game, b[0].input)
            else:
                a[-1].forward(b[0].input)
            relu(b[0].input)
        self.path[-1].forward(res_raw)
        res.p[:POLICY_SIZE] = res_raw[:POLICY_SIZE]
        res.v = np.tanh(res_raw[POLICY_SIZE])
        softmax(res.p)
        return res
    
    def read_weights(self):
        buffer_f32 = [0]*1000000
        idx = 0
        for i in range(len(self.path)):
            next_size = POLICY_SIZE + 1 if i == len(self.path) - 1 else self.path[i + 1].input.size
            weights_size = self.path[i].input.size * next_size
            self.path[i].weights = np.zeros((self.path[i].input.size, next_size), dtype=np.float32)
            self.path[i].bias = np.zeros(next_size, dtype=np.float32)
            self.path[i].weights.flat[:] = buffer_f32[idx:idx + weights_size]
            idx += weights_size
           
            self.path[i].bias[:] = buffer_f32[idx:idx + next_size]
            idx += next_size
def relu(v):
    return np.maximum(v, 0, out=v)

def softmax(v):
    max_val = np.max(v)
    exp_v = np.exp(v - max_val)
    sum_exp_v = np.sum(exp_v)
    v[:] = exp_v / sum_exp_v

class NNManager:
    def __init__(self):
        self.cache = {}
        self.nn = NN()
        self.access = 0
        self.hit = 0

    def get(self, game):
        game_hash = game.hash()
        if game_hash not in self.cache:
            self.cache[game_hash] = self.nn.forward(game)
        else:
            self.hit += 1
        self.access += 1
        return self.cache[game_hash]

