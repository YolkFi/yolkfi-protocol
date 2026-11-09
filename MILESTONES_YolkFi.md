# Milestones — YolkFi

## Verifiable Revenue Sharing on a Live Romanian Egg-Production Business, Settled in EURC on Stellar

---

**Track:** Open Track, Build Award
**Total budget:** **$121,400**
**Timeline:** December 15, 2026 to April 8, 2027 (3.75 months)
**Budget split:** T1 $40,500 (33.4%) / T2 $42,400 (34.9%) / T3 $38,500 (31.7%)
**Deliverables:** 11 (4 / 4 / 3)
**Ecosystem components used:** Stellar Wallets Kit (T1), Privy and Bridge (T2), Soroswap (T3), OpenZeppelin Stellar token and RWA libraries (T2)

Every component named below is specified in [ARCHITECTURE.md](./ARCHITECTURE.md). Each deliverable budget covers the engineering work of that deliverable plus the infrastructure it needs (Stellar RPC access, hosting and CI, monitoring tooling, the farm-side gateway hardware in 1.4, mainnet operating costs and sponsored reserves in 3.1).

**Repository access.** `github.com/YolkFi/yolkfi-protocol` is open under Apache-2.0 before the first tranche begins, with CI running on every commit. Code is not gated behind a tranche: contracts, tests and commit history can be read as they are written. The open-source deliverable in Tranche 3 is the tagged `v1.0.0` release and the independent verification toolkit, not the act of publishing.

**Prerequisites, and where one is load-bearing.** The issuing entity, the per-lot revenue purchase agreement with A.T. Grup and the regulatory qualification of the instrument are legal work carried by the company. No completion criterion below is a legal, licensing or commercial achievement. Tranches 1 and 2 are independent of that work: the whole protocol, the attestation path and the complete investor journey are proven on testnet against replayed real production data. Tranche 3 registers the first real lot, which presupposes that lot's agreement being signed; that dependency is stated at the head of Tranche 3. The farm itself (barns, birds, Big Dutchman grading line, ERP) is an operating asset of a company trading since 1997, not something this award builds.

**Dates.** Dates assume a kickoff on December 15, 2026. Working-day capacity excludes Romanian public holidays (December 25 and 26, January 1, 2, 6 and 7, January 24). Work windows overlap tranche boundaries: contract work on a later tranche begins while the previous tranche's integration and QA work is closing.

---

## Tranche 1 — MVP: Flock Registry, Accrual Engine, Identity Layer and Attestation Gateway ($40,500)

**Distribution channels.** Public repository and attestation design shared in Soroban developer channels; A.T. Grup's existing commercial partners and the farm's own network informed through YolkFi's site and social accounts.

### 1.1 `FlockRegistry`: flock-lot registry, issuance gate and early termination
**Description:** The Soroban contract that makes a physical batch of birds an on-chain object: lot registration (breed code, placement date, headcount, expected cycle end, authorised share supply), the `Placed → Laying ⇄ Molting → Depopulated` status machine, `doc_root` anchoring of signed placement, veterinary and agreement documents, deterministic deployment of the per-lot `LotShare` instance, the on-chain allocation registry that is the only path to minting and bounds it by `share_supply`, the admin / farm-operator / attestor role model enforced with `require_auth`, and the early-termination path for sanitary depopulation or total loss, which freezes transfers, releases the reserve and triggers pro-rata residual settlement (ARCHITECTURE.md §3, §4).
**How to measure completion:**
- Contract deployed on Stellar testnet with at least 3 lots registered, each carrying a distinct `doc_root`, verifiable on Stellar Expert
- Unit and fuzz test coverage above 85% on the registry crate; every invalid status transition rejected in 100% of fuzz cases
- Role separation and the minting gate proven by negative tests (mint without a recorded allocation, allocation exceeding `share_supply`, status change by a non-attestor, document anchoring by a non-admin): 100% rejection
- `LotShare` instances deployed with `salt = sha256(lot_id ‖ doc_root)`: the pre-computed address matches the deployed address for every lot, and a second deployment with the same salt is rejected on-chain
- Early-termination path tested end to end against a simulated sanitary depopulation: transfers frozen, reserve released, residual settlement computed pro rata
- Public repository with CI passing, published wasm hash and a documented contract interface
**Budget:** $9,000
**Delivery date:** December 31, 2026

### 1.2 `RevenueDistributor`: pull-based accrual engine
**Description:** The cumulative-index accounting that replaces per-holder push distribution: the `acc_per_share` accumulator (`i128`, `SCALE = 10^12`) updated once per revenue period regardless of holder count and divided by issued supply rather than the authorised ceiling; pull-based `claim`; per-holder accumulator snapshots initialised at mint and settled on every balance change; the `undistributed_dust` counter; the per-lot reserve buffer with carried-forward deficit; and the end-of-cycle residual settlement path (ARCHITECTURE.md §5).
**How to measure completion:**
- Property-test suite over at least 1,000 randomised scenarios (holder counts, transfer timings, zero-revenue and loss periods, progressive subscription) with the invariant Σ claims ≤ Σ deposits holding in 100% of runs and zero double-claims
- Simulated 12-period run with at least 20 holders including mid-period transfers and mid-cycle subscriptions: every holder's payout matches the independently computed value to the stroop
- Accrual settled on every balance-change path (mint, burn, transfer, recovery, forced transfer), proven by one test per path; a newly minted holder can claim nothing of the lot's prior history
- Benchmark report demonstrating constant-cost revenue deposit at 10, 100 and 1,000 holders
- Overflow safety: fuzz run across the documented revenue envelope with checked arithmetic and `overflow-checks` enabled, producing no arithmetic panic and no wrap
- Zero-revenue and loss periods absorbed by the lot reserve and a carried-forward deficit, with the accrual index never decreasing
- Test coverage above 85% on the distributor crate
**Budget:** $8,500
**Delivery date:** January 23, 2027

### 1.3 `IdentityRegistry`, investor onboarding and wallet binding
**Description:** The claim-based implementation of the SEP-57 `IdentityVerifier` interface (`IdentityRegistryStorage` for investor profiles and country of residence, `IdentityClaims` for per-investor cryptographic claims, `ClaimTopicsAndIssuers` as the trust registry), the off-chain claim-issuance service wrapping the KYC provider, the investor onboarding interface, and the Stellar Wallets Kit integration (Freighter, Lobstr, WalletConnect) used to bind a self-custody wallet to a verified identity claim (ARCHITECTURE.md §4, §4.2, §7).
**How to measure completion:**
- Claim lifecycle functional on testnet (issuance, update, revocation, expiry) with at least 10 test identities registered across 3 country codes
- `IdentityVerifier` negative suite (unverified address, expired claim, revoked claim, claim from an untrusted issuer): 100% rejected
- Onboarding interface completes the path from KYC hand-off to a claim readable on-chain for at least 10 test investors, with claim status visible in the interface
- Wallet binding functional through Stellar Wallets Kit for all supported connectors, with a signature-based proof of control recorded against the identity
- No personal data written on-chain, verified by inspecting every emitted event and stored key for the test identities
- Test coverage above 85% on the identity crate
**Budget:** $8,300
**Delivery date:** February 3, 2027

### 1.4 `AttestationGateway`: farm ERP and IoT connector with the attestation signer set
**Description:** The physical-to-digital bridge deployed on site at A.T. Grup: ingestion from the Big Dutchman grading line and the ERP, normalisation into the canonical `ProductionReport` schema, plausibility bounds derived from the breed lay curve and flock age, canonical serialisation, the signer service with split key custody on hardware secure elements, offline buffering, the append-only hash-linked telemetry archive, and the farm operator console. The budget includes the gateway hardware and secure elements, Stellar RPC access and hosting for the tranche (ARCHITECTURE.md §6).
**How to measure completion:**
- A multi-month replay of the farm's historical production data passed through the gateway, normalised, bound-checked and finalised on testnet via the signature quorum, with every period traceable to its archive segment
- Fault-injection suite covering 5 fault classes (forged signature, out-of-bounds report, replayed report, missing signer, malformed payload), each detected and rejected in 100% of trials
- Offline resilience: a 24-hour simulated connectivity loss produces zero data loss, and replay on reconnection yields exactly one on-chain state transition per period
- Archive head hash published, with a documented procedure by which a third party confirms any attested figure against its archive segment
- Farm operator console (lot dashboard, report review, signature status) operational and used by farm staff during the replay run
- The data-access path to the Big Dutchman and ERP systems documented in the repository (interface, transport, refresh cadence)
**Budget:** $14,700
**Delivery date:** February 6, 2027

---

## Tranche 2 — Testnet: Compliance-Gated Token, Oracle Finalisation, Investor Journey and Threat Model ($42,400)

**Distribution channels.** Testnet investor journey opened to A.T. Grup's partners and to YolkFi's waiting list; SEP-57 conformance notes shared with the Stellar regulated-asset working group and Soroban developer channels.

### 2.1 `LotShare` SEP-41 token and `ComplianceModules`
**Description:** The per-lot share token implementing the SEP-41 interface and the SEP-57 `RWAToken` trait (forced transfer, partial freeze, recovery, mint, burn), with the pluggable compliance modules behind the SEP-57 `Compliance` trait. The trait's three post-operation hooks (`transferred`, `created`, `destroyed`) receive an `AccountSnapshot` and a `TransferKind`; a module rejects by panicking, which reverts the whole transaction. Four modules: jurisdiction eligibility, holder cap per EU member state, per-investor position ceiling, and lock-up before secondary transfer, plus compliance configuration tooling with an on-chain event trail for every rule change. Built on the OpenZeppelin Stellar token and RWA libraries where they cover the standard, so the work is the YolkFi-specific modules and their integration rather than the framework itself (ARCHITECTURE.md §4.1, §4.2).
**How to measure completion:**
- SEP-41 interface conformance demonstrated by a published conformance test suite in the repository, passing in CI; every SEP-57 `RWAToken` feature exercised by a passing test
- 4 compliance modules operational, each proven to reject in 100% of its negative tests, with rejection verified to revert the entire transaction
- Holder-cap boundary tested at cap minus 1, cap and cap plus 1 for a member state: the last operation reverts in 100% of trials
- Accrual settlement verified across at least 50 transfers spanning revenue periods: the seller retains pre-transfer accrual and the buyer accrues only from the transfer onward
- Snapshot migration proven for `recover` and `forced_transfer`: neither loses or duplicates accrued value in 100% of trials
- Every compliance rule change emits an event; configuration state re-derivable from events alone
- Conformance delta against the pinned SEP-57 draft version documented in `docs/sep57-conformance.md`; test coverage above 85% across the token and compliance crates
**Budget:** $7,900
**Delivery date:** February 20, 2027

### 2.2 `ProductionOracle` and `Treasury`
**Description:** On-chain finalisation of attested production: the signature quorum with a mandatory independent attestor seat (weights 1/1/2, threshold 3, so the two aligned signers cannot finalise alone), the challenge window before finalisation, volume bounds from the breed lay curve and price bounds against a published wholesale index, and period-over-period delta guards; the `Treasury` contract holding per-lot EURC escrow under multisig disbursement authorisation with exposure limits and the reserve buffer; the three-way reconciliation report generator; and the raw-telemetry archival service that backs every attestation (ARCHITECTURE.md §4, §6).
**How to measure completion:**
- A multi-month series of monthly periods finalised on testnet from the replayed production history, each carrying a valid quorum including the attestor signature, with an explorer-verifiable finalisation transaction
- Quorum enforcement proven: the operations and protocol signers together cannot finalise a report without the attestor, in 100% of negative trials
- Volume and price bounds reject out-of-range reports in 100% of trials, including a 100% lay rate, an under-declared price outside the index band and a period-over-period delta beyond the configured guard
- Challenge window demonstrated: a challenged report is frozen before any disbursement in 100% of trials and never reaches `finalised` while a challenge is open
- Three-way reconciliation report generated automatically for each period (graded eggs, invoiced revenue, on-chain EURC deposited), reconciling within a documented tolerance with every variance identified and explained, and explorer links per deposit
- `Treasury` escrow live on testnet under multisig; disbursement without the required threshold rejected in 100% of negative tests; reserve retention matching `reserve_bps` exactly for every period
- Test coverage above 85% on the oracle and treasury crates
**Budget:** $10,900
**Delivery date:** February 27, 2027

### 2.3 Investor journey: Privy access layer, web application, Bridge EUR on-ramp and primary subscription
**Description:** Privy embedded Stellar wallets with email or social login, enclave signing and a scoped auto-signing policy for claims, presented behind the same connection interface as the Stellar Wallets Kit path from Tranche 1; the server-side-rendered investor application (portfolio, lot detail, accrual ledger, claim flow, production charts) and the event indexer feeding it; account provisioning with sponsored reserves so account creation and the EURC trustline cost the holder nothing; and euro funding over SEPA through Bridge (customer creation, KYC link, per-investor virtual IBAN, webhook ingestion) wired to the `SubscriptionOrchestrator`, which settles received EUR as EURC, checks compliance eligibility and requests issuance through `FlockRegistry`'s on-chain allocation gate. Identity is verified by YolkFi's own provider first and shared onward to the rail (ARCHITECTURE.md §7).
**How to measure completion:**
- Wallet creation to first signed claim transaction measured over at least 20 new test accounts via Privy on the project's reference environment and reported in the repository
- At least 50 fee-bumped transactions submitted on testnet with 0 requiring the holder to hold XLM, and at least 20 accounts provisioned with sponsored account and EURC trustline reserves, verifiable on explorer (fee source and reserve sponsor = platform accounts)
- Both connection paths functional: identical claim and transfer flows completed through Privy and through Stellar Wallets Kit
- Server-side rendering verified: a request with JavaScript disabled and a crawler-style fetch both return portfolio and lot content rather than an empty shell
- Bridge sandbox flow (virtual IBAN created, simulated SEPA deposit, EURC settled to the testnet treasury, `LotShare` issued) demonstrated for at least 20 consecutive subscriptions with 0 attribution failures; duplicate webhooks produce exactly one state transition in 100% of trials
- Compliance gate enforced structurally: funds from an identity that is unverified, expired or outside the lot's eligible jurisdictions are held and issuance refused in 100% of negative tests; an issuance request exceeding authorised supply reverts on-chain in 100% of trials
- Index correctness under progressive subscription verified across at least 10 staged subscriptions spanning a revenue period; end-to-end browser suite (onboarding, subscription view, claim, transfer) passing against testnet with 0 failures in CI
**Budget:** $16,300
**Delivery date:** March 6, 2027

### 2.4 STRIDE threat model, on-chain monitoring plan and dispute resolution path
**Description:** The STRIDE threat model across contracts, attestation gateway, treasury and web layer; the threat-driven contract hardening, including an emergency pause on `RevenueDistributor` and `Treasury`, caller checks on every cross-contract hook, and upgrade mechanics behind a timelock with documented storage migration; the on-chain monitoring plan following the SDF builder template and its tooling; and the dispute resolution path (challenged-report states, named resolution authority, re-attestation, accounting continuity preserved across the gap). The budget includes monitoring and log-retention tooling, RPC access and hosting for the tranche (ARCHITECTURE.md §6, §11).
**How to measure completion:**
- Threat model covers all 6 STRIDE categories with at least one identified threat and one mitigation per category, each carrying a stable identifier
- Monitoring plan complete across its 6 required sections, with a full component and on-chain address inventory; every threat identifier maps to a monitor or a documented reason for having none
- Dispute path implemented and tested: a challenged report can be resolved by re-attestation or withdrawn, and in 100% of trials the lot's accounting resumes with no gap, no double-count and no index movement during the dispute
- Upgrade attempted before the timelock elapses reverts in 100% of trials; the pause halts deposits, claims and transfers and is covered by one test per entry point; role and signer-set changes emit events and are re-derivable from them
- Monitoring plan reviewed against the SDF template checklist, with the review recorded in the repository
**Budget:** $7,300
**Delivery date:** March 13, 2027

---

## Tranche 3 — Mainnet: Production Launch, Live Lot, Exit Paths and Verification ($38,500)

**Distribution channels.** First live lot subscribed by identity-verified accounts of the issuing entity and the team; public evidence page and verification tool announced to Stellar RWA builders and to A.T. Grup's commercial network; Season 1 public offering prepared on that basis.

### 3.1 Mainnet deployment, first production lot live, monitoring in production
**Description:** Production deployment of the contract set with a reproducible build and published wasm hashes; `Treasury` mainnet multisig configuration and signer ceremony documentation; production compliance configuration; registration of the first real flock lot with its placement records, headcount and anchored documents, which presupposes that lot's revenue purchase agreement being signed and the issuing entity existing (company prerequisites, entry conditions to this tranche rather than deliverables inside it); primary issuance on mainnet to identity-verified accounts under the issuing entity's control and the team's, registered through the same `IdentityRegistry` and subject to the same `ComplianceModules` as any other holder; the monitoring implementation (event watchers, on-chain invariant checks, alert routing) cut over to production addresses; the mainnet alert drill; and the `TtlSweeper` keeping dormant holder snapshots reachable. The budget includes mainnet operating costs, sponsored account and trustline reserves, RPC access and hosting for the tranche (ARCHITECTURE.md §4, §5, §10, §11).
**How to measure completion:**
- Every contract live on mainnet with addresses and wasm hashes published in `docs/addresses.md`; a clean rebuild from the tagged source reproduces every deployed wasm hash exactly
- `Treasury` under multisig with the signer set and thresholds published and verifiable on Stellar Expert; signer ceremony documented
- First real lot registered on mainnet with `doc_root` anchoring its signed placement, veterinary and revenue-purchase documents, and lot parameters matching the physical batch
- Primary issuance executed on mainnet for the first lot, with every allocation traceable to a funding reference and a mainnet transaction hash
- Monitoring live on mainnet addresses with continuous invariant checks on Σ deposits ≥ Σ claims, reserve integrity, dust accounting and attestation freshness
- Alert drill on mainnet: at least 5 simulated anomaly classes (stale attestation, invariant breach, unauthorised role call, treasury exposure breach, storage-rent runway), each raising a routed alert in under 5 minutes, with the drill recorded in the repository
- `TtlSweeper` operating on a schedule with a reported rent runway for every persistent entry class; go-live checklist and rollback procedure published
**Budget:** $12,200
**Delivery date:** March 18, 2027

### 3.2 Holder exit paths: stablecoin routing, fiat withdrawal and permissioned secondary transfer
**Description:** Routing of the USDC/EURC leg through Soroswap for proceeds that settle in the wrong denomination, with quote and slippage controls and a price-impact guard before funds enter a lot's escrow; the off-ramp converting claimed EURC to a SEPA payout through a payout state machine, with the withdrawal interface, fee preview and payout tracking; and the compliance-checked exit path for a fixed-term instrument whose lay cycle runs 62 to 72 weeks: holder-to-holder transfer with accrual settled at transfer, the off-chain transfer intent and matching service with on-chain settlement, and the secondary transfer interface with a compliance pre-check before signing (ARCHITECTURE.md §7, §7.1).
**How to measure completion:**
- Routed stablecoin conversions executed through Soroswap with the price-impact guard blocking execution beyond the configured threshold in 100% of negative tests; failover through the fallback path demonstrated in staging with the recorded run published
- The withdrawal path demonstrated end to end, from a mainnet claim to a payout instruction issued and tracked to the rail's confirmation, with linked claim transaction hashes and payout references; fee preview matching the executed amount to the cent across all withdrawals
- Secondary transfers settled on-chain through the matching service, each passing the full `ComplianceModules` evaluation, verifiable on explorer
- Pre-check accuracy: a transfer the interface shows as eligible succeeds in 100% of cases; a transfer to an ineligible counterparty is blocked before signing and also reverts on-chain if forced
- Accrual correctness across transfers verified over at least 3 settlement periods on the reference environment plus the live period, with seller and buyer entitlements matching independently computed values within the documented rounding policy; lock-up enforced in 100% of trials
- Liquidity design note published explaining why the compliance-gated share token is not AMM-listed and how the stablecoin leg is routed instead
**Budget:** $12,400
**Delivery date:** April 3, 2027

### 3.3 First live settlement cycle, public evidence package, open-source release v1.0.0 and verification toolkit
**Description:** Execution of a complete live settlement period on mainnet (real attested production finalised through the quorum and the challenge window, net revenue deposited in EURC, accrual index updated, holders claiming); the public evidence page publishing per-period attestation hashes, explorer links and reconciliation reports, regenerated by an automated pipeline; the live-cycle operational runbook; the tagged Apache-2.0 release of the whole stack with the reproducible build pipeline and the golden-file fixture suite; `yolkfi-verify`, the command-line tool that reconstructs the protocol's distribution history from public Stellar data alone, reading the history archives for periods beyond public RPC retention and the RPC for recent ledgers; and a SEP-41 / SEP-57 integration guide for other regulated-RWA builders on Stellar. The settlement cycle is monthly, following the farm's invoicing; the first live period runs from the lot going live to the first settlement and exercises the same machinery as a full one (ARCHITECTURE.md §6, §12).
**How to measure completion:**
- The first live settlement period executed end to end on mainnet from real attested farm production, each step linked to a mainnet transaction hash
- Holder claims executed on mainnet from at least 10 distinct verified accounts, each payout matching the independently recomputed entitlement within the documented rounding policy, with the residual tracked in `undistributed_dust`
- Three-way reconciliation closed for the live period within the documented tolerance; public evidence page live and regenerated automatically by the evidence pipeline; operational runbook and on-call rotation published
- `v1.0.0` tagged and released under Apache-2.0 with CI passing and test coverage above 85% across all contract crates; architecture documentation, deployment guide and address inventory published
- `yolkfi-verify` reproduces 100% of mainnet distributions for the live period from public inputs only, reporting Σ deposits = Σ claims + unclaimed accrual + reserve + `undistributed_dust`, and exiting non-zero on each of at least 5 injected tamper scenarios
- Verification run reproduced from a fresh machine using only the public repository and public ledger data, with the recorded run published in the repository
- SEP-41 / SEP-57 regulated-RWA integration guide published in the repository
**Budget:** $13,900
**Delivery date:** April 8, 2027

---

## Summary

| Tranche | Phase | Period | Deliverables | Ecosystem components | Budget | % |
|---|---|---|---|---|---|---|
| T1 | MVP: registry, accrual, identity, attestation | December 15, 2026 to February 6, 2027 | 4 | Stellar Wallets Kit | $40,500 | 33.4% |
| T2 | Testnet: regulated token, oracle, investor journey, threat model | February 9 to March 13, 2027 | 4 | Privy, Bridge, OpenZeppelin Stellar libraries | $42,400 | 34.9% |
| T3 | Mainnet: live lot, exit paths, settlement cycle, verification | March 16 to April 8, 2027 | 3 | Soroswap | $38,500 | 31.7% |
| **Total** | | **3.75 months** | **11** | | **$121,400** | 100% |

**Evidence model.** No tranche closes on deployment. T1 closes on a multi-month replay of real farm data passing through the attestation set into on-chain accrual, with constant-cost deposits and overflow safety proven. T2 closes on a regulated token whose every compliance rule is proven to reject and to revert, an oracle quorum the two aligned signers cannot meet alone, a reconciling period series, a complete investor journey from SEPA to shares, and a dispute path that preserves accounting continuity. T3 closes on a real lot completing a real settlement period on mainnet, working exit paths, and a verification run reproduced from public data only. Every criterion resolves to an explorer link, a public repository artefact or a reproducible report.
