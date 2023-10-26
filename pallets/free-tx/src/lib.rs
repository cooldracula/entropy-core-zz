#![cfg_attr(not(feature = "std"), no_std)]

//! TODO JH: This is NOT SAFE for production yet. This is an MVP and likely DoS-able.

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::IsSubType,
    };
    use frame_system::{pallet_prelude::*, RawOrigin};
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{DispatchInfoOf, SignedExtension},
        transaction_validity::{InvalidTransaction, TransactionValidityError},
    };
    use sp_staking::EraIndex;
    use sp_std::{cmp::min, fmt::Debug, prelude::*};

    pub use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_staking::Config {
        /// Pallet emits events
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Requirements for callable functions
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        // Counsil (or another) can update the number of free transactions per era
        type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // The weight information of this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Batteries and zaps are represented in the base unit of electricity: cells. One cell
    /// can be used to do one action)
    pub type Cells = u32;

    /// Shows the number of cells that were used in the previously used era
    /// ie. `latest_era` stores the `EraIndex` that the count is valid for
    #[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Eq, PartialEq)]
    pub struct ElectricityMeter {
        pub latest_era: EraIndex,
        pub count: Cells,
    }

    /// Keeps track of the electricity a user has and has spent this era
    #[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Eq, PartialEq)]
    pub struct ElectricalPanel {
        pub batteries: Cells,
        pub zaps: Cells,
        pub used: ElectricityMeter,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Maximum number of cells a user can use per era.
    ///
    /// `None`: users can use as many cells as they own.
    /// `Some(0)`: cells are disabled.
    /// `Some(n)`: users can use up to `n` cells per era
    #[pallet::storage]
    #[pallet::getter(fn max_user_electricity_usage_per_era)]
    pub type MaxUserElectricityUsagePerEra<T: Config> = StorageValue<_, Cells>;

    /// Stores the balance of batteries, zaps, and usage of electricity of a user
    #[pallet::storage]
    #[pallet::getter(fn electrical_account)]
    pub type ElectricalAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ElectricalPanel, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user spent electricity to dispatch a transaction; the account did not pay any
        /// transaction fees.
        ElectricitySpent(T::AccountId, DispatchResult),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Cell usage has been disabled
        ElectricityIsDisabled,
        /// Account has no cells left. Call the extrinsic directly or use
        /// `call_using_electricity()`
        NoCellsAvailable,
        /// Account has hit max number of cells that can be used this era
        ElectricityEraLimitReached,
    }

    // TODO: https://linear.app/entropyxyz/issue/ENT-58/free-tx-on-idle-hook-for-pruning-old-free-tx-entries
    // #[pallet::hooks]
    // impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    // 	fn on_idle(_block: T::BlockNumber, remaining_weight: Weight) -> Weight {
    //     // TODO for when we want to prune FreeCallsRemaining
    // 	}
    // }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Try to call an extrinsic using the account's available electricity.
        ///
        /// If electricity is available, a cell is used and the account will pay zero tx fees,
        /// regardless of the call's result.
        #[pallet::call_index(0)]
        #[pallet::weight({
            let dispatch_info = call.get_dispatch_info();
            (<T as Config>::WeightInfo::call_using_electricity().saturating_add(dispatch_info.weight), dispatch_info.class, Pays::No)
        })]
        #[allow(clippy::boxed_local)]
        pub fn call_using_electricity(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::try_spend_cell(&sender)?;

            let res = call
                .dispatch(RawOrigin::Signed(sender.clone()).into())
                .map(|_| ())
                .map_err(|e| e.error);
            let event = Event::ElectricitySpent(sender, res);

            Self::deposit_event(event);

            Ok(())
        }

        /// Put a cap on the number of cells individual accounts can use per era.
        /// To disable electricity temporary, set this to `0`.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as crate::Config>::WeightInfo::set_individual_electricity_era_limit())]
        pub fn set_individual_electricity_era_limit(
            origin: OriginFor<T>,
            max_cells: Option<Cells>,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;

            match max_cells {
                Some(n) => MaxUserElectricityUsagePerEra::<T>::put(n),
                None => MaxUserElectricityUsagePerEra::<T>::kill(),
            }

            Ok(())
        }

        /// Set the number of batteries an account has. Since they are rechargable, setting (vs
        /// giving) makes more sense in this context.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as crate::Config>::WeightInfo::set_battery_count())]
        pub fn set_battery_count(
            origin: OriginFor<T>,
            account: T::AccountId,
            battery_count: Cells,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            <ElectricalAccount<T>>::mutate(
                &account,
                |electrical_panel: &mut Option<ElectricalPanel>| match electrical_panel {
                    Some(electrical_panel) => {
                        electrical_panel.batteries = battery_count;
                    },
                    None => {
                        *electrical_panel = Some(ElectricalPanel {
                            batteries: battery_count,
                            zaps: 0,
                            used: ElectricityMeter { latest_era: 0, count: 0 },
                        });
                    },
                },
            );
            Ok(())
        }

        /// Give the recipient some zaps
        #[pallet::call_index(3)]
        #[pallet::weight(<T as crate::Config>::WeightInfo::give_zaps())]
        pub fn give_zaps(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            cells: Cells,
        ) -> DispatchResult {
            T::UpdateOrigin::ensure_origin(origin)?;
            <ElectricalAccount<T>>::mutate(
                &recipient,
                |electrical_panel: &mut Option<ElectricalPanel>| match electrical_panel {
                    Some(electrical_panel) => {
                        electrical_panel.zaps += cells;
                    },
                    None => {
                        *electrical_panel = Some(ElectricalPanel {
                            batteries: 0,
                            zaps: cells,
                            used: ElectricityMeter { latest_era: 0, count: 0 },
                        });
                    },
                },
            );
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // if OK(()), a electricity for the account was available
        /// TODO JH: maybe remove 'try' from the name since the updated version of Substrate makes
        /// all transactions `transactional`
        pub fn try_spend_cell(account_id: &<T>::AccountId) -> Result<(), Error<T>> {
            // gets max cells per era or return error if electricity is disabled
            let max_cells_per_era = Self::individual_electricity_era_limit();
            if max_cells_per_era == 0 as Cells {
                return Err(Error::<T>::ElectricityIsDisabled);
            }

            <ElectricalAccount<T>>::try_mutate(account_id, |panel: &mut Option<ElectricalPanel>| {
                let current_era_index = pallet_staking::Pallet::<T>::current_era().unwrap_or(0);

                match panel {
                    // User has at least had electricity at some point
                    Some(electrical_panel) => {
                        let era_index_is_current =
                            |electrical_panel: &mut ElectricalPanel| -> bool {
                                electrical_panel.used.latest_era == current_era_index
                            };

                        let user_has_used_max_electricity_allowed_this_era =
                            |electrical_panel: &mut ElectricalPanel| -> Result<bool, Error<T>> {
                                if era_index_is_current(electrical_panel)
                                    && electrical_panel.used.count >= max_cells_per_era
                                {
                                    return Err(Error::<T>::ElectricityEraLimitReached);
                                }

                                Ok(false)
                            };

                        let spend_cell = |electrical_panel: &mut ElectricalPanel| {
                            if era_index_is_current(electrical_panel) {
                                electrical_panel.used.count += 1;
                            } else {
                                electrical_panel.used =
                                    ElectricityMeter { latest_era: current_era_index, count: 1 }
                            }
                        };

                        let use_battery = |electrical_panel: &mut ElectricalPanel| {
                            spend_cell(electrical_panel);
                        };

                        let spend_zap = |electrical_panel: &mut ElectricalPanel| {
                            let count = electrical_panel.zaps;

                            electrical_panel.zaps = count.saturating_sub(1u32 as Cells);
                            spend_cell(electrical_panel);
                        };

                        let user_can_use_batteries =
                            |electrical_panel: &mut ElectricalPanel| -> Result<bool, Error<T>> {
                                let user_has_electricity_to_spend =
                                    !user_has_used_max_electricity_allowed_this_era(
                                        electrical_panel,
                                    )?;

                                Ok(user_has_electricity_to_spend
                                    && (electrical_panel.batteries > 0 as Cells)
                                    && ((era_index_is_current(electrical_panel)
                                        && electrical_panel.used.count
                                            < electrical_panel.batteries)
                                        || (electrical_panel.used.latest_era < current_era_index)))
                            };

                        let user_can_spend_zaps =
                            |electrical_panel: &mut ElectricalPanel| -> Result<bool, Error<T>> {
                                let user_has_electricity_to_spend =
                                    !user_has_used_max_electricity_allowed_this_era(
                                        electrical_panel,
                                    )?;

                                Ok(user_has_electricity_to_spend
                                    && electrical_panel.zaps > 0
                                    && ((era_index_is_current(electrical_panel)
                                        && electrical_panel.used.count < electrical_panel.zaps)
                                        || (electrical_panel.used.latest_era < current_era_index)))
                            };

                        // everything boils down this...
                        if user_can_use_batteries(electrical_panel)? {
                            use_battery(electrical_panel);
                        } else if user_can_spend_zaps(electrical_panel)? {
                            spend_zap(electrical_panel);
                        } else {
                            return Err(Error::<T>::NoCellsAvailable);
                        }
                    },
                    // if None, then account has no cells to use
                    None => return Err(Error::<T>::NoCellsAvailable),
                };
                Ok(())
            })
        }

        /// Returns number of cells a user can use this era
        pub fn cells_usable_this_era(account_id: &<T>::AccountId) -> Cells {
            if !Self::electricity_is_enabled() {
                return 0 as Cells;
            }

            // if the electricity was last used this era, return however many cells they
            // have left
            if let Some(data) = Self::electrical_account(account_id) {
                let ElectricalPanel { batteries, zaps, used } = data;

                // TODO refactor era_index_is_current() out of try_spend_cell() for reuse
                // here.
                if used.latest_era == pallet_staking::Pallet::<T>::current_era().unwrap_or(0) {
                    return min(
                        Self::individual_electricity_era_limit().saturating_sub(used.count),
                        batteries.saturating_sub(used.count).saturating_add(zaps),
                    );
                } else {
                    return min(
                        Self::individual_electricity_era_limit(),
                        batteries.saturating_add(zaps),
                    );
                }
            };

            0 as Cells
        }

        /// Returns the max number of cells a user can use per era
        pub fn individual_electricity_era_limit() -> Cells {
            match Self::max_user_electricity_usage_per_era() {
                Some(n) => n,
                None => Cells::MAX,
            }
        }

        /// Checks if electricity is enabled
        fn electricity_is_enabled() -> bool {
            Self::individual_electricity_era_limit() != 0 as Cells
        }
    }

    /// Verifies that the account has cells available before executing or broadcasting to other
    /// validators.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct ValidateElectricityPayment<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
    where
        <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>;

    impl<T: Config + Send + Sync> Debug for ValidateElectricityPayment<T>
    where
        <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
    {
        #[cfg(feature = "std")]
        fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            write!(f, "ValidateElectricityPayment")
        }

        #[cfg(not(feature = "std"))]
        fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
            Ok(())
        }
    }

    impl<T: Config + Send + Sync> ValidateElectricityPayment<T>
    where
        <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
    {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self(sp_std::marker::PhantomData)
        }
    }

    impl<T: Config + Send + Sync> SignedExtension for ValidateElectricityPayment<T>
    where
        <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
    {
        type AccountId = T::AccountId;
        type AdditionalSigned = ();
        type Call = <T as frame_system::Config>::RuntimeCall;
        type Pre = ();

        const IDENTIFIER: &'static str = "ValidateElectricityPayment";

        fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
            Ok(())
        }

        fn pre_dispatch(
            self,
            who: &Self::AccountId,
            call: &Self::Call,
            info: &DispatchInfoOf<Self::Call>,
            len: usize,
        ) -> Result<Self::Pre, TransactionValidityError> {
            self.validate(who, call, info, len).map(|_| ())
        }

        fn validate(
            &self,
            who: &Self::AccountId,
            call: &Self::Call,
            _info: &DispatchInfoOf<Self::Call>,
            _len: usize,
        ) -> TransactionValidity {
            #[allow(clippy::collapsible_match)]
            // is there a way to do all this shit better?
            if let Some(local_call) = call.is_sub_type() {
                if let Call::call_using_electricity { .. } = local_call {
                    if Pallet::<T>::electricity_is_enabled()
                        && Pallet::<T>::cells_usable_this_era(who) != 0
                    {
                        return Ok(ValidTransaction::default());
                    }
                    return Err(TransactionValidityError::Invalid(InvalidTransaction::Payment));
                }
            }
            Ok(ValidTransaction::default())
        }
    }
}
