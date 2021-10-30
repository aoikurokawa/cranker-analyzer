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

export async function signAndSendTransaction(transaction: Transaction) {
  try {
    console.log("start signAndSendTransaction");
    let signedTrans = await wallet.signTransaction(transaction);
    let signature = await connection.sendRawTransaction(
      signedTrans.serialize()
    );
    console.log("end signAndSendTransaction");
    return signature;
  } catch (err: any) {
    console.log("signAndSendTransaction error", err);
    throw err;
  }
}

class CampaignDetails {
  constructor(properties: any) {
    Object.keys(properties).forEach((key, i) => {
      // let intkey = parseInt(key);
      this[key] = properties[key];
    });
  }

  static schema: any = new Map([
    [
      CampaignDetails,
      {
        kind: "struct",
        fields: [
          ["admin", [32]],
          ["name", "string"],
          ["description", "string"],
          ["image_link", "string"],
          ["amount_donated", "u64"],
        ],
      },
    ],
  ]);
}
