#!/usr/bin/env python
import tensorflow as tf
from keras import regularizers
import numpy as np
import os
import subprocess

W = 9;
H = 7;

def save_model(model, fileSTR):
    Wmodel = open("./"+fileSTR, "wb")
    for x in model.weights:
        nn = x.numpy()
        v = np.ndarray.tobytes(nn)
        Wmodel.write(bytearray(v))
    Wmodel.close()

POLICY_SIZE = W;
INPUT_SIZE = H * W * 2;

K_BATCH_SIZE=256
K_EPOCH=10  

def dense(features, x):
    x = tf.keras.layers.Dense(
        features,
        activation = 'relu',
        kernel_regularizer=regularizers.l1_l2(l1=1e-5, l2=1e-4),
        bias_regularizer=regularizers.l2(1e-4),
        )(x)
    return x

inputs = tf.keras.Input(shape=(INPUT_SIZE,), name='input')
x = dense(128, inputs)
x = dense(64, x)
x = tf.keras.layers.Dense(
        POLICY_SIZE + 1,
        activation = None,
        kernel_regularizer=regularizers.l1_l2(l1=1e-5, l2=1e-4),
        bias_regularizer=regularizers.l2(1e-4),
        )(x)
policy, value = tf.split(x, num_or_size_splits=[POLICY_SIZE, 1], axis=-1)
policy = tf.keras.layers.Activation(tf.nn.softmax, name='policy')(policy)
value = tf.keras.layers.Activation(tf.nn.tanh, name='value')(value)
model = tf.keras.Model(inputs=inputs, outputs=[value, policy])

opt = tf.keras.optimizers.Adam()

def loss_func(y_true, y_pred):
    return tf.losses.categorical_crossentropy(y_true, y_pred, from_logits = False) 

model.compile(
    loss={'value': 'mean_squared_error', 'policy': loss_func },
    optimizer=opt,
)

MODEL_FILE = 'best'
postfix_32 = '.w32'
postfix_keras = '.h5'
keras_model_file = MODEL_FILE + postfix_keras

def save_all(model, prefix):
    file_keras = prefix + postfix_keras
    save_model(model, prefix + postfix_32)
    model.save(file_keras)
    subprocess.run("cargo run -q --release -- --encode", shell=True)
   
if True and os.path.exists(keras_model_file):
    model = tf.keras.models.load_model(keras_model_file, custom_objects={'loss_func': loss_func})

save_all(model, MODEL_FILE)
np.set_printoptions(suppress=True)
model.optimizer.learning_rate.assign(0.001)

while True:   
    list_of_files = os.listdir('traindata')
    if list_of_files:
        csv_data = np.array([], dtype=float)
        full_path = ["traindata/{0}".format(x) for x in list_of_files]
        full_path.sort(key=os.path.getctime)
        full_path = full_path[-40:]
        for f in full_path:
            csv_data = np.append(csv_data, np.fromfile(f, dtype=np.float32))
        csv_data=np.reshape(csv_data, (-1,INPUT_SIZE+POLICY_SIZE+1))
        np.random.shuffle(csv_data)
        cut_index = [(csv_data.shape)[1]-POLICY_SIZE-1, (csv_data.shape)[1]-1]
        samples,policy,value=np.split(csv_data, cut_index,axis=1)
        model.fit({'input':samples}, {'policy': policy, 'value':value},verbose=2, epochs = 10, batch_size=int(K_BATCH_SIZE))
        save_all(model, MODEL_FILE)
    subprocess.run("cargo run --release", shell=True)
   
