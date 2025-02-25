// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";

/// @notice Emitted when new model is registered.
event ModelRegistered(uint256 id, string uri, address registrar, bytes32 root, uint256 numOperators);

/// @notice Emitted when new inference request is made.
event InferenceRequested(
    uint256 modelId, uint256 inferenceId, address requester, bytes inputData, bytes32 inputDataHash
);

/// @notice Emitted when inference is responded.
event InferenceResponded(
    uint256 modelId, uint256 inferenceId, address responder, bytes outputData, bytes32 outputDataHash
);

contract ModelRegistry is IModelRegistry {
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
    function registerModel(string memory uri, bytes32 root, uint256 numOperators) public returns (uint256 modelId) {
        modelId = modelCounter;
        modelCounter = modelCounter + 1;
        models[modelId] = Model(modelId, uri, msg.sender, root, numOperators);

        emit ModelRegistered(modelId, uri, msg.sender, root, numOperators);
    }

    /// @notice Returns a registered model.
    function getModel(uint256 modelId) public view returns (Model memory model) {
        return models[modelId];
    }

    /// @notice Requests an inference for a model.
    // TODO: inputData should be URI reference to IPFS
    function requestInference(uint256 modelId, bytes calldata inputData, bytes32 inputDataHash)
        public
        returns (uint256 inferenceId)
    {
        inferenceId = inferenceCounter;
        inferenceCounter = inferenceCounter + 1;
        inferences[inferenceId] = Inference(
            inferenceId, block.timestamp, 0, false, msg.sender, address(0), modelId, inputData, inputDataHash, "", ""
        );

        emit InferenceRequested(modelId, inferenceId, msg.sender, inputData, inputDataHash);
    }

    /// @notice Responds to an inference request.
    // TODO: outputData should be URI reference to IPFS
    function respondInference(uint256 inferenceId, bytes calldata outputData, bytes32 outputDataHash)
        public
        returns (bool success)
    {
        if (inferences[inferenceId].done) {
            return false;
        }

        inferences[inferenceId].timestampResponse = block.timestamp;
        inferences[inferenceId].done = true;
        inferences[inferenceId].responder = msg.sender;
        inferences[inferenceId].outputData = outputData;
        inferences[inferenceId].outputDataHash = outputDataHash;

        emit InferenceResponded(inferences[inferenceId].modelId, inferenceId, msg.sender, outputData, outputDataHash);

        success = true;
    }

    /// @notice Returns an inference.
    function getInference(uint256 inferenceId) public view returns (Inference memory inference) {
        return inferences[inferenceId];
    }
}
