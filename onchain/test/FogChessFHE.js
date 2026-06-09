const { ethers, fhevm } = require("hardhat");
const { expect } = require("chai");

function board(pairs) {
  const b = new Array(64).fill(0);
  for (const [s, p] of pairs) b[s] = p;
  return b;
}

async function deployWith(cells) {
  const [alice, bob] = await ethers.getSigners();
  const c = await (await ethers.getContractFactory("FogChessFHE")).deploy();
  await c.waitForDeployment();
  const addr = await c.getAddress();
  const input = fhevm.createEncryptedInput(addr, bob.address);
  for (let i = 0; i < 64; i++) input.add8(cells[i]);
  const enc = await input.encrypt();
  await (await c.connect(bob).commitBoard(enc.handles, enc.inputProof)).wait();
  return { c, addr, alice, bob };
}

async function decrypt(c, addr, signer) {
  return fhevm.userDecryptEbool(await c.result(), addr, signer);
}

describe("FogChessFHE (real fhEVM SDK, mock coprocessor)", function () {
  it("occupancy of a public square", async function () {
    if (!fhevm.isMock) this.skip();
    const { c, addr, alice, bob } = await deployWith(board([[60, 4]])); // enemy rook on e8
    await (await c.connect(alice).occupancy(bob.address, 60)).wait();
    expect(await decrypt(c, addr, alice)).to.equal(true);
    await (await c.connect(alice).occupancy(bob.address, 20)).wait();
    expect(await decrypt(c, addr, alice)).to.equal(false);
  });

  it("inCheck: open enemy rook on the file is check", async function () {
    if (!fhevm.isMock) this.skip();
    const { c, addr, alice, bob } = await deployWith(board([[60, 4]])); // rook e8, our king e1 (4)
    await (await c.connect(alice).inCheck(bob.address, 4, true)).wait();
    expect(await decrypt(c, addr, alice)).to.equal(true);
  });

  it("inCheck: a blocked rook is not check", async function () {
    if (!fhevm.isMock) this.skip();
    const { c, addr, alice, bob } = await deployWith(board([[60, 4], [36, 1]])); // rook e8 + pawn e5
    await (await c.connect(alice).inCheck(bob.address, 4, true)).wait();
    expect(await decrypt(c, addr, alice)).to.equal(false);
  });
});
