import * as anchor from "@coral-xyz/anchor";
import { QuantumMarkets } from "../target/types/quantum_markets";
import { PublicKey, Keypair } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
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
  let marketPda: PublicKey, marketBump: number;

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

  it("Initializes global state", async () => {
    await program.methods
      .initializeGlobal()
      .accounts({
        payer: payer.publicKey,
      })
      .rpc();

    const g = await program.account.globalState.fetch(globalPda);
    assert.equal(g.nextId.toNumber(), 0);
  });

  it("Creates the first market", async () => {
    // derive the market PDA (uses global.nextMarketId == 0)
    [marketPda, marketBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), Buffer.from(Uint8Array.of(0,0,0,0,0,0,0,0))],
      program.programId
    )


    await program.methods
      .createMarket(
        /* minDeposit */ new anchor.BN(1000),
        /* strikePrice */ new anchor.BN(42),
        /* title */ "My First Market"
      )
      .accounts({
        payer: payer.publicKey,
        rewardMint,
        resolver: payer.publicKey,   // for now just use yourself
      })
      .rpc();

    // fetch and assert
    const m = await program.account.marketConfig.fetch(marketPda);
    assert.equal(m.id.toNumber(), 0);
    assert.equal(m.minDeposit.toNumber(), 1000);
    assert.equal(m.strikePrice.toNumber(), 42);
    assert.equal(m.title, "My First Market");
    assert.deepEqual(m.status, { open: {} });
    assert.ok(m.marketToken.equals(rewardMint));
    assert.ok(m.resolver.equals(payer.publicKey));
  });



})
