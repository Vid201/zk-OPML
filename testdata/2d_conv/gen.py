import random
import math
import numpy as np

import torch
from torch import nn
import torch.nn.functional as F
import json

# Conv2D model parameters - larger model with same operator types
input_channels = 3
input_height = 32  # Standard input size
input_width = 32

# Create a larger Conv2D model with more layers (same operator types)
model = nn.Sequential(
    # Block 1
    nn.Conv2d(3, 8, 3, padding=1),       # Conv2d
    nn.ReLU(),                           # ReLU
    nn.Conv2d(8, 8, 3, padding=1),       # Conv2d
    nn.ReLU(),                           # ReLU
    nn.MaxPool2d(2, 2),                  # MaxPool2d
    
    # Block 2
    nn.Conv2d(8, 16, 3, padding=1),      # Conv2d
    nn.ReLU(),                           # ReLU
    nn.Conv2d(16, 16, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.MaxPool2d(2, 2),                  # MaxPool2d
    
    # Block 3
    nn.Conv2d(16, 24, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.Conv2d(24, 32, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.MaxPool2d(2, 2),                  # MaxPool2d
    
    # Block 4
    nn.Conv2d(32, 32, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.Conv2d(32, 32, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.MaxPool2d(2, 2),                  # MaxPool2d
    
    # Block 5 - Additional block for more operators
    nn.Conv2d(32, 32, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.Conv2d(32, 32, 3, padding=1),     # Conv2d
    nn.ReLU(),                           # ReLU
    nn.MaxPool2d(2, 2),                  # MaxPool2d
    
    # Flatten and fully connected layers
    nn.Flatten(),                        # Flatten
    nn.Linear(32 * 1 * 1, 64),           # Linear
    nn.ReLU(),                           # ReLU
    nn.Linear(64, 10),                   # Linear
    nn.ReLU(),                           # ReLU
    nn.Linear(10, 10)                    # Linear (10 output classes)
)

# Create input tensor for Conv2D: [batch, channels, height, width]
batch_size = 1
x = torch.randn(batch_size, input_channels, input_height, input_width)

print(f"Input shape: {x.shape}")
print(f"Model parameters: {sum(p.numel() for p in model.parameters()):,}")
print(f"Model size: {sum(p.numel() for p in model.parameters()) * 4 / 1024 / 1024:.2f} MB")
print(f"Input data size: {x.numel():,} elements")

# Flips the neural net into inference mode
model.eval()
model.to('cpu')

# Export the model
torch.onnx.export(model,               # model being run
                  # model input (or a tuple for multiple inputs)
                  x,
                  # where to save the model (can be a file or file-like object)
                  "network.onnx",
                  export_params=True,        # store the trained parameter weights inside the model file
                  opset_version=10,          # the ONNX version to export the model to
                  do_constant_folding=True,  # whether to execute constant folding for optimization
                  input_names=['input'],   # the model's input names
                  output_names=['output'],  # the model's output names
                  dynamic_axes={'input': {0: 'batch_size'},    # variable length axes
                                'output': {0: 'batch_size'}})

# Flatten the input data for JSON export
data_array = x.detach().numpy().reshape([-1]).tolist()

data_json = dict(input_data=[data_array])

# print(data_json)

# Serialize data into file:
json.dump(data_json, open("input.json", 'w'))
