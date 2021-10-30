import Wallet from "@project-serum/sol-wallet-adapter";
import {
  Connection,
  SystemProgram,
  Transaction,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { deserialize, serialize } from "borsh";
import { type } from "os";

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
    Object.keys(properties).forEach((key) => {
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

async function checkWallet() {
  if (!wallet.connected) {
    await wallet.connect();
  }
}

export async function createCampaign(name, description, image_link) {
  await checkWallet();

  const SEED = "abcdef" + Math.random().toString();
  let newAccout = await PublicKey.createWithSeed(
    wallet.publicKey,
    SEED,
    programId
  );

  let campaign = new CampaignDetails({
    name: name,
    description: description,
    image_link: image_link,
    admin: wallet.publicKey.toBuffer(),
    amount_donated: 0,
  });

  let data = serialize(CampaignDetails.schema, campaign);
  let data_to_send = new Uint8Array([0, ...data]);

  const lamports = await connection.getMinimumBalanceForRentExemption(
    data.length
  );

  const createProgramAccount = SystemProgram.createAccountWithSeed({
    fromPubkey: wallet.publicKey,
    basePubkey: wallet.publicKey,
    seed: SEED,
    newAccountPubkey: newAccout,
    lamports: lamports,
    space: data.length,
    programId: programId,
  });

  const instructionToOurProgram = new TransactionInstruction({
    keys: [
      { pubkey: newAccout, isSigner: false, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
    ],
    programId: programId,
    data: data_to_send,
  });

  const trans = await setPayerAndBlockhashTransaction([
    createProgramAccount,
    instructionToOurProgram,
  ]);
  const signature = await signAndSendTransaction(trans);

  const result = await connection.confirmTransaction(signature);
  console.log("end sendMessage", result);
}
