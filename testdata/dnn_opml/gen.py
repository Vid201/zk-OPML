import random
import math
import numpy as np

import torch
from torch import nn
import torch.nn.functional as F
import json

# Define model class
class Net(nn.Module):
  def __init__(self, input_size, hidden_size, num_classes):
    super(Net, self).__init__()
    self.fc1 = nn.Linear(input_size, hidden_size)
    self.relu = nn.ReLU()
    self.fc2 = nn.Linear(hidden_size, num_classes)
  
  def forward(self, x):
    out = self.fc1(x)
    out = self.relu(out)
    out = self.fc2(out)
    return out

# Define Hyperparameters
input_size = 784  # img_size = (28,28) ---> 28*28=784 in total
hidden_size = 20  # number of nodes at hidden layer
num_classes = 10  # number of output classes discrete range [0,9]
num_epochs = 20  # number of times which the entire dataset is passed throughout the model
batch_size = 100  # the size of input data took for one iteration
lr = 1e-3  # size of step

# Create model instance
model = Net(input_size, hidden_size, num_classes)

# Create input tensor: [batch, input_size]
# Using batch_size=1 for ONNX export (can use batch_size=100 for training, but ONNX export typically uses 1)
x = torch.randn(1, input_size)

print(f"Input shape: {x.shape}")
print(f"Model parameters: {sum(p.numel() for p in model.parameters()):,}")
print(f"Model size: {sum(p.numel() for p in model.parameters()) * 4 / 1024 / 1024:.2f} MB")
print(f"Input data size: {x.numel():,} elements")

# Flips the neural net into inference mode
model.eval()
model.to('cpu')

# Export the model
# Using fixed input shape [1, 784] instead of dynamic axes to avoid dimension parameter naming issues
torch.onnx.export(model,               # model being run
                  # model input (or a tuple for multiple inputs)
                  x,
                  # where to save the model (can be a file or file-like object)
                  "network.onnx",
                  export_params=True,        # store the trained parameter weights inside the model file
                  opset_version=14,          # the ONNX version to export the model to
                  do_constant_folding=True,  # whether to execute constant folding for optimization
                  input_names=['input'],   # the model's input names
                  output_names=['output'],  # the model's output names
                  dynamic_axes=None,  # Fixed batch size of 1 (no dynamic axes)
                  external_data=False)

# Flatten the input data for JSON export
data_array = x.detach().numpy().reshape([-1]).tolist()

data_json = dict(input_data=[data_array])

# print(data_json)

# Serialize data into file:
json.dump(data_json, open("input.json", 'w'))
