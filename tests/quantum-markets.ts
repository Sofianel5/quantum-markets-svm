import * as anchor from "@coral-xyz/anchor";
import { QuantumMarkets } from "../target/types/quantum_markets";
import { PublicKey, Keypair, SystemProgram, ComputeBudgetProgram } from "@solana/web3.js";
import { createMint, createAssociatedTokenAccount, getAssociatedTokenAddress, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, mintTo } from "@solana/spl-token";
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
  let userAta: PublicKey;
  let marketVault: PublicKey;
  let depositPda: PublicKey;
  let propCtrPda: PublicKey;
  let proposalPda: PublicKey;
  let vusdMint: PublicKey, yesMint: PublicKey, noMint: PublicKey;
  let userVusdAta: PublicKey, userYesAta: PublicKey, userNoAta: PublicKey;
  let proposalAuthPda: PublicKey;
  const DECIMALS = 1_000_000;

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
        /* minDeposit */ new anchor.BN(1000 * DECIMALS),
        /* strikePrice */ new anchor.BN(42 * DECIMALS),
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
    assert.equal(m.minDeposit.toNumber(), 1000 * DECIMALS);
    assert.equal(m.strikePrice.toNumber(), 42 * DECIMALS);
    assert.equal(m.title, "My First Market");
    assert.deepEqual(m.status, { open: {} });
    assert.ok(m.marketToken.equals(rewardMint));
    assert.ok(m.resolver.equals(payer.publicKey));
  });

  it("Deposits into the market", async () => {
    // Associated-token address for payer’s reward tokens
    userAta = await getAssociatedTokenAddress(rewardMint, payer.publicKey);

    if (!(await provider.connection.getAccountInfo(userAta))) {
      await createAssociatedTokenAccount(
        provider.connection,   // connection
        payer,                 // fee-payer & owner of new account
        rewardMint,            // mint
        payer.publicKey        // owner of ATA
      );
    }
    await mintTo(
      provider.connection,
      payer,
      rewardMint,
      userAta,
      payer,
      2_000 * DECIMALS /* tokens */
    );

    // PDA that stores this user’s running total for the market

    depositPda = PublicKey.findProgramAddressSync(
      [Buffer.from("deposit"), marketPda.toBuffer(), payer.publicKey.toBuffer()],
      program.programId
    )[0];

    // Vault (ATA owned by the market PDA) that will hold deposits
    marketVault = await getAssociatedTokenAddress(
      rewardMint,
      marketPda,
      true,          // “owner” is a PDA
      TOKEN_PROGRAM_ID
    );

    // Call depositToMarket
    await program.methods
      .depositToMarket(new anchor.BN(2_000 * DECIMALS))
      .accounts({
        payer:          payer.publicKey,
        rewardMint,                     // ← add this line
        userToken:      userAta,
        marketVault,
        market:         marketPda,
        depositRecord:  depositPda,
        tokenProgram:   TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram:  SystemProgram.programId,
      })
      .signers([payer])
      .rpc();


    // Assert the DepositRecord now shows 2 000
    const record = await program.account.depositRecord.fetch(depositPda);
    assert.equal(record.amount.toNumber(), 2_000 * DECIMALS);
  });

  it("Creates proposal-0 (auto-id) and mints vUSD/YES/NO", async () => {

    const idBytes = Buffer.from(Uint8Array.of(1,0,0,0,0,0,0,0));

    // derive proposal-0 PDA + mints/auth
    proposalPda = PublicKey.findProgramAddressSync(
      [Buffer.from("proposal"), idBytes],
      program.programId
    )[0];
    vusdMint = PublicKey.findProgramAddressSync(
      [Buffer.from("vusd"), idBytes],
      program.programId
    )[0];
    yesMint  = PublicKey.findProgramAddressSync(
      [Buffer.from("yes_mint"), idBytes],
      program.programId
    )[0];
    noMint   = PublicKey.findProgramAddressSync(
      [Buffer.from("no_mint"), idBytes],
      program.programId
    )[0];
    proposalAuthPda = PublicKey.findProgramAddressSync(
      [Buffer.from("proposal_auth")],
      program.programId
    )[0];

    await program.methods
      .createProposal(Buffer.from("hello-world"))
      .accounts({
        payer:              payer.publicKey,
        market:             marketPda,
        userDeposit:        depositPda,
        rewardMint,
        // freshly-derived accounts:
        vusdMint,
        yesMint,
        noMint,
        vusdVault: await getAssociatedTokenAddress(vusdMint, proposalAuthPda, true, TOKEN_PROGRAM_ID),
        yesVault:  await getAssociatedTokenAddress(yesMint,  proposalAuthPda, true, TOKEN_PROGRAM_ID),
        noVault:   await getAssociatedTokenAddress(noMint,   proposalAuthPda, true, TOKEN_PROGRAM_ID),
        userYes:   await getAssociatedTokenAddress(yesMint,  payer.publicKey),
        userNo:    await getAssociatedTokenAddress(noMint,   payer.publicKey),
        proposalAuth: proposalAuthPda,
        proposal:     proposalPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .preInstructions([
        // ask the runtime for 400k CU
        ComputeBudgetProgram.setComputeUnitLimit({ units: 2_000_000 }),
      ])
      .rpc();

    const pcfg = await program.account.proposalConfig.fetch(proposalPda);
    assert.equal(pcfg.marketId.toNumber(), 0);
  });

  it("Claims vUSD equal to deposit and receives 1 000 vUSD", async () => {
    userVusdAta = await getAssociatedTokenAddress(vusdMint, payer.publicKey);

    await program.methods
      .claimForProposal()
      .accounts({
        payer: payer.publicKey,
        proposal: proposalPda,
        market: marketPda,
        depositRecord: depositPda,
        claimRecord: PublicKey.findProgramAddressSync(
          [Buffer.from("claim"), proposalPda.toBuffer(), payer.publicKey.toBuffer()],
          program.programId
        )[0],
        vusdMint,
        userVusd: userVusdAta,
        proposalAuth: proposalAuthPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    const bal = await provider.connection.getTokenAccountBalance(userVusdAta);
    assert.equal(bal.value.uiAmount, 1000);
  });

  it("Swaps 200 vUSD → 200 YES + NO, then redeems back", async () => {
    // 1) mintYesNo
    userYesAta = await getAssociatedTokenAddress(yesMint, payer.publicKey);
    userNoAta  = await getAssociatedTokenAddress(noMint,  payer.publicKey);

    await program.methods
      .mintYesNo(new anchor.BN(200 * DECIMALS))
      .accounts({
        payer: payer.publicKey,
        proposal: proposalPda,
        vusdMint,
        proposalAuth: proposalAuthPda,
        userVusd: userVusdAta,
        vaultVusd: await getAssociatedTokenAddress(vusdMint, proposalAuthPda, true, TOKEN_PROGRAM_ID),
        yesMint,
        noMint,
        userYes: userYesAta,
        userNo: userNoAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    let yesBal = await provider.connection.getTokenAccountBalance(userYesAta);
    assert.equal(Number(yesBal.value.amount), 333_333_333 + 200_000_000);

    // 2) redeemYesNo (burn back to vUSD)
    await program.methods
      .redeemYesNo(new anchor.BN(200 * DECIMALS))
      .accounts({
        payer: payer.publicKey,
        proposal: proposalPda,
        proposalAuth: proposalAuthPda,
        yesMint,
        noMint,
        vusdMint,
        userYes: userYesAta,
        userNo: userNoAta,
        userVusd: userVusdAta,
        vaultVusd: await getAssociatedTokenAddress(vusdMint, proposalAuthPda, true, TOKEN_PROGRAM_ID),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();

    yesBal = await provider.connection.getTokenAccountBalance(userYesAta);
    assert.equal(Number(yesBal.value.amount), 333_333_333); // back to original 333.3333
    const vusdBal = await provider.connection.getTokenAccountBalance(userVusdAta);
    assert.equal(vusdBal.value.uiAmount, 1000);          // back to original 1 000
  });

})
