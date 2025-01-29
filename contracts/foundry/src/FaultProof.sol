// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";

/// @notice Enum representing the action of a challenge.
enum ChallengeActor {
    /// @notice The responder is the actor.
    RESPONDER,
    /// @notice The challenger is the actor.
    CHALLENGER
}

/// @notice Struct representing a challenge.
struct Challenge {
    /// @notice Id of the challenge.
    uint256 id;
    /// @notice Id of the inference.
    uint256 inferenceId;
    /// @notice Address of the responder.
    address responder;
    /// @notice Address of the challenger.
    address challenger;
    /// @notice Timestamp of the last action.
    uint256 timestampAction;
    /// @notice Actor of the last action.
    ChallengeActor lastActor;
    /// @notice Current ONNX operator position in the challenge.
    uint256 operatorPosition;
    /// @notice Input data hash for the current ONNX operator.
    bytes32 inputDataHash;
    /// @notice Output data hash for the current ONNX operator.
    bytes32 outputDataHash;
    /// @notice Flag indicating if the challenge is resolved.
    bool resolved;
}

/// @notice Emitted when a challenge is created.
event ChallengeCreated(uint256 challengeId, uint256 inferenceId, address responder, address challenger);

contract FaultProof {
    /// @notice Model registry.
    IModelRegistry internal immutable MODEL_REGISTRY;

    /// @notice The challenge window.
    uint256 public immutable CHALLENGE_WINDOW;

    /// @notice The response window.
    uint256 public immutable RESPONSE_WINDOW;

    /// @notice Counter of challenges.
    uint256 public challengeCounter;

    /// @notice Mapping of all challenges.
    mapping(uint256 => Challenge) public challenges;

    /// @notice Returns the address of the model registry.
    function modelRegistry() public view returns (IModelRegistry modelRegistry_) {
        modelRegistry_ = MODEL_REGISTRY;
    }

    /// @notice Returns the challenge window.
    function challengeWindow() public view returns (uint256 challengeWindow_) {
        challengeWindow_ = CHALLENGE_WINDOW;
    }

    /// @notice Returns the response window.
    function responseWindow() public view returns (uint256 responseWindow_) {
        responseWindow_ = RESPONSE_WINDOW;
    }

    constructor(IModelRegistry _modelRegistry, uint256 _challengeWindow, uint256 _responseWindow) {
        MODEL_REGISTRY = _modelRegistry;
        CHALLENGE_WINDOW = _challengeWindow;
        RESPONSE_WINDOW = _responseWindow;
    }

    /// @notice Creates/opens a new challenge.
    function createChallenge(uint256 inferenceId) public returns (uint256 challengeId) {
        Inference memory inference = MODEL_REGISTRY.getInference(inferenceId);

        require(inference.done, "inference not responded yet");
        require(inference.timestampResponse + CHALLENGE_WINDOW > block.timestamp, "challenge window expired");

        uint256 modelId = inference.modelId;
        Model memory model = MODEL_REGISTRY.getModel(modelId);
        uint256 startPosition = model.numOperators / 2;

        challengeId = challengeCounter;
        challengeCounter = challengeCounter + 1;
        challenges[challengeId] = Challenge(
            challengeId,
            inferenceId,
            inference.responder,
            msg.sender,
            block.timestamp,
            ChallengeActor.CHALLENGER,
            startPosition,
            "",
            "",
            false
        );
    }

    /// @notice Responds to a challenge.
    function respondChallenge() public {
        // TODO
    }

    /// @notice Resolves a challenge.
    function resolveChallenge() public {
        // TODO
    }
}
