## Jito MEV Bot

## CLI

### Get Config

```bash
cargo r --bin  jito-tip-distribution-cli -- \
    --rpc-url  https://api.devnet.solana.com \
    --program-id 3vgVYgJxqFKF2cFYHV4GPBUnLynCJYmKizq9DRmZmTUf \
    --keypair-path ~/.config/solana/id.json \
    get-config
```

### Initialize Tip Distribution Account

```bash
cargo r --bin  jito-tip-distribution-cli -- \
    --rpc-url  https://api.devnet.solana.com \
    --program-id 3vgVYgJxqFKF2cFYHV4GPBUnLynCJYmKizq9DRmZmTUf \
    --keypair-path ~/.config/solana/id.json \
    initialize-tip-distribution-account \
    --vote-account 8QyvcGJuZ55HjhqwR3uSqsyziww41hDV4osDEGMER2tc \
    --merkle-root-upload-authority 8QyvcGJuZ55HjhqwR3uSqsyziww41hDV4osDEGMER2tc \
    --validator-commission-bps 30
```

### Upload Merkle Root

```bash
cargo r --bin  jito-tip-distribution-cli -- \
    --rpc-url  https://api.devnet.solana.com \
    --program-id 3vgVYgJxqFKF2cFYHV4GPBUnLynCJYmKizq9DRmZmTUf \
    --keypair-path ~/.config/solana/id.json \
    upload-merkle-root \
    --vote-account 8QyvcGJuZ55HjhqwR3uSqsyziww41hDV4osDEGMER2tc \
    --root "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32" \
    --max-total-claim 10 \
    --max-num-nodes 10
```



## Libraries
- [mpl-token-metadata](https://github.com/metaplex-foundation/mpl-token-metadata/tree/main)
