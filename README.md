# YolkFi Protocol (`yolkfi-protocol`)

**Verifiable revenue sharing on a live Romanian egg-production business, settled in EURC on Stellar.**

YolkFi turns individual production lots of an operating poultry farm (**A.T. Grup Prod Impex SRL**, CUI RO9971244, est. 1997) into compliance-gated Stellar tokens (`LotShare`, SEP-41 / SEP-57) whose holders receive a share of that lot's attested net revenue in EURC. Every euro distributed is traceable back to a signed production report from the farm's grading line, and any third party can re-derive every distribution from public ledger data using the `yolkfi-verify` CLI.

---

## Live Stellar Testnet Deployment

| Contract | Stellar Testnet Contract ID | WASM SHA-256 | Explorer |
| :--- | :--- | :--- | :--- |
| **`FlockRegistry`** | `CCGUMRHRFOCER7UDH3NZ32OL4E5LXN7AFK3HSXJ3IBNOSC4EIGFQHKLU` | `74DEBA52E26BAA9361D74C699B85F190995E0D4248E5768EA8DD4EC6CFC641AA` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCGUMRHRFOCER7UDH3NZ32OL4E5LXN7AFK3HSXJ3IBNOSC4EIGFQHKLU) |
| **`RevenueDistributor`** | `CCZ2ZXWXZEYND66ARY5BOWDZZ63D5H7GS3N7H2LRII2JADU35BZ3FY22` | `95ABC2364627D8BC96E8DD77B8ABCEB542BFD5B5445E3DCD371988976C77E056` | [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCZ2ZXWXZEYND66ARY5BOWDZZ63D5H7GS3N7H2LRII2JADU35BZ3FY22) |

Full deployment details, quorum addresses, and registered Season 1 test lots (`LOTS1A`, `LOTS1B`, `LOTS1C`) are documented in **[`docs/addresses.md`](./docs/addresses.md)**.

---

## Repository Structure

```text
yolkfi-protocol/
├── Cargo.toml                         # Soroban Rust workspace manifest
├── ARCHITECTURE.md                    # System design & Soroban contract specification
├── MILESTONES.md                      # SCF Build Award delivery schedule & verification criteria
├── docs/
│   └── addresses.md                   # Deployed Stellar Testnet contract IDs, WASM hashes & lots
└── contracts/
    ├── flock-registry/                # Lot registration, Placed -> Laying -> Molting -> Depopulated FSM, doc_root anchoring & allocation gating
    └── revenue-distributor/           # O(1) pull-based cumulative EURC revenue index (acc_per_share, scale 10^12) & loss-reserve buffer
```

### Build & Test

```bash
# Run the Soroban contract unit test suite
cargo test --workspace

# Compile optimized WASM binaries
cargo build --workspace --target wasm32-unknown-unknown --release
```

---

## Documentation

- **[Technical Architecture (`ARCHITECTURE.md`)](./ARCHITECTURE.md)** — Full system overview, Soroban smart contract specifications (`FlockRegistry`, `LotShare`, `IdentityRegistry`, `ComplianceModules`, `ProductionOracle`, `RevenueDistributor`, `Treasury`), SEP-41 / SEP-57 compliance hooks, pull-based cumulative revenue index, k-of-n attestation set, STRIDE threat model, and `yolkfi-verify` reconciliation CLI.
- **[Delivery Milestones (`MILESTONES.md`)](./MILESTONES.md)** — Tranche 1 (MVP), Tranche 2 (Testnet), and Tranche 3 (Mainnet) scope, completion criteria, and delivery schedule.
- **[Official Documentation (`docs.yolkfi.com`)](https://docs.yolkfi.com/)** — Project overview, operating partner traction (A.T. Grup FY2025 filings), integrations, and governance.

*(Mirrored copies are also included at [`ARCHITECTURE_YolkFi.md`](./ARCHITECTURE_YolkFi.md) and [`MILESTONES_YolkFi.md`](./MILESTONES_YolkFi.md).)*

---

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).
