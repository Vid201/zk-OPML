# DNN (from OPML repo)

This is a deep neural network (DNN) featuring fully connected layers. It consists of two linear layers with a ReLU activation in between, designed for classification tasks with 784 input features (typical for flattened 28x28 images) and 10 output classes. This is the same model used in the OPML repository: [https://github.com/ora-io/opml](https://github.com/ora-io/opml)

To generate the necessary files (`network.onnx` and `input.json`) for this model, run:

```bash
python3 gen.py
```
