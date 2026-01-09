import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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

  function getUserAddressLookupTablePda(signer: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("UserAddressLookupTable"), signer.toBuffer()],
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

  it("creates an address lookup table", async () => {
    const recentSlot = new anchor.BN(
      await provider.connection.getSlot("finalized")
    );

    const [userAddressLookupTable] = getUserAddressLookupTablePda(signer);
    const addressLookupTable = await deriveAddressLookupTable(
      userAddressLookupTable,
      recentSlot
    );

    const tx = await program.methods
      .createAddressLookupTable({ recentSlot })
      .accountsStrict({
        signer,
        systemProgram: SystemProgram.programId,
        addressLookupTableProgram: ADDRESS_LOOKUP_TABLE_PROGRAM,
        addressLookupTable,
        userAddressLookupTable,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Create LUT transaction signature:", tx);

    const account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );

    expect(account.signer.toString()).to.equal(signer.toString());
    expect(account.addressLookupTable.toString()).to.equal(
      addressLookupTable.toString()
    );
    expect(account.accounts.length).to.equal(0);
    expect(account.lastUpdatedSlot.toNumber()).to.be.greaterThan(0);
  });

  it("extends an address lookup table with new addresses", async () => {
    const [userAddressLookupTable] = getUserAddressLookupTablePda(signer);

    const account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    const addressLookupTable = account.addressLookupTable;

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

    expect(updatedAccount.accounts.length).to.equal(2);
    expect(updatedAccount.accounts[0].toString()).to.equal(addr1.toString());
    expect(updatedAccount.accounts[1].toString()).to.equal(addr2.toString());
  });

  it("filters duplicate addresses when extending", async () => {
    const [userAddressLookupTable] = getUserAddressLookupTablePda(signer);

    const account = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );
    const addressLookupTable = account.addressLookupTable;

    const existingAddr = account.accounts[0];
    const newAddr = PublicKey.unique();

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
        { pubkey: existingAddr, isSigner: false, isWritable: false },
        { pubkey: newAddr, isSigner: false, isWritable: false },
      ])
      .rpc();

    console.log("Extend with dedup transaction signature:", tx);

    const updatedAccount = await program.account.userAddressLookupTable.fetch(
      userAddressLookupTable
    );

    expect(updatedAccount.accounts.length).to.equal(3);
  });
});
