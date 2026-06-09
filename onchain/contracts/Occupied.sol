// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {FHE, euint8, ebool, externalEuint8} from "@fhevm/solidity/lib/FHE.sol";
import {ZamaEthereumConfig} from "@fhevm/solidity/config/ZamaConfig.sol";

contract Occupied is ZamaEthereumConfig {
    ebool public last;
    function check(externalEuint8 cell, bytes calldata proof) external {
        euint8 c = FHE.fromExternal(cell, proof);
        last = FHE.ne(c, FHE.asEuint8(0)); // occupied?
        FHE.allowThis(last);
        FHE.allow(last, msg.sender);
    }
}
