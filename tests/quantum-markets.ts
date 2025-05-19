import * as anchor from "@coral-xyz/anchor";
import { QuantumMarkets } from "../target/types/quantum_markets";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { createMint, getAssociatedTokenAddress, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { QuantumMarkets, IDL } from "../target/types/quantum_markets"
import { assert } from "chai";

describe("quantum-markets", () => {
  // 1) point at local validator
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.QuantumMarkets as anchor.Program<QuantumMarkets>;

  // 2) sync keypairs
  const payer = provider.wallet.payer as Keypair;
  let rewardMint: PublicKey;
  let globalPda: PublicKey, globalBump: number;

  // derive the “global” state PDA
  before(async () => {
    [globalPda, globalBump] =
      PublicKey.findProgramAddressSync([Buffer.from("global")], program.programId);

    // airdrop if needed
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 2e9)
    );
  });

  it("Creates a USDC‐like mint", async () => {
    // 6 decimals, payer is mint authority
    rewardMint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      6
    );
    assert.ok(rewardMint);
  });

})
