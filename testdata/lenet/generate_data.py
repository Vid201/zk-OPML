#!/usr/bin/env python3
"""
Generate input data and run inference for LeNet model.

This script generates random input data compatible with the LeNet ONNX model,
runs inference, and saves both input and output data to JSON files.
"""

import json
import numpy as np
import onnx
import onnxruntime as ort


def generate_input_data(model_path):
    """
    Generate random input data compatible with the ONNX model.

    Args:
        model_path (str): Path to the ONNX model file

    Returns:
        tuple: Generated input data shape
    """
    # Load the ONNX model
    model = onnx.load(model_path)

    # Get the input shape from the model
    input_shape = None
    for input_info in model.graph.input:
        # Get the shape from the type information
        shape = []
        for dim in input_info.type.tensor_type.shape.dim:
            if dim.dim_value:
                shape.append(dim.dim_value)
            else:
                # If dimension is not specified, use 1 as default
                shape.append(1)
        input_shape = tuple(shape)
        break  # We only need the first input

    if not input_shape:
        raise ValueError("Could not determine input shape from model")

    # Generate random input data
    # Using small random values between -1 and 1 for better numerical stability
    input_data = np.random.uniform(-1, 1, input_shape).astype(np.float32)

    # Convert to list for JSON serialization
    input_list = input_data.tolist()

    # Create JSON object with input_data property
    json_data = {
        "input_data": input_list
    }

    # Save to JSON file
    with open('input.json', 'w') as f:
        json.dump(json_data, f, indent=2)

    print(f"Generated input data with shape: {input_shape}")
    print(f"Saved to input.json")

    return input_shape


def run_inference(model_path, input_data_path):
    """
    Run inference using the ONNX model and save output data.

    Args:
        model_path (str): Path to the ONNX model file
        input_data_path (str): Path to the input JSON file

    Returns:
        list: Model outputs
    """
    # Read input data from JSON file
    with open(input_data_path, 'r') as f:
        json_data = json.load(f)
    input_data = np.array(json_data["input_data"], dtype=np.float32)

    # Create inference session
    session = ort.InferenceSession(model_path)

    # Get input name for the model
    input_name = session.get_inputs()[0].name

    # Prepare input dictionary
    input_dict = {input_name: input_data}

    # Run inference
    outputs = session.run(None, input_dict)

    # Save output data in the same format as input data
    output_list = outputs[0].tolist()

    output_json = {
        "output_data": output_list
    }

    with open('output.json', 'w') as f:
        json.dump(output_json, f, indent=2)

    print(f"Generated output data with shape: {outputs[0].shape}")
    print(f"Saved to output.json")

    return outputs


if __name__ == "__main__":
    model_path = "model.onnx"

    try:
        # Generate input data
        input_shape = generate_input_data(model_path)

        # Run inference and generate output data
        outputs = run_inference(model_path, "input.json")

        print(f"\nSuccessfully completed data generation and inference!")
        print(f"Input shape: {input_shape}")
        print(f"Output shape: {outputs[0].shape}")

    except FileNotFoundError:
        print("Error: model.onnx not found. Please download the model first.")
    except Exception as e:
        print(f"Error: {e}")
