// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";

import {ISP1Verifier} from "sp1-contracts/src/ISP1Verifier.sol";

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
    /// @notice Lower bound of the ONNX operator position (binary search).
    uint256 operatorLow;
    /// @notice Upper bound of the ONNX operator position (binary search).
    uint256 operatorHigh;
    /// @notice Input data hash of the inference.
    bytes32 inputDataHash;
    /// @notice Output data hash of the inference.
    bytes32 outputDataHash;
    /// @notice Flag indicating whether the challenge is ready to be resolved.
    bool ready;
    /// @notice Flag indicating if the challenge is resolved.
    bool resolved;
    /// @notice Winning actor;
    ChallengeActor winner;
}

/// @notice Struct representing a proposed operator execution.
struct OperatorExecution {
    /// @notice Input data hash.
    bytes32 inputDataHash;
    /// @notice Output data hash.
    bytes32 outputDataHash;
}

/// @notice Emitted when a challenge is created.
event ChallengeCreated(uint256 challengeId, uint256 inferenceId, address responder, address challenger);

/// @notice Emitted when new operator execution is proposed.
event OperatorExecutionProposed(
    uint256 challengeId, uint256 operatorPosition, bytes32 inputDataHash, bytes32 outputDataHash
);

/// @notice Emitted when submitter responds to a proposed operator execution.
event OperatorExecutionResponded(uint256 challengeId, uint256 operatorPosition, bool input, bool output);

/// @notice Emitted when a challenge is resolved.
event ChallengeResolved(uint256 challengeId, bool success, address winner);

contract FaultProof {
    /// @notice Model registry.
    IModelRegistry internal immutable MODEL_REGISTRY;

    /// @notice The challenge window.
    uint256 public immutable CHALLENGE_WINDOW;

    /// @notice The response window.
    uint256 public immutable RESPONSE_WINDOW;

    /// @notice Address of the SP1 verifier contract.
    address public immutable SP1_VERIFIER;

    /// @notice SP1 program verification key.
    bytes32 public immutable PROGRAM_VKEY;

    /// @notice Counter of challenges.
    uint256 public challengeCounter;

    /// @notice Mapping of all challenges.
    mapping(uint256 => Challenge) public challenges;

    /// @notice Mapping of all operator executions.
    mapping(uint256 => mapping(uint256 => OperatorExecution)) public operatorExecutions;

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

    constructor(
        IModelRegistry _modelRegistry,
        uint256 _challengeWindow,
        uint256 _responseWindow,
        address _sp1Verifier,
        bytes32 _programVKey
    ) {
        MODEL_REGISTRY = _modelRegistry;
        CHALLENGE_WINDOW = _challengeWindow;
        RESPONSE_WINDOW = _responseWindow;
        SP1_VERIFIER = _sp1Verifier;
        PROGRAM_VKEY = _programVKey;
    }

    /// @notice Returns the challenge with the given id.
    function getChallenge(uint256 challengeId) public view returns (Challenge memory) {
        return challenges[challengeId];
    }

    /// @notice Creates/opens a new challenge.
    function createChallenge(uint256 inferenceId) public returns (uint256 challengeId) {
        Inference memory inference = MODEL_REGISTRY.getInference(inferenceId);

        require(inference.done, "inference not responded yet");
        require(inference.timestampResponse + CHALLENGE_WINDOW > block.timestamp, "challenge window expired");

        uint256 modelId = inference.modelId;
        Model memory model = MODEL_REGISTRY.getModel(modelId);

        challengeId = challengeCounter;
        challengeCounter = challengeCounter + 1;
        challenges[challengeId] = Challenge(
            challengeId,
            inferenceId,
            inference.responder,
            msg.sender,
            block.timestamp,
            ChallengeActor.RESPONDER,
            0,
            model.numOperators - 1,
            inference.inputDataHash,
            inference.outputDataHash,
            false,
            false,
            ChallengeActor.RESPONDER
        );

        emit ChallengeCreated(challengeId, inferenceId, inference.responder, msg.sender);
    }

    /// @notice Challenger proposes an operator execution.
    function proposeOperatorExecution(uint256 challengeId, bytes32 inputDataHash, bytes32 outputDataHash) public {
        require(challengeId < challengeCounter, "challenge does not exist");
        require(!challenges[challengeId].ready || !challenges[challengeId].resolved, "challenge is ready or resolved");
        require(challenges[challengeId].challenger == msg.sender, "only challenger can propose operator execution");
        require(challenges[challengeId].lastActor == ChallengeActor.RESPONDER, "last actor must be responder");
        require(
            challenges[challengeId].timestampAction + RESPONSE_WINDOW > block.timestamp, "response time window expired"
        );

        uint256 mid = (challenges[challengeId].operatorLow + challenges[challengeId].operatorHigh) / 2;

        uint256 inferenceId = challenges[challengeId].inferenceId;
        Inference memory inference = MODEL_REGISTRY.getInference(inferenceId);
        uint256 modelId = inference.modelId;
        Model memory model = MODEL_REGISTRY.getModel(modelId);

        if (mid == 0) {
            require(
                inputDataHash == challenges[challengeId].inputDataHash, "input data hash does not match (condition 1)"
            );
        }
        if (mid == model.numOperators - 1) {
            require(
                outputDataHash != challenges[challengeId].outputDataHash,
                "output data hash must not match (condition 2)"
            );
        }
        if (mid > 0 && operatorExecutions[challengeId][mid - 1].outputDataHash != bytes32(0)) {
            require(
                operatorExecutions[challengeId][mid - 1].outputDataHash == inputDataHash,
                "input data hash does not match (condition 3)"
            );
        }
        if (operatorExecutions[challengeId][mid + 1].inputDataHash != bytes32(0)) {
            require(
                operatorExecutions[challengeId][mid + 1].inputDataHash == outputDataHash,
                "input data hash does not match (condition 4)"
            );
        }

        operatorExecutions[challengeId][mid] = OperatorExecution(inputDataHash, outputDataHash);

        challenges[challengeId].lastActor = ChallengeActor.CHALLENGER;
        challenges[challengeId].timestampAction = block.timestamp;

        if (challenges[challengeId].operatorHigh - challenges[challengeId].operatorLow == 0) {
            challenges[challengeId].ready = true;
        }

        emit OperatorExecutionProposed(
            challengeId,
            (challenges[challengeId].operatorLow + challenges[challengeId].operatorHigh) / 2,
            inputDataHash,
            outputDataHash
        );
    }

    /// @notice Responder responds to operator execution.
    /// @dev Proposer can either:
    /// 1. Agree with the input/output data hash. - go right
    /// 2. Disagree with the input data hash. - go left
    /// 3. Agree with input data hash, but disagree with output data hash. - propose different operator execution
    function respondOperatorExecution(uint256 challengeId, bool input, bool output) public {
        require(challengeId < challengeCounter, "challenge does not exist");
        require(!challenges[challengeId].ready || !challenges[challengeId].resolved, "challenge is ready or resolved");
        require(challenges[challengeId].responder == msg.sender, "only responder can to respond operator execution");
        require(challenges[challengeId].lastActor == ChallengeActor.CHALLENGER, "last actor must be challenger");
        require(
            challenges[challengeId].timestampAction + RESPONSE_WINDOW > block.timestamp, "response time window expired"
        );

        uint256 mid = (challenges[challengeId].operatorLow + challenges[challengeId].operatorHigh) / 2;

        // TODO: check if the responder already agreed with the input/output data hash (mid - 1 or mid + 1)

        // 1.
        if (input && output) {
            challenges[challengeId].operatorLow = mid + 1;
        }
        // 2.
        else if (!input) {
            challenges[challengeId].operatorHigh = mid - 1;
        }
        // 3.
        else {
            challenges[challengeId].ready = true;
        }

        challenges[challengeId].lastActor = ChallengeActor.RESPONDER;
        challenges[challengeId].timestampAction = block.timestamp;

        emit OperatorExecutionResponded(challengeId, mid, input, output);
    }

    /// @notice Can be called by anyone, but will usually be called by challenger.
    /// @dev This function will call SP1 verifier contract to verify ZKP of the operator execution.
    function resolveOpenChallenge(uint256 challengeId, bytes calldata publicValues, bytes calldata proofBytes) public {
        require(challenges[challengeId].ready, "challenge not ready to be resolved yet");
        require(!challenges[challengeId].resolved, "challenge already resolved");

        // SP1 verification
        ISP1Verifier(SP1_VERIFIER).verifyProof(PROGRAM_VKEY, publicValues, proofBytes);

        // Verify the public commitments of the proof
        bytes32 merkleRoot = bytes32(publicValues[:66]);
        bytes16 leafIndices = bytes16(publicValues[32:66]);
        bytes32 inputDataHash = bytes32(publicValues[48:80]);
        bytes32 outputDataHash = bytes32(publicValues[80:112]);

        uint256 mid = (challenges[challengeId].operatorLow + challenges[challengeId].operatorHigh) / 2;
        uint256 modelId = MODEL_REGISTRY.getInference(challenges[challengeId].inferenceId).modelId;
        Model memory model = MODEL_REGISTRY.getModel(modelId);

        // Verify merkle root
        require(merkleRoot == model.root, "merkle root does not match");

        // Verify leaf indices
        // TODO: support execution of multiple ONNX operators
        uint256 leaf_index = littleToBigEndian(leafIndices);
        require(leaf_index == mid, "leaf index does not match current ONNX operator");

        // Verify input data hash
        require(
            operatorExecutions[challengeId][mid].inputDataHash != bytes32(0)
                && inputDataHash == operatorExecutions[challengeId][mid].inputDataHash,
            "input data hash does not match"
        );

        // Verify output data hash
        require(
            operatorExecutions[challengeId][mid].outputDataHash != bytes32(0)
                && outputDataHash == operatorExecutions[challengeId][mid].outputDataHash,
            "output data hash does not match"
        );

        challenges[challengeId].winner = ChallengeActor.CHALLENGER;
        // TODO: slash the responder

        challenges[challengeId].resolved = true;

        emit ChallengeResolved(challengeId, challenges[challengeId].winner == ChallengeActor.CHALLENGER, challenges[challengeId].challenger);
    }

    // This can be called by responder to resolve the expired challenge.
    function resolveExpiredChallenge(uint256 challengeId) public {
        require(
            challenges[challengeId].timestampAction + RESPONSE_WINDOW < block.timestamp,
            "challenge window not expired yet"
        );
        require(!challenges[challengeId].resolved, "challenge already resolved");

        address winner;

        if (challenges[challengeId].lastActor == ChallengeActor.RESPONDER || challenges[challengeId].ready) {
            challenges[challengeId].winner = ChallengeActor.RESPONDER;
            winner = challenges[challengeId].responder;
            // TODO: slash the challenger
        } else {
            challenges[challengeId].winner = ChallengeActor.CHALLENGER;
            winner = challenges[challengeId].challenger;
            // TODO: slash the responder
        }

        challenges[challengeId].resolved = true;

        emit ChallengeResolved(challengeId, challenges[challengeId].winner == ChallengeActor.CHALLENGER, winner);
    }

    function littleToBigEndian(bytes16 input) public pure returns (uint256) {
        // Extract the last 8 bytes by shifting
        bytes8 last8Bytes = bytes8(input << 64);
        uint256 result = 0;

        uint256 mul = 1;

        for (uint256 i = 0; i < 8; i++) {
            result += uint256(uint8(last8Bytes[i])) * mul;
            mul *= 16 * 16;
        }

        return result;
    }
}
