import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { getExtendAddressLookupTableInstruction } from "../../codama-ts-luts";
import { toTransactionSigner } from "../utils";
import { address, AccountRole } from "@solana/kit";

export type BuildExtendAddressLookupTableInput = {
  signer: PublicKey;
  addressLookupTable: PublicKey;
  userAddressLookupTable: PublicKey;
  newAddresses: PublicKey[];
};

export function buildExtendAddressLookupTableInstruction({
  signer,
  addressLookupTable,
  userAddressLookupTable,
  newAddresses,
}: BuildExtendAddressLookupTableInput): TransactionInstruction {
  const ix = getExtendAddressLookupTableInstruction({
    signer: toTransactionSigner(signer),
    addressLookupTable: address(addressLookupTable.toBase58()),
    userAddressLookupTable: address(userAddressLookupTable.toBase58()),
  });

  const keys = ix.accounts.map((acc) => ({
    pubkey: new PublicKey(acc.address),
    isSigner:
      acc.role === AccountRole.READONLY_SIGNER ||
      acc.role === AccountRole.WRITABLE_SIGNER,
    isWritable:
      acc.role === AccountRole.WRITABLE ||
      acc.role === AccountRole.WRITABLE_SIGNER,
  }));

  for (const addr of newAddresses) {
    keys.push({
      pubkey: addr,
      isSigner: false,
      isWritable: false,
    });
  }

  return new TransactionInstruction({
    programId: new PublicKey(ix.programAddress),
    keys,
    data: Buffer.from(ix.data),
  });
}
