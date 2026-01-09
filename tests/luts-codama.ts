import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";
import { expect } from "chai";
import {
  getUserAddressLookupTableAddress,
  deriveAddressLookupTableAddress,
  buildCreateAddressLookupTableInstruction,
  buildExtendAddressLookupTableInstruction,
  processAndValidateTransaction,
} from "../codama-ts-luts-custom";
import {
  getUserAddressLookupTableDecoder,
  type UserAddressLookupTable,
} from "../codama-ts-luts";
import { LUTS_PROGRAM_ADDRESS } from "../codama-ts-luts/programs";

describe("luts-codama", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const connection = provider.connection;
  const signer = (provider.wallet as anchor.Wallet).payer;

  let lutId = 100n;

  async function fetchUserAddressLookupTable(
    address: PublicKey
  ): Promise<UserAddressLookupTable> {
    const accountInfo = await connection.getAccountInfo(address);
    if (!accountInfo) {
      throw new Error("Account not found");
    }
    const decoder = getUserAddressLookupTableDecoder();
    return decoder.decode(accountInfo.data);
  }

  async function createLut(id: bigint): Promise<{
    userAddressLookupTable: PublicKey;
    addressLookupTable: PublicKey;
  }> {
    const recentSlot = BigInt(await connection.getSlot("finalized"));

    const { instruction, userAddressLookupTable, addressLookupTable } =
      buildCreateAddressLookupTableInstruction({
        signer: signer.publicKey,
        recentSlot,
        id,
      });

    await processAndValidateTransaction([instruction], connection, signer);

    return { userAddressLookupTable, addressLookupTable };
  }

  it("creates an address lookup table", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    const account = await fetchUserAddressLookupTable(userAddressLookupTable);

    expect(account.signer).to.equal(signer.publicKey.toBase58());
    expect(account.addressLookupTable).to.equal(addressLookupTable.toBase58());
    expect(account.id).to.equal(id);
    expect(account.size).to.equal(0n);
    expect(Number(account.lastUpdatedSlot)).to.be.greaterThan(0);
  });

  it("rejects extend during cooldown period (LutNotReady)", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    const addr1 = PublicKey.unique();

    const instruction = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1],
    });

    try {
      await processAndValidateTransaction([instruction], connection, signer);
      expect.fail("Expected LutNotReady error");
    } catch (err) {
      expect((err as Error).message).to.include("LutNotReady");
    }
  });

  it("extends an address lookup table with new addresses after cooldown", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();

    const instruction = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1, addr2],
    });

    await processAndValidateTransaction([instruction], connection, signer);

    const updatedAccount = await fetchUserAddressLookupTable(
      userAddressLookupTable
    );

    expect(updatedAccount.size).to.equal(2n);
  });

  it("rejects adding only duplicate addresses (NoNewAddresses)", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();

    const extendIx1 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1],
    });

    await processAndValidateTransaction([extendIx1], connection, signer);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const extendIx2 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1],
    });

    try {
      await processAndValidateTransaction([extendIx2], connection, signer);
      expect.fail("Expected NoNewAddresses error");
    } catch (err) {
      expect((err as Error).message).to.include("NoNewAddresses");
    }
  });

  it("size field stays in sync with addresses added", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    let account = await fetchUserAddressLookupTable(userAddressLookupTable);
    expect(account.size).to.equal(0n);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();
    const addr3 = PublicKey.unique();

    const extendIx1 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1, addr2, addr3],
    });

    await processAndValidateTransaction([extendIx1], connection, signer);

    account = await fetchUserAddressLookupTable(userAddressLookupTable);
    expect(account.size).to.equal(3n);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr4 = PublicKey.unique();
    const addr5 = PublicKey.unique();

    const extendIx2 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr4, addr5],
    });

    await processAndValidateTransaction([extendIx2], connection, signer);

    account = await fetchUserAddressLookupTable(userAddressLookupTable);
    expect(account.size).to.equal(5n);
  });

  it("filters duplicate addresses and only adds new ones", async () => {
    const id = lutId;
    lutId += 1n;

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();

    const extendIx1 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1, addr2],
    });

    await processAndValidateTransaction([extendIx1], connection, signer);

    let account = await fetchUserAddressLookupTable(userAddressLookupTable);
    expect(account.size).to.equal(2n);

    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr3 = PublicKey.unique();

    const extendIx2 = buildExtendAddressLookupTableInstruction({
      signer: signer.publicKey,
      addressLookupTable,
      userAddressLookupTable,
      newAddresses: [addr1, addr3], // addr1 is duplicate
    });

    await processAndValidateTransaction([extendIx2], connection, signer);

    account = await fetchUserAddressLookupTable(userAddressLookupTable);
    expect(account.size).to.equal(3n); // Only 1 new address added
  });
});
