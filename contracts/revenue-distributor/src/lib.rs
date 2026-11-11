#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

pub const SCALE: i128 = 1_000_000_000_000; // 10^12

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LotAccrualState {
    pub lot_id: Symbol,
    pub minted_supply: i128,
    pub reserve_bps: u32,
    pub acc_per_share: i128,
    pub reserve_balance: i128,
    pub carry_forward_deficit: i128,
    pub undistributed_dust: i128,
    pub total_deposited: i128,
    pub total_claimed: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderPosition {
    pub balance: i128,
    pub snapshot: i128,
    pub settled_unclaimed: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Lot(Symbol),
    Holder(Symbol, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DistributorError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    LotAlreadyExists = 3,
    LotNotFound = 4,
    ZeroMintedSupply = 5,
    ArithmeticOverflow = 6,
    NothingToClaim = 7,
    InvalidAmount = 8,
}

#[contract]
pub struct RevenueDistributor;

#[contractimpl]
impl RevenueDistributor {
    pub fn initialize(env: Env, admin: Address) -> Result<(), DistributorError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(DistributorError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn init_lot(
        env: Env,
        lot_id: Symbol,
        reserve_bps: u32,
    ) -> Result<LotAccrualState, DistributorError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DistributorError::NotInitialized)?;
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Lot(lot_id.clone())) {
            return Err(DistributorError::LotAlreadyExists);
        }

        let state = LotAccrualState {
            lot_id: lot_id.clone(),
            minted_supply: 0,
            reserve_bps,
            acc_per_share: 0,
            reserve_balance: 0,
            carry_forward_deficit: 0,
            undistributed_dust: 0,
            total_deposited: 0,
            total_claimed: 0,
        };
        env.storage().persistent().set(&DataKey::Lot(lot_id), &state);
        Ok(state)
    }

    /// Deposit a period's net revenue (or negative loss) in stroops.
    /// Updates `acc_per_share` in O(1) time regardless of holder count.
    pub fn deposit_revenue(
        env: Env,
        lot_id: Symbol,
        net_revenue: i128,
    ) -> Result<LotAccrualState, DistributorError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DistributorError::NotInitialized)?;
        admin.require_auth();

        let mut lot: LotAccrualState = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(DistributorError::LotNotFound)?;

        if lot.minted_supply <= 0 {
            return Err(DistributorError::ZeroMintedSupply);
        }

        // Zero or loss-making period: absorbed by reserve buffer + carry-forward deficit.
        // `acc_per_share` never decreases.
        if net_revenue <= 0 {
            let loss = net_revenue.checked_neg().ok_or(DistributorError::ArithmeticOverflow)?;
            if loss <= lot.reserve_balance {
                lot.reserve_balance -= loss;
            } else {
                let uncovered = loss - lot.reserve_balance;
                lot.reserve_balance = 0;
                lot.carry_forward_deficit = lot
                    .carry_forward_deficit
                    .checked_add(uncovered)
                    .ok_or(DistributorError::ArithmeticOverflow)?;
            }
            env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
            env.events().publish((symbol_short!("loss_prd"), lot_id), net_revenue);
            return Ok(lot);
        }

        // Positive net revenue: first offset any carried-forward deficit
        let mut distributable = net_revenue;
        if lot.carry_forward_deficit > 0 {
            if distributable <= lot.carry_forward_deficit {
                lot.carry_forward_deficit -= distributable;
                env.storage().persistent().set(&DataKey::Lot(lot_id), &lot);
                return Ok(lot);
            } else {
                distributable -= lot.carry_forward_deficit;
                lot.carry_forward_deficit = 0;
            }
        }

        let reserve_cut = distributable
            .checked_mul(lot.reserve_bps as i128)
            .ok_or(DistributorError::ArithmeticOverflow)?
            .checked_div(10_000)
            .ok_or(DistributorError::ArithmeticOverflow)?;

        let holder_pool = distributable - reserve_cut;
        let scaled_pool = holder_pool
            .checked_mul(SCALE)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        let increment = scaled_pool
            .checked_div(lot.minted_supply)
            .ok_or(DistributorError::ArithmeticOverflow)?;

        let allocated = increment
            .checked_mul(lot.minted_supply)
            .ok_or(DistributorError::ArithmeticOverflow)?
            .checked_div(SCALE)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        let dust = holder_pool - allocated;

        lot.reserve_balance = lot
            .reserve_balance
            .checked_add(reserve_cut)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        lot.acc_per_share = lot
            .acc_per_share
            .checked_add(increment)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        lot.undistributed_dust = lot
            .undistributed_dust
            .checked_add(dust)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        lot.total_deposited = lot
            .total_deposited
            .checked_add(distributable)
            .ok_or(DistributorError::ArithmeticOverflow)?;

        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events().publish((symbol_short!("rev_dep"), lot_id), holder_pool);
        Ok(lot)
    }

    /// Settle pre-operation accrual on mint (`from = None`) or transfer (`from = Some(...)`).
    pub fn settle_balance_change(
        env: Env,
        lot_id: Symbol,
        from: Option<Address>,
        to: Address,
        amount: i128,
    ) -> Result<(), DistributorError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(DistributorError::NotInitialized)?;
        admin.require_auth();

        if amount <= 0 {
            return Err(DistributorError::InvalidAmount);
        }

        let mut lot: LotAccrualState = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(DistributorError::LotNotFound)?;

        if let Some(sender) = from {
            let mut sender_pos = Self::load_position(&env, &lot_id, &sender, lot.acc_per_share);
            if sender_pos.balance < amount {
                return Err(DistributorError::InvalidAmount);
            }
            let delta = Self::compute_pending(sender_pos.balance, lot.acc_per_share, sender_pos.snapshot)?;
            sender_pos.settled_unclaimed = sender_pos
                .settled_unclaimed
                .checked_add(delta)
                .ok_or(DistributorError::ArithmeticOverflow)?;
            sender_pos.balance -= amount;
            sender_pos.snapshot = lot.acc_per_share;
            env.storage()
                .persistent()
                .set(&DataKey::Holder(lot_id.clone(), sender), &sender_pos);
        } else {
            // Primary mint increases minted_supply
            lot.minted_supply = lot
                .minted_supply
                .checked_add(amount)
                .ok_or(DistributorError::ArithmeticOverflow)?;
            env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        }

        let mut recv_pos = Self::load_position(&env, &lot_id, &to, lot.acc_per_share);
        let recv_delta = Self::compute_pending(recv_pos.balance, lot.acc_per_share, recv_pos.snapshot)?;
        recv_pos.settled_unclaimed = recv_pos
            .settled_unclaimed
            .checked_add(recv_delta)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        recv_pos.balance = recv_pos
            .balance
            .checked_add(amount)
            .ok_or(DistributorError::ArithmeticOverflow)?;
        recv_pos.snapshot = lot.acc_per_share;
        env.storage()
            .persistent()
            .set(&DataKey::Holder(lot_id, to), &recv_pos);

        Ok(())
    }

    /// View total claimable EURC (in stroops) for a holder on a lot.
    pub fn accrued(env: Env, lot_id: Symbol, holder: Address) -> Result<i128, DistributorError> {
        let lot: LotAccrualState = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(DistributorError::LotNotFound)?;
        let pos = Self::load_position(&env, &lot_id, &holder, lot.acc_per_share);
        let pending = Self::compute_pending(pos.balance, lot.acc_per_share, pos.snapshot)?;
        pos.settled_unclaimed
            .checked_add(pending)
            .ok_or(DistributorError::ArithmeticOverflow)
    }

    /// Pull-based claim of accrued EURC for a holder.
    pub fn claim(env: Env, lot_id: Symbol, holder: Address) -> Result<i128, DistributorError> {
        holder.require_auth();

        let mut lot: LotAccrualState = env
            .storage()
            .persistent()
            .get(&DataKey::Lot(lot_id.clone()))
            .ok_or(DistributorError::LotNotFound)?;
        let mut pos = Self::load_position(&env, &lot_id, &holder, lot.acc_per_share);

        let pending = Self::compute_pending(pos.balance, lot.acc_per_share, pos.snapshot)?;
        let claimable = pos
            .settled_unclaimed
            .checked_add(pending)
            .ok_or(DistributorError::ArithmeticOverflow)?;

        if claimable <= 0 {
            return Err(DistributorError::NothingToClaim);
        }

        pos.settled_unclaimed = 0;
        pos.snapshot = lot.acc_per_share;
        lot.total_claimed = lot
            .total_claimed
            .checked_add(claimable)
            .ok_or(DistributorError::ArithmeticOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Holder(lot_id.clone(), holder.clone()), &pos);
        env.storage().persistent().set(&DataKey::Lot(lot_id.clone()), &lot);
        env.events().publish((symbol_short!("claimed"), lot_id, holder), claimable);
        Ok(claimable)
    }

    pub fn lot_state(env: Env, lot_id: Symbol) -> Result<LotAccrualState, DistributorError> {
        env.storage()
            .persistent()
            .get(&DataKey::Lot(lot_id))
            .ok_or(DistributorError::LotNotFound)
    }

    fn load_position(env: &Env, lot_id: &Symbol, holder: &Address, current_acc: i128) -> HolderPosition {
        env.storage()
            .persistent()
            .get(&DataKey::Holder(lot_id.clone(), holder.clone()))
            .unwrap_or(HolderPosition {
                balance: 0,
                snapshot: current_acc,
                settled_unclaimed: 0,
            })
    }

    fn compute_pending(balance: i128, acc_per_share: i128, snapshot: i128) -> Result<i128, DistributorError> {
        if balance <= 0 || acc_per_share <= snapshot {
            return Ok(0);
        }
        let diff = acc_per_share - snapshot;
        balance
            .checked_mul(diff)
            .ok_or(DistributorError::ArithmeticOverflow)?
            .checked_div(SCALE)
            .ok_or(DistributorError::ArithmeticOverflow)
    }
}

#[cfg(test)]
mod test;
