const { ethers, fhevm } = require("hardhat");
const { expect } = require("chai");

describe("Occupied (fhEVM mock)", function () {
  it("decrypts the occupied bit", async function () {
    if (!fhevm.isMock) this.skip();
    const [alice] = await ethers.getSigners();
    const c = await (await ethers.getContractFactory("Occupied")).deploy();
    await c.waitForDeployment();
    const addr = await c.getAddress();

    const enc = await fhevm.createEncryptedInput(addr, alice.address).add8(5).encrypt();
    await (await c.connect(alice).check(enc.handles[0], enc.inputProof)).wait();

    const clear = await fhevm.userDecryptEbool(await c.last(), addr, alice);
    expect(clear).to.equal(true);
  });
});
