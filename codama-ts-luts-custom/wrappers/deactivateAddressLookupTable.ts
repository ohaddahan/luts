import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { getDeactivateAddressLookupTableInstruction } from "../../codama-ts-luts";
import { toTransactionInstruction, toTransactionSigner } from "../utils";
import { address } from "@solana/kit";

export type BuildDeactivateAddressLookupTableInput = {
  signer: PublicKey;
  addressLookupTable: PublicKey;
  userAddressLookupTable: PublicKey;
};

export function buildDeactivateAddressLookupTableInstruction({
  signer,
  addressLookupTable,
  userAddressLookupTable,
}: BuildDeactivateAddressLookupTableInput): TransactionInstruction {
  const ix = getDeactivateAddressLookupTableInstruction({
    signer: toTransactionSigner(signer),
    addressLookupTable: address(addressLookupTable.toBase58()),
    userAddressLookupTable: address(userAddressLookupTable.toBase58()),
  });

  return toTransactionInstruction(
    ix as unknown as Parameters<typeof toTransactionInstruction>[0]
  );
}
