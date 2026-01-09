import { address, type Address, type TransactionSigner } from "@solana/kit";
import {
  AccountMeta,
  AddressLookupTableAccount,
  BlockheightBasedTransactionConfirmationStrategy,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SignatureResult,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import assert from "assert";

export async function processAndValidateTransaction(
  instructions: TransactionInstruction[],
  connection: Connection,
  signer: Keypair
) {
  const sig = await processTransaction(instructions, connection, signer);
  const txn = await connection.getParsedTransaction(sig.Signature, "confirmed");
  assert.equal(
    sig.SignatureResult.err,
    null,
    `${txn?.meta?.logMessages?.join("\n")}\n\n${JSON.stringify(sig)}`
  );
}

export declare type TxnResult = {
  Signature: string;
  SignatureResult: SignatureResult;
};

export async function processTransaction(
  instructions: TransactionInstruction[],
  connection: Connection,
  payer: Keypair,
  lookupTableAccount?: AddressLookupTableAccount
): Promise<TxnResult> {
  const blockStats = await connection.getLatestBlockhash();
  if (lookupTableAccount) {
    const messageV0 = new TransactionMessage({
      payerKey: payer.publicKey,
      recentBlockhash: blockStats.blockhash,
      instructions: instructions,
    }).compileToV0Message([lookupTableAccount]);
    const transactionV0 = new VersionedTransaction(messageV0);
    transactionV0.sign([payer]);
    const sig = await connection.sendTransaction(transactionV0);
    const strategy: BlockheightBasedTransactionConfirmationStrategy = {
      signature: sig,
      blockhash: blockStats.blockhash,
      lastValidBlockHeight: blockStats.lastValidBlockHeight,
    };
    const result = await connection.confirmTransaction(strategy, "confirmed");
    return {
      Signature: sig,
      SignatureResult: result.value,
    };
  } else {
    const tx = new Transaction();
    instructions.map((i) => tx.add(i));
    tx.recentBlockhash = blockStats.blockhash;
    tx.feePayer = payer.publicKey;
    tx.sign(payer);
    const sig = await connection.sendRawTransaction(tx.serialize(), {
      maxRetries: 3,
      preflightCommitment: "confirmed",
      skipPreflight: true,
    });
    const strategy: BlockheightBasedTransactionConfirmationStrategy = {
      signature: sig,
      blockhash: blockStats.blockhash,
      lastValidBlockHeight: blockStats.lastValidBlockHeight,
    };
    const result = await connection.confirmTransaction(strategy, "confirmed");
    return {
      Signature: sig,
      SignatureResult: result.value,
    };
  }
}

export async function accountExists(
  connection: Connection,
  pubkey: PublicKey
): Promise<boolean> {
  const account_info = await connection.getAccountInfo(pubkey, "confirmed");
  return account_info !== null;
}

export const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
  microLamports: 10_000,
});

export const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
  units: 300_000,
});

export function toAccountMeta(key: PublicKey, writeable: boolean): AccountMeta {
  return {
    pubkey: key,
    isSigner: false,
    isWritable: writeable,
  };
}

export function toTransactionSigner(pubkey: PublicKey): TransactionSigner {
  return {
    address: address(pubkey.toBase58()),
    signTransactions: async (txs) => txs,
  } as TransactionSigner;
}

export function toTransactionInstruction(instruction: {
  programAddress: Address;
  accounts: readonly { address: Address; role: number; signer?: unknown }[];
  data: Uint8Array;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: new PublicKey(instruction.programAddress),
    keys: instruction.accounts.map((acc) => ({
      pubkey: new PublicKey(acc.address),
      isSigner: acc.role >= 2,
      isWritable: acc.role === 1 || acc.role === 3,
    })),
    data: Buffer.from(instruction.data),
  });
}
