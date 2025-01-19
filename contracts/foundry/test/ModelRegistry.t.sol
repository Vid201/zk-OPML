// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {ModelRegistry, Inference} from "../src/ModelRegistry.sol";

contract ModelRegistryTest is Test {
    ModelRegistry public modelRegistry;

    function setUp() public {
        modelRegistry = new ModelRegistry();
    }

    function test_RegisterModel() public {
        modelRegistry.registerModel(
            "ipfs://QmTzQ1dz4N7UwW7EeLNkFvn7sPjscV8RjyC3K7yNZF2egM",
            "0x100",
            "0x200",
            bytes32(
                0xcd316985c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8
            )
        );
        assertEq(modelRegistry.modelCounter(), 1);
    }

    function test_RequestInference() public {
        uint256 modelId = 0;
        uint256 inferenceId = modelRegistry.requestInference(
            modelId,
            "0xcd316986c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8",
            bytes32(
                0xcd316985c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8
            )
        );
        assertEq(inferenceId, 0);

        Inference memory inference = modelRegistry.getInference(inferenceId);
        assertEq(inference.modelId, modelId);
        assertEq(
            inference.inputData,
            "0xcd316986c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8"
        );
        assertEq(inference.done, false);
    }

    function test_RespondInference() public {
        uint256 inferenceId = 0;
        bytes
            memory outputData = "0xcd316986c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8";
        bool success = modelRegistry.respondInference(
            inferenceId,
            outputData,
            bytes32(
                0xcd316985c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8
            )
        );
        assertEq(success, true);

        Inference memory inference = modelRegistry.getInference(inferenceId);
        assertEq(
            inference.outputData,
            "0xcd316986c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8"
        );
        assertEq(inference.done, true);

        success = modelRegistry.respondInference(
            inferenceId,
            outputData,
            bytes32(
                0xcd316985c6f85acd9dc31a14fef75077a4fb3b9607236cc0fc8f6ac0434eefa8
            )
        );
        assertEq(success, false);
    }
}
