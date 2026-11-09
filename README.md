# YolkFi Protocol (`yolkfi-protocol`)

**Verifiable revenue sharing on a live Romanian egg-production business, settled in EURC on Stellar.**

YolkFi turns individual production lots of an operating poultry farm (**A.T. Grup Prod Impex SRL**, CUI RO9971244, est. 1997) into compliance-gated Stellar tokens (`LotShare`, SEP-41 / SEP-57) whose holders receive a share of that lot's attested net revenue in EURC. Every euro distributed is traceable back to a signed production report from the farm's grading line, and any third party can re-derive every distribution from public ledger data using the `yolkfi-verify` CLI.

---

## Documentation

- **[Technical Architecture (`ARCHITECTURE.md`)](./ARCHITECTURE.md)** — Full system overview, Soroban smart contract specifications (`FlockRegistry`, `LotShare`, `IdentityRegistry`, `ComplianceModules`, `ProductionOracle`, `RevenueDistributor`, `Treasury`), SEP-41 / SEP-57 compliance hooks, pull-based cumulative revenue index, k-of-n attestation set, STRIDE threat model, and `yolkfi-verify` reconciliation CLI.
- **[Delivery Milestones (`MILESTONES.md`)](./MILESTONES.md)** — Tranche 1 (MVP), Tranche 2 (Testnet), and Tranche 3 (Mainnet) scope, completion criteria, and delivery schedule.

*(Mirrored copies are also included at [`ARCHITECTURE_YolkFi.md`](./ARCHITECTURE_YolkFi.md) and [`MILESTONES_YolkFi.md`](./MILESTONES_YolkFi.md).)*

---

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).
