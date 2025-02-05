use alloy::sol;

sol!(
    #[sol(rpc)]
    ModelRegistry,
    "foundry/out/ModelRegistry.sol/ModelRegistry.json"
);

sol!(
    #[sol(rpc)]
    FaultProof,
    "foundry/out/FaultProof.sol/FaultProof.json"
);
