// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

/// @notice Struct representing a model.
struct Model {
    /// @notice Id of the model.
    uint256 id;
    /// @notice URI/location of the model.
    string uri;
    /// @notice Merkle root of the model operators.
    bytes32 root;
}

/// @notice Struct representing an inference.
struct Inference {
    /// @notice Id of the inference.
    uint256 inferenceId;
    /// @notice Flag indicating if the inference is done.
    bool done;
    /// @notice Id of the model.
    uint256 modelId;
    /// @notice Input data of the inference.
    bytes inputData;
    /// @notice Output data of the inference.
    bytes outputData;
}

/// @notice Emitted when new model is registered.
event ModelRegistered(uint256 id, string uri, bytes32 root);

/// @notice Emitted when new inference request is made.
event InferenceRequested(uint256 modelId, uint256 inferenceId, bytes inputData);

/// @notice Emitted when inference is responded.
event InferenceResponded(uint256 modelId, uint256 inferenceId, bytes outputData);

contract ModelRegistry {
    /// @notice Semantic version.
    /// @custom:sermver 0.1.0
    string public constant version = "0.1.0";

    /// @notice Counter of registered models.
    uint256 public modelCounter;

    /// @notice Mapping of all models.
    mapping(uint256 => Model) public models;

    /// @notice Counter of inference requests for all models.
    uint256 public inferenceCounter;

    /// @notice Mapping of all inferences.
    mapping(uint256 => Inference) public inferences;

    /// @notice Registers a new model.
    function registerModel(string memory uri, bytes32 root) public returns (uint256 modelId) {
        modelId = modelCounter++;
        models[modelId] = Model(modelId, uri, root);

        emit ModelRegistered(modelId, uri, root);
    }

    /// @notice Returns a registered model.
    function getModel(uint256 modelId) public view returns (Model memory model) {
        return models[modelId];
    }

    /// @notice Requests an inference for a model.
    function requestInference(uint256 modelId, bytes calldata inputData) public returns (uint256 inferenceId) {
        inferenceId = inferenceCounter++;
        inferences[inferenceId] = Inference(inferenceId, false, modelId, inputData, "");

        emit InferenceRequested(modelId, inferenceId, inputData);
    }

    /// @notice Responds to an inference request.
    function respondInference(uint256 inferenceId, bytes calldata outputData) public returns (bool success) {
        if (inferences[inferenceId].done) {
            return false;
        }

        inferences[inferenceId].done = true;
        inferences[inferenceId].outputData = outputData;

        emit InferenceResponded(inferences[inferenceId].modelId, inferenceId, outputData);

        success = true;
    }

    /// @notice Retussn an inference.
    function getInference(uint256 inferenceId) public view returns (Inference memory inference) {
        return inferences[inferenceId];
    }
}
