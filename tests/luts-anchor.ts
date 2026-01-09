import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorError } from "@coral-xyz/anchor";
import { Luts } from "../target/types/luts";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { expect } from "chai";

describe("luts", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.luts as Program<Luts>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const signer = provider.wallet.publicKey;

  const ADDRESS_LOOKUP_TABLE_PROGRAM = new PublicKey(
    "AddressLookupTab1e1111111111111111111111111"
  );

  let lutId = new anchor.BN(0);

  function getUserAddressLookupTablePda(
    signer: PublicKey,
    id: anchor.BN
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("UserAddressLookupTable"),
        signer.toBuffer(),
        id.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
  }

  async function deriveAddressLookupTable(
    authority: PublicKey,
    recentSlot: anchor.BN
  ): Promise<PublicKey> {
    const [lutAddress] = PublicKey.findProgramAddressSync(
      [authority.toBuffer(), recentSlot.toArrayLike(Buffer, "le", 8)],
      ADDRESS_LOOKUP_TABLE_PROGRAM
    );
    return lutAddress;
  }

  async function createLut(id: anchor.BN): Promise<{
    userAddressLookupTable: PublicKey;
    addressLookupTable: PublicKey;
  }> {
    const recentSlot = new anchor.BN(
      await provider.connection.getSlot("finalized")
    );

    const [userAddressLookupTable] = getUserAddressLookupTablePda(signer, id);
    const addressLookupTable = await deriveAddressLookupTable(
      userAddressLookupTable,
      recentSlot
    );

    await program.methods
      .createAddressLookupTable({ recentSlot, id })
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    return { userAddressLookupTable, addressLookupTable };
  }

  it("creates an address lookup table", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    const account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );

    expect(account.signer.toString()).to.equal(signer.toString());
    expect(account.addressLookupTable.toString()).to.equal(
      addressLookupTable.toString()
    );
    expect(account.id.toNumber()).to.equal(id.toNumber());
    expect(account.size.toNumber()).to.equal(0);
    expect(account.lastUpdatedSlot.toNumber()).to.be.greaterThan(0);
  });

  it("rejects extend during cooldown period (LutNotReady)", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    const addr1 = PublicKey.unique();

    try {
      await program.methods
        .extendAddressLookupTable()
        .accountsStrict({
          signer,
          systemProgram: SystemProgram.programId,
          addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
          addressLookupTable,
          userAddressLookupTable,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .remainingAccounts([
          { pubkey: addr1, isSigner: false, isWritable: false },
        ])
        .rpc();

      expect.fail("Expected LutNotReady error");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.code).to.equal("LutNotReady");
    }
  });

  it("extends an address lookup table with new addresses after cooldown", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    // Wait for cooldown (15 slots ~ 6 seconds)
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();

    const tx = await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr1, isSigner: false, isWritable: false },
        { pubkey: addr2, isSigner: false, isWritable: false },
      ])
      .rpc();

    console.log("Extend LUT transaction signature:", tx);

    const updatedAccount = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );

    expect(updatedAccount.size.toNumber()).to.equal(2);
  });

  it("rejects adding only duplicate addresses (NoNewAddresses)", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    // Wait for cooldown
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();

    // First extend - should succeed
    await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr1, isSigner: false, isWritable: false },
      ])
      .rpc();

    // Wait for cooldown again
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Second extend with same address - should fail
    try {
      await program.methods
        .extendAddressLookupTable()
        .accountsStrict({
          signer,
          systemProgram: SystemProgram.programId,
          addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
          addressLookupTable,
          userAddressLookupTable,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .remainingAccounts([
          { pubkey: addr1, isSigner: false, isWritable: false },
        ])
        .rpc();

      expect.fail("Expected NoNewAddresses error");
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.code).to.equal(
        "NoNewAddresses"
      );
    }
  });

  it("size field stays in sync with addresses added", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    let account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    expect(account.size.toNumber()).to.equal(0);

    // Wait for cooldown
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Add 3 addresses
    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();
    const addr3 = PublicKey.unique();

    await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr1, isSigner: false, isWritable: false },
        { pubkey: addr2, isSigner: false, isWritable: false },
        { pubkey: addr3, isSigner: false, isWritable: false },
      ])
      .rpc();

    account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    expect(account.size.toNumber()).to.equal(3);

    // Wait for cooldown
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Add 2 more addresses
    const addr4 = PublicKey.unique();
    const addr5 = PublicKey.unique();

    await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr4, isSigner: false, isWritable: false },
        { pubkey: addr5, isSigner: false, isWritable: false },
      ])
      .rpc();

    account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    expect(account.size.toNumber()).to.equal(5);
  });

  it("filters duplicate addresses and only adds new ones", async () => {
    const id = lutId;
    lutId = lutId.addn(1);

    const { userAddressLookupTable, addressLookupTable } = await createLut(id);

    // Wait for cooldown
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const addr1 = PublicKey.unique();
    const addr2 = PublicKey.unique();

    // Add first batch
    await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr1, isSigner: false, isWritable: false },
        { pubkey: addr2, isSigner: false, isWritable: false },
      ])
      .rpc();

    let account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    expect(account.size.toNumber()).to.equal(2);

    // Wait for cooldown
    console.log("Waiting for cooldown period...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Add mix of existing and new addresses
    const addr3 = PublicKey.unique();

    await program.methods
      .extendAddressLookupTable()
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: addr1, isSigner: false, isWritable: false }, // duplicate
        { pubkey: addr3, isSigner: false, isWritable: false }, // new
      ])
      .rpc();

    account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    expect(account.size.toNumber()).to.equal(3); // Only 1 new address added
  });
});
