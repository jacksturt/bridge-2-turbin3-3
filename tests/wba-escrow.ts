import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WbaEscrow } from "../target/types/wba_escrow";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";
import { randomBytes } from "crypto";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddressSync,
  createMint,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import base58 from "bs58";

describe("wba-escrow", () => {
  // Configure the client to use the local cluster.

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.WbaEscrow as Program<WbaEscrow>;
  const connection = anchor.getProvider().connection;

  const maker = Keypair.generate();
  const taker = Keypair.generate();
  let mintX: PublicKey;
  let mintY: PublicKey;
  let makerAtaX: PublicKey;
  let makerAtaY: PublicKey;

  let takerAtaX: PublicKey;
  let takerAtaY: PublicKey;
  const token_decimals = 1_000_000;

  const seed = new anchor.BN(randomBytes(8));
  const escrow = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toBuffer("le", 8)],
    program.programId
  )[0];
  let vault: PublicKey;
  const amount_x = new anchor.BN(100);
  const amount_y = new anchor.BN(20);

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  it("Airdrop", async () => {
    await connection
      .requestAirdrop(maker.publicKey, LAMPORTS_PER_SOL * 10)
      .then(confirm)
      .then(log);
    await connection
      .requestAirdrop(taker.publicKey, LAMPORTS_PER_SOL * 10)
      .then(confirm)
      .then(log);
  });

  it("Create mints and mint to", async () => {
    mintX = await createMint(connection, maker, maker.publicKey, null, 6);

    mintY = await createMint(connection, taker, taker.publicKey, null, 6);

    makerAtaX = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        maker,
        mintX,
        maker.publicKey
      )
    ).address;
    makerAtaY = getAssociatedTokenAddressSync(mintY, maker.publicKey);
    vault = getAssociatedTokenAddressSync(mintX, escrow, true);

    console.log(`Your mint_x ata is: ${makerAtaX.toBase58()}`);
    // Mint to ATA
    await mintTo(
      connection,
      maker,
      mintX,
      makerAtaX,
      maker.publicKey,
      token_decimals
    )
      .then(confirm)
      .then(log);

    takerAtaY = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        taker,
        mintY,
        taker.publicKey
      )
    ).address;

    takerAtaX = getAssociatedTokenAddressSync(mintX, taker.publicKey);

    console.log(`Your mint_y ata is: ${makerAtaX.toBase58()}`);
    // Mint to ATA
    await mintTo(
      connection,
      taker,
      mintY,
      takerAtaY,
      taker.publicKey,
      token_decimals
    )
      .then(confirm)
      .then(log);
  });

  it("Can Make", async () => {
    const tx = program.methods
      .make(seed, amount_x, amount_y)
      .accounts({
        maker: maker.publicKey,
        makerAtaX,
        makerAtaY,
        mintX,
        mintY,
        escrow,
        vault,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc({ skipPreflight: true })
      .then(confirm)
      .then(log);
  });

  xit("Can Refund", async () => {
    const tx = program.methods
      .refund()
      .accounts({
        maker: maker.publicKey,
        makerAtaX,
        mintX,
        escrow,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc({ skipPreflight: true })
      .then(confirm)
      .then(log);
  });

  it("Can Take", async () => {
    const tx = program.methods
      .take()
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        makerAtaX,
        makerAtaY,
        takerAtaX,
        takerAtaY,
        mintX,
        mintY,
        escrow,
        vault,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc({ skipPreflight: true })
      .then(confirm)
      .then(log);
  });
});
