
import os
from .sample import Config
import threading
from .mcts import MCTS
import random
from .sample import Sample,SampleStore
from .code_dec import DecodeBase16K
import sys





def main():
    if "--encode" in os.sys.argv:
        enc = DecodeBase16K.encode_b16k("best.w32")
        st = enc.decode("utf-16")
        path = "src/nn_string.py"
        if os.path.exists(path):
            os.remove(path)
        with open(path, 'w') as output:
            output.write(f'NNSTR = "{st}"')
    elif sys.argv[-1]=='train':
        handles = []
        for _ in range(4):
            handle = threading.Thread(target=run_mcts)
            handle.start()
            handles.append(handle)
        for h in handles:
            h.join()
    elif sys.argv[-1]=='test':
        run_mcts()
    else:
        MCTS().cg()

def run_mcts():
    rng = random.Random()
    sample_store = SampleStore()
    mcts = MCTS()  # Assuming MCTS is defined elsewhere
    for i in range(250):
        print(i)
        mcts.self_play(sample_store)  # Assuming self_play is defined in MCTS
        mcts.clear()  # Assuming clear is defined in MCTS
    file_path = f"./traindata/{rng.randint(0, 2**32)}"
    with open(file_path, 'wb') as file:
        for _, sample in sample_store.samples.items():
            sample.v /= sample.visits
            for j in range(len(sample.p)):
                sample.p[j] /= sample.visits
            a = bytearray(sample.input)
            file.write(a)
            a = bytearray(sample.p)
            file.write(a)
            a = bytearray([sample.v])
            file.write(a)

if __name__ == "__main__":
    main()