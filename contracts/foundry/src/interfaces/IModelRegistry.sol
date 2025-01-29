// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

/// @notice Struct representing a model.
struct Model {
    /// @notice Id of the model.
    uint256 id;
    /// @notice URI/location of the model.
    string uri;
    /// @notice Address of the model registrar.
    address registrar;
    /// @notice Input shape of the model.
    bytes inputShape;
    /// @notice Output shape of the model.
    bytes outputShape;
    /// @notice Merkle root of the model operators.
    bytes32 root;
    /// @notice Number of ONNX operators in the model.
    uint256 numOperators;
}

/// @notice Struct representing an inference.
struct Inference {
    /// @notice Id of the inference.
    uint256 inferenceId;
    /// @notice Timestamp of the inference request.
    uint256 timestampRequest;
    /// @notice Timestamp of the inference response.
    uint256 timestampResponse;
    /// @notice Flag indicating if the inference is done.
    bool done;
    /// @notice Address of the requester.
    address requester;
    /// @notice Address of the responder.
    address responder;
    /// @notice Id of the model.
    uint256 modelId;
    /// @notice Input data of the inference.
    bytes inputData;
    /// @notice Input data hash.
    bytes32 inputDataHash;
    /// @notice Output data of the inference.
    bytes outputData;
    /// @notice Output data hash.
    bytes32 outputDataHash;
}

interface IModelRegistry {
    /// @notice Returns a registered model.
    function getModel(uint256 modelId) external view returns (Model memory model);

    /// @notice Returns an inference.
    function getInference(uint256 inferenceId) external view returns (Inference memory inference);
}
