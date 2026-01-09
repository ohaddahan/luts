import { PublicKey } from "@solana/web3.js";
import { ADDRESS_LOOKUP_TABLE_PROGRAM_ID } from "../constants";

export function deriveAddressLookupTableAddress(
  authority: PublicKey,
  recentSlot: bigint | number
): [PublicKey, number] {
  const slotBuffer = Buffer.alloc(8);
  slotBuffer.writeBigUInt64LE(BigInt(recentSlot));

  return PublicKey.findProgramAddressSync(
    [authority.toBuffer(), slotBuffer],
    new PublicKey(ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
  );
}
