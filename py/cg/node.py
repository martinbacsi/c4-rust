from .game import *
from .nn import NNManager
from .sample import CONF
from typing import List
class Node:
    def __init__(self):
        self.terminal = False
        self.visits = 0
        self.value = -1
        self.q = 0.0
        self.p = 0.0
        self.children:List[Node] = []
        self.game = Connect4()  # Assuming Connect4 class is defined elsewhere
        self.expanded = False
        self.live_child = 0

    def reinit(self):
        self.terminal = False
        self.visits = 0
        self.value = -1
        self.q = 0.0
        self.p = 0.0
        self.children.clear()
        self.game = Connect4()  # Assuming Connect4 class is defined elsewhere
        self.expanded = False
        self.live_child = 0
        return self

    def ucb(self, multiplier):
        return (self.p * multiplier + self.q) / (1 + self.visits)

    def select(self, nn_manager, pool):
        if not self.children:
            nn_value = nn_manager.get(self.game)  # Assuming get method is defined in NNManager
            for action in range(POLICY_SIZE):  # Assume POLICY_SIZE is defined
                if self.game.height[action] < HEIGHT:  # Assuming HEIGHT is defined
                    self.live_child |= 1 << action
                    n = pool.pop()  # Assuming pop method is defined in Pool
                    n.p = nn_value.p[action]
                    n.game = self.game
                    n.game.step(action)  # Assuming step method is defined in Connect4
                    self.children.append(n)

        multiplier = CONF.cpuct * (self.visits ** 0.5)  # Assuming CONF is defined
        best_value = float('-inf')
        best_index = 0
        for child_index in range(len(self.children)):
            child = self.children[child_index]
            value = child.ucb(multiplier)
            if value > best_value:
                best_value = value
                best_index = child_index
        return best_index

    def expand(self, nn_manager:NNManager):
        nn_value = nn_manager.get(self.game)  # Assuming get method is defined in NNManager
        self.expanded = True
        return nn_value.v

    def update(self, value):
        self.visits += 1
        self.q += value

    def prob_vector(self):
        probabilities = [0.0] * POLICY_SIZE  # Assume POLICY_SIZE is defined
        total_sum = 0.0
        if self.terminal:
            for child in self.children:
                if child.terminal and child.value == -self.value:
                    probabilities[child.game.last_move] = 1.0  # Assuming last_move is defined
                    total_sum += 1.0
        else:
            for child in self.children:
                probabilities[child.game.last_move] = float(child.visits)
                total_sum += probabilities[child.game.last_move]
        for i in range(len(probabilities)):
            probabilities[i] /= total_sum
        return probabilities

    def playout(self, nn_manager, pool):
        if self.game.outcome != Outcome.NULL:  # Assuming Outcome is defined
            self.value = 1 if self.game.outcome == Outcome.Win else 0
            self.terminal = True

        if not self.expanded:
            value = self.expand(nn_manager)
        else:
            child_index = self.select(nn_manager, pool)
            if self.terminal:
                value = float(self.value)
            else:
                child = self.children[child_index]
                value = -child.playout(nn_manager, pool)
                if child.terminal:
                    self.value = max(self.value, child.value)
                    if child.value == 1:
                        self.live_child = 0
                    else:
                        self.live_child ^= 1 << child.game.last_move
                    if self.live_child == 0:
                        self.terminal = True
                        self.value = -self.value

        self.update(value)
        return value
