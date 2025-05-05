import numpy as np
from .node import Node
from .sample import Pool,SampleStore,Sample
from .nn import NNManager
from typing import Optional
from .game import *
import time
def dirichlet_noise(random_generator):
    gamma_distribution = np.random.gamma(0.5, 1.0, ACTION_SIZE)
    dirichlet_vector = np.zeros(ACTION_SIZE)
    for index in range(ACTION_SIZE):
        dirichlet_vector[index] = gamma_distribution[index]
    return dirichlet_vector
class MCTS:
    def __init__(self):
        self.pool = Pool(2000000)
        self.roots = [Node(),Node()]
        self.root=Node()
        self.nn = NNManager()
        self.nn.nn.read_weights()
        for i in range(2):
            self.roots[i].playout(self.nn, self.pool)


    def get_move_probs_selfplay(self, player: int) -> tuple[int, list[float]]:
        prob_vector = self.roots[player].prob_vector()

        dirichlet_noise_values = dirichlet_noise(self.r)
        total_sum = 0.0
        for child in self.roots[player].children:
            index = child.action
            total_sum += dirichlet_noise_values[index]

        for child in self.roots[player].children:
            index = child.action
            prob_vector[index] = prob_vector[index] * (1.0 - DIRICHLET_EPS) + DIRICHLET_EPS * dirichlet_noise_values[index] / total_sum

        best_value = 0.0
        best_action = float('inf')
        for child in self.roots[player].children:
            decision_value = prob_vector[child.action] * self.r.gen_range(0.0, 1.0)

            if decision_value > best_value:
                best_value = decision_value
                best_action = child.action

        return best_action, prob_vector
    def self_play(self, samples):
        game_instance = Game(self.r.randint(0, 2**64 - 1))
        new_samples = []
        while not game_instance.is_game_over():
            for _ in range(CONF.iters):
                self.playout(game_instance)
            actions = [69, 69]
            for player_index in range(2):
                action, probabilities = self.get_move_probs_selfplay(player_index)
                actions[player_index] = action
                sample = Sample(self.roots[player_index], game_instance, player_index)
                sample.p = probabilities[:]
                new_samples.append(sample)
                self.update_with_action(player_index, action)
            game_instance.step([
                Action(actions[0], False),
                Action(actions[1], True),
            ])
        for index, sample in enumerate(new_samples):
            sample.v = game_instance.score(index % 2)
            samples.append(sample)

        print(
            f"truns: {game_instance.game_turn}, scores: {game_instance.compute_player_score(0)} vs {game_instance.compute_player_score(1)}. "
            f"val: {self.nn.run_game(game_instance, 0).v}, ku_val: {self.roots[0].q} vs {self.roots[1].q}"
        )

    def clear(self):
        a = self.pool.pop()
        self.root = a
        self.pool.push(a)

    def update_with_action(self, action: int):
        if not self.root.children:
            self.root.select(self.nn, self.pool)
        new_root: Optional[Node] = None
        while self.root.children:
            if self.root.children[-1].game.last_move == action:
                new_root = self.root.children.pop()
            else:
                self.pool.push(self.root.children.pop())
        self.root = new_root
        self.pool.push(new_root)

    def get_move_probs_play(self, end_time=None):
        if end_time is None:
            end_time=time.time()+1
        while time.time() < end_time:
            self.root.playout(self.nn, self.pool)
        best_child = max(self.root.children, key=lambda b: b.value if self.root.terminal else b.visits)
        print(f"root visits: {self.root.visits}")
        return best_child.game.last_move

    def cg(self):
        end_time = None
        input_line = ""
        input_line = sys.stdin.readline().strip()
        my_last: int = -1
        for i in range(65):
            input_line = sys.stdin.readline().strip()
            for _ in range(7):
                input_line = sys.stdin.readline().strip()
            input_line = sys.stdin.readline().strip()
            num_valid_actions = int(input_line)
            for _ in range(num_valid_actions):
                input_line = sys.stdin.readline().strip()
            input_line = sys.stdin.readline().strip()
            if i == 0:
                end_time = time.time() + 0.9
            else:
                end_time = time.time() + 0.1
            if my_last != -1:
                self.update_with_action(my_last)
            hard_coded = -1
            if input_line != "STEAL":
                opp_action = int(input_line)
                if opp_action >= 0:
                    self.update_with_action(opp_action)
                if opp_action == -1:
                    hard_coded = 1
                    self.update_with_action(hard_coded)
                    my_last = -1
                elif i == 0:
                    hard_coded = -2