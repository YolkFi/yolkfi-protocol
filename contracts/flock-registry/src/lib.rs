#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LotStatus {
    Placed = 0,
    Laying = 1,
    Molting = 2,
    Depopulated = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LotParams {
    pub lot_id: Symbol,
    pub barn_id: Symbol,
    pub breed_code: Symbol,
    pub placement_date: u64,
    pub headcount: u32,
    pub expected_cycle_end: u64,
    pub share_supply: i128,
    pub minted_supply: i128,
    pub reserve_bps: u32,
    pub doc_root: BytesN<32>,
    pub status: LotStatus,
    pub terminated_early: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuorumConfig {
    pub farm_operator: Address,
    pub protocol_signer: Address,
    pub independent_attestor: Address,
    pub operator_weight: u32,
    pub protocol_weight: u32,
    pub attestor_weight: u32,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Quorum,
    Lot(Symbol),
    Allocation(Symbol, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    LotAlreadyExists = 3,
    LotNotFound = 4,
    InvalidStatusTransition = 5,
    InvalidSupplyOrReserve = 6,
    SupplyExceeded = 7,
    InsufficientAllocation = 8,
    LotTerminatedOrClosed = 9,
    InvalidQuorumWeights = 10,
}

#[contract]
pub struct FlockRegistry;

#[contractimpl]
impl FlockRegistry {
    /// Initialise `FlockRegistry` with the admin address and the 3-party attestation quorum
    /// (weights 1 / 1 / 2, threshold 3 so the independent attestor is mandatory).
    pub fn initialize(
        env: Env,
        admin: Address,
        farm_operator: Address,
        protocol_signer: Address,
        independent_attestor: Address,
    ) -> Result<(), RegistryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RegistryError::AlreadyInitialized);
        }
        admin.require_auth();

        let quorum = QuorumConfig {
            farm_operator,
            protocol_signer,
            independent_attestor,
            operator_weight: 1,
            protocol_weight: 1,
            attestor_weight: 2,
            threshold: 3,
        };

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Quorum, &quorum);
        Ok(())
    }

    /// Register a new flock lot in `Placed` status with its anchored document root (`doc_root`).
    #[allow(clippy::too_many_arguments)]
    pub fn register_lot(
        env: Env,
        lot_id: Symbol,
        barn_id: Symbol,
        breed_code: Symbol,
        placement_date: u64,
        headcount: u32,
        expected_cycle_end: u64,
        share_supply: i128,
        reserve_bps: u32,
        doc_root: BytesN<32>,
    ) -> Result<LotParams, RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Lot(lot_id.clone())) {
            return Err(RegistryError::LotAlreadyExists);
        }
        if share_supply <= 0 || headcount == 0 || reserve_bps > 5_000 || expected_cycle_end <= placement_date {
            return Err(RegistryError::InvalidSupplyOrReserve);
        }

        let params = LotParams {
            lot_id: lot_id.clone(),
            barn_id,
            breed_code,
            placement_date,
            headcount,
            expected_cycle_end,
            share_supply,
            minted_supply: 0,
            reserve_bps,
            doc_root,
            status: LotStatus::Placed,
            terminated_early: false,
        };

        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &params);
        env.events().publish((symbol_short!("lot_reg"), lot_id), params.clone());
        Ok(params)
    }

    /// Transition a lot through the `Placed -> Laying <-> Molting -> Depopulated` state machine.
    /// Requires the mandatory independent attestor's authorisation.
    pub fn set_status(
        env: Env,
        lot_id: Symbol,
        new_status: LotStatus,
    ) -> Result<LotStatus, RegistryError> {
        let quorum: QuorumConfig = env
            .storage()
            .instance()
            .get(&DataKey::Quorum)
            .ok_or(RegistryError::NotInitialized)?;
        quorum.independent_attestor.require_auth();

        let mut lot: LotParams = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(RegistryError::LotNotFound)?;

        let valid = matches!(
            (lot.status, new_status),
            (LotStatus::Placed, LotStatus::Laying)
                | (LotStatus::Laying, LotStatus::Molting)
                | (LotStatus::Molting, LotStatus::Laying)
                | (LotStatus::Laying, LotStatus::Depopulated)
                | (LotStatus::Molting, LotStatus::Depopulated)
        );

        if !valid {
            return Err(RegistryError::InvalidStatusTransition);
        }

        lot.status = new_status;
        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events().publish((symbol_short!("status"), lot_id), new_status);
        Ok(new_status)
    }

    /// Update the anchored `doc_root` hash for supplementary veterinary or settlement records.
    pub fn anchor_docs(
        env: Env,
        lot_id: Symbol,
        new_doc_root: BytesN<32>,
    ) -> Result<(), RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        admin.require_auth();

        let mut lot: LotParams = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(RegistryError::LotNotFound)?;

        lot.doc_root = new_doc_root.clone();
        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events().publish((symbol_short!("doc_root"), lot_id), new_doc_root);
        Ok(())
    }

    /// Record an authorised primary subscription allocation for an investor, bounded by `share_supply`.
    pub fn authorise_allocation(
        env: Env,
        lot_id: Symbol,
        investor: Address,
        amount: i128,
    ) -> Result<i128, RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        admin.require_auth();

        if amount <= 0 {
            return Err(RegistryError::InvalidSupplyOrReserve);
        }

        let lot: LotParams = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(RegistryError::LotNotFound)?;

        if lot.status == LotStatus::Depopulated || lot.terminated_early {
            return Err(RegistryError::LotTerminatedOrClosed);
        }

        let current_alloc: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allocation(lot_id.clone(), investor.clone()))
            .unwrap_or(0);

        let new_alloc = current_alloc
            .checked_add(amount)
            .ok_or(RegistryError::SupplyExceeded)?;

        if lot
            .minted_supply
            .checked_add(new_alloc)
            .ok_or(RegistryError::SupplyExceeded)?
            > lot.share_supply
        {
            return Err(RegistryError::SupplyExceeded);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Allocation(lot_id.clone(), investor.clone()), &new_alloc);
        env.events()
            .publish((symbol_short!("alloc"), lot_id, investor), new_alloc);
        Ok(new_alloc)
    }

    /// Consume an authorised allocation to mint `LotShare` units, strictly bounded by `share_supply`.
    pub fn mint_allocated(
        env: Env,
        lot_id: Symbol,
        investor: Address,
        amount: i128,
    ) -> Result<i128, RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        admin.require_auth();

        if amount <= 0 {
            return Err(RegistryError::InvalidSupplyOrReserve);
        }

        let mut lot: LotParams = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(RegistryError::LotNotFound)?;

        if lot.status == LotStatus::Depopulated || lot.terminated_early {
            return Err(RegistryError::LotTerminatedOrClosed);
        }

        let current_alloc: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Allocation(lot_id.clone(), investor.clone()))
            .unwrap_or(0);

        if amount > current_alloc {
            return Err(RegistryError::InsufficientAllocation);
        }

        let new_minted = lot
            .minted_supply
            .checked_add(amount)
            .ok_or(RegistryError::SupplyExceeded)?;
        if new_minted > lot.share_supply {
            return Err(RegistryError::SupplyExceeded);
        }

        let remaining_alloc = current_alloc - amount;
        lot.minted_supply = new_minted;

        env.storage()
            .persistent()
            .set(&DataKey::Allocation(lot_id.clone(), investor.clone()), &remaining_alloc);
        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events()
            .publish((symbol_short!("minted"), lot_id, investor), amount);
        Ok(new_minted)
    }

    /// Early termination path for sanitary depopulation or total loss: transitions lot to `Depopulated`,
    /// marks `terminated_early = true`, and freezes further issuance.
    pub fn early_terminate(env: Env, lot_id: Symbol) -> Result<LotParams, RegistryError> {
        let quorum: QuorumConfig = env
            .storage()
            .instance()
            .get(&DataKey::Quorum)
            .ok_or(RegistryError::NotInitialized)?;
        quorum.independent_attestor.require_auth();

        let mut lot: LotParams = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(RegistryError::LotNotFound)?;

        if lot.status == LotStatus::Depopulated {
            return Err(RegistryError::LotTerminatedOrClosed);
        }

        lot.status = LotStatus::Depopulated;
        lot.terminated_early = true;
        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events().publish((symbol_short!("term"), lot_id), true);
        Ok(lot)
    }

    /// Read parameters and current state of a registered lot.
    pub fn lot_params(env: Env, lot_id: Symbol) -> Result<LotParams, RegistryError> {
        env.storage()
            .persistent()
            .get(&DataKey::Lot(lot_id))
            .ok_or(RegistryError::LotNotFound)
    }

    /// Read remaining authorised allocation for an investor on a lot.
    pub fn allocation_of(env: Env, lot_id: Symbol, investor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allocation(lot_id, investor))
            .unwrap_or(0)
    }

    /// Read the active attestation quorum configuration.
    pub fn quorum_config(env: Env) -> Result<QuorumConfig, RegistryError> {
        env.storage()
            .instance()
            .get(&DataKey::Quorum)
            .ok_or(RegistryError::NotInitialized)
    }
}

#[cfg(test)]
mod test;
