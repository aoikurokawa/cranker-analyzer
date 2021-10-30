import Wallet from "@project-serum/sol-wallet-adapter";
import {
  Connection,
  SystemProgram,
  Transaction,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { deserialize, serialize } from "borsh";

const cluster = "https://api/devnet.solana.com";
const connection = new Connection(cluster, "confirmed");
const wallet = new Wallet("https://www.sollet.io", cluster);
const programId = new PublicKey("AMThEjMzMdLBzYKnMXZVLLfTUhhCMxfrQ3L3xDsTaHdG");

export async function setPayerAndBlockhashTransaction(instructions: any) {
    const transaction = new Transaction();
    instructions.forEach((element: any) => {
        transaction.add(element);
    });
    transaction.feePayer = wallet.publicKey!;
    let hash = await connection.getRecentBlockhash();
    transaction.recentBlockhash = hash.blockhash;
    return transaction;
}
