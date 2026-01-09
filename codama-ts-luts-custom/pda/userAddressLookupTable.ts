import { PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { USER_ADDRESS_LOOKUP_TABLE_SEED } from "../constants";
import { LUTS_PROGRAM_ADDRESS } from "../../codama-ts-luts";

export function getUserAddressLookupTableAddress(
  signer: PublicKey,
  id: bigint | number
): [PublicKey, number] {
  const idBuffer = Buffer.alloc(8);
  idBuffer.writeBigUInt64LE(BigInt(id));

  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(USER_ADDRESS_LOOKUP_TABLE_SEED)),
      signer.toBuffer(),
      idBuffer,
    ],
    new PublicKey(LUTS_PROGRAM_ADDRESS)
  );
}
