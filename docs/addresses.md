# Stellar Testnet & Mainnet Contract Addresses

## Stellar Testnet Deployment (Live)

- **Network:** Stellar Testnet (`Test SDF Network ; September 2015`)
- **Soroban RPC:** `https://soroban-testnet.stellar.org`
- **Horizon URL:** `https://horizon-testnet.stellar.org`

### Deployed Soroban Contracts

| Contract | Contract ID | WASM SHA-256 Hash | Stellar Expert Link |
| :--- | :--- | :--- | :--- |
| **`FlockRegistry`** | `CCGUMRHRFOCER7UDH3NZ32OL4E5LXN7AFK3HSXJ3IBNOSC4EIGFQHKLU` | `74DEBA52E26BAA9361D74C699B85F190995E0D4248E5768EA8DD4EC6CFC641AA` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCGUMRHRFOCER7UDH3NZ32OL4E5LXN7AFK3HSXJ3IBNOSC4EIGFQHKLU) |
| **`RevenueDistributor`** | `CCZ2ZXWXZEYND66ARY5BOWDZZ63D5H7GS3N7H2LRII2JADU35BZ3FY22` | `95ABC2364627D8BC96E8DD77B8ABCEB542BFD5B5445E3DCD371988976C77E056` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCZ2ZXWXZEYND66ARY5BOWDZZ63D5H7GS3N7H2LRII2JADU35BZ3FY22) |

### Registered Attestation Quorum Roles (Testnet)

| Role | Weight | Stellar Address |
| :--- | :--- | :--- |
| **Admin / Deployer** | — | `GAKZI376VZXARSJIVNU5VV7DIYJMNPGAR3IQR5GWNQDZA3AZZ25QTULV` |
| **Farm Operator (`A.T. Grup`)** | `1` | `GA4AUUNY3FZGTEQRUUMBDYZO5SJCULXZQSLQIR4JHP3FRYRHQVIPHKCU` |
| **Protocol Signer (`YolkFi`)** | `1` | `GAZTDRKQFWLMQ3YFLYTMIR5NF6PAD2GKFT26DDMMYUEFOCGIO4CQYV2Z` |
| **Independent Attestor Seat (Mandatory: CECCAR Accountant + DSVSA Veterinarian)** | `2` | `GDHNXTECIZHNCFYNQJNIB2YFVH2T443CEWYGFLIHPM24CIE2LMXGYUHD` |
| **Quorum Threshold** | **`3` of `4`** | Requires Independent Attestor Quorum (`weight 2`: CECCAR Accountant for invoiced revenue + DSVSA Veterinarian for headcount/mortality) + at least one of Farm Operator or Protocol Signer |

### Registered Season 1 Test Lots (`FlockRegistry` — `LOT-S1-VLASCA`: 30,000 Shares Total, Sept 2027 – Aug 2028 Window, 30-Day Lock-Up)

| Lot Symbol | Barn | Breed | Headcount | Share Supply | Reserve (`bps`) | `doc_root` (SHA-256) | Registration Tx Hash |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`LOTS1A`** | `BARN1` | `LOHMANN` | `10,000` | `10,000` | `1000` (10%) | `b3a1ef7ef02cfca833cfc7fa7e50e19e3d2c2bce0aa67efa892d40e6ba1adec1` | [`2831c477...48c002ea`](https://stellar.expert/explorer/testnet/tx/2831c47726f38857626b9c0fcd6224b8796d4937db76b568ce7cbeec48c002ea) |
| **`LOTS1B`** | `BARN2` | `LOHMANN` | `10,000` | `10,000` | `1000` (10%) | `6a642d530ed63eb53d507ca79487cc1b7a240a54f2ee9fcdae4738aeb9b84d5c` | [`dafcc131...09c17960`](https://stellar.expert/explorer/testnet/tx/dafcc131b75774786b84e403dbcad8d005e9806fd684d1021042799209c17960) |
| **`LOTS1C`** | `BARN3` | `HYLINE` | `10,000` | `10,000` | `1200` (12%) | `e648a9d29bca5838e1ddbfecb22dd6d95b643cd1fd9a17f25410d4b040c6e4f9` | [`865669f5...89fa41fb`](https://stellar.expert/explorer/testnet/tx/865669f5eec97db448903e21df72bfc6f7897423907e0e9dbcf5d31389fa41fb) |
