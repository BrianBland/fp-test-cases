// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

contract TransferScript is Script {
    function setUp() public {}

    function run(uint256 gas_target, address to) public {
        vm.startBroadcast();

        uint256 start_gas = gasleft();
        uint256 gas_used = 0;

        while (gas_used < gas_target) {
            payable(to).transfer(1 wei);
            gas_used = start_gas - gasleft();
        }

        vm.stopBroadcast();
    }
}
