#![no_main]
sp1_zkvm::entrypoint!(main);

use substrate_bn::{Group, G1, G2, pairing};

pub fn main() {
    let n: u32 = sp1_zkvm::io::read();

    // Use generator points (known valid curve points)
    let g1 = G1::one();
    let g2 = G2::one();

    for i in 0..n {
        println!("cycle-tracker-report-start: pairing-{}", i);
        let _result = pairing(g1, g2);
        println!("cycle-tracker-report-end: pairing-{}", i);
    }

    sp1_zkvm::io::commit(&n);
}
