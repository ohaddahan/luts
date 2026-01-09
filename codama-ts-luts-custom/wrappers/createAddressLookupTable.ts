import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import {
  CreateAddressLookupTableInput,
  getCreateAddressLookupTableInstruction,
} from "../../codama-ts-luts";
import {
  getUserAddressLookupTableAddress,
  deriveAddressLookupTableAddress,
} from "../pda";
import { toTransactionInstruction, toTransactionSigner } from "../utils";
import { address } from "@solana/kit";

export type BuildCreateAddressLookupTableInput = {
  signer: PublicKey;
  recentSlot: bigint | number;
  id: bigint | number;
};

export type BuildCreateAddressLookupTableOutput = {
  instruction: TransactionInstruction;
  userAddressLookupTable: PublicKey;
  addressLookupTable: PublicKey;
};

export function buildCreateAddressLookupTableInstruction({
  signer,
  recentSlot,
  id,
}: BuildCreateAddressLookupTableInput): BuildCreateAddressLookupTableOutput {
  const [userAddressLookupTable] = getUserAddressLookupTableAddress(signer, id);
  const [addressLookupTable] = deriveAddressLookupTableAddress(
    userAddressLookupTable,
    recentSlot
  );

  const input: CreateAddressLookupTableInput = {
    signer: toTransactionSigner(signer),
    addressLookupTable: address(addressLookupTable.toBase58()),
    userAddressLookupTable: address(userAddressLookupTable.toBase58()),
    recentSlot: BigInt(recentSlot),
    id: BigInt(id),
  };

  const ix = getCreateAddressLookupTableInstruction(input);

  return {
    instruction: toTransactionInstruction(
      ix as unknown as Parameters<typeof toTransactionInstruction>[0]
    ),
    userAddressLookupTable,
    addressLookupTable,
  };
}
