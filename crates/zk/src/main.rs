#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let merkle_root = sp1_zkvm::io::read::<[u8; 32]>();
    let merkle_proof = sp1_zkvm::io::read::<Vec<u8>>();
    // onnx operator
    let operator_index = sp1_zkvm::io::read::<u32>();

    // sp1_zkvm::io::commit(&n);

    // let mut a = 0;
    // let mut b = 1;
    // for _ in 0..n {
    //     let mut c = a + b;
    //     c %= 7919;
    //     a = b;
    //     b = c;
    // }

    // sp1_zkvm::io::commit(&a);
    // sp1_zkvm::io::commit(&b);
}
