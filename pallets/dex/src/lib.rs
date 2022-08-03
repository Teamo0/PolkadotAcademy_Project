#![cfg_attr(not(feature = "std"), no_std)]


// Re-export pallet items so that they can be accessed from the crate namespace.
pub use frame_system::pallet::*;
use pallet_assets;
use frame_support::{decl_module, dispatch, ensure};
use frame_system::ensure_signed;


#[frame_support::pallet]
pub mod pallet {
	use codec::HasCompact;
	use frame_support::dispatch::RawOrigin;
	use frame_support::pallet_prelude::*;
	use frame_support::PalletId;
	use frame_support::sp_runtime::{
		ArithmeticError,
		TokenError, traits::{
			AccountIdConversion, AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
		},
	};
	use frame_support::traits::ReservableCurrency;
	use frame_system::pallet_prelude::*;
	// use sp_runtime::traits::CheckedDiv;
	use sp_std::vec::Vec;

// Step 3.1 will include this in Cargo.toml

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_assets::Config {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;


		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		TokenSwap { owner: T::AccountId, asset_id: T::AssetId, amount1: T::Balance, asset_id_2: T::AssetId, amount2: T::Balance},
		AddLiquidity { owner: T::AccountId, asset_id_1: T::AssetId, amount1: T::Balance, asset_id_2: T::AssetId, amount2: T::Balance },
		RemoveLiquidity { owner: T::AccountId, asset_id_1: T::AssetId, amount1: T::Balance, asset_id_2: T::AssetId, amount2: T::Balance },

	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Account balance must be greater than or equal to the transfer amount.
		BalanceLow,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::storage]
	/// Map containing the number of token in liquidity pool
	pub(super) type LiqPool<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId, T::Balance, ValueQuery>;

	#[pallet::storage]
	/// individual stake per user for token1
	pub(super) type Tok1Bal<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage]
	/// individual stake per user for token2
	pub(super) type Tok2Bal<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, T::Balance, ValueQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(1_000)]
		pub fn add_liquidity(origin: OriginFor<T>, assetId_1: T::AssetId, amount_1: T::Balance, assetId_2: T::AssetId) -> DispatchResult {
			// function allows user to stake their asset token1 and token2 in the liquidity pool
			let sender = ensure_signed(origin.clone())?;

			// ratio of assets in liquidity pool has to remain constant
			let ratio = LiqPool::<T,I>::get(&assetId_2) / LiqPool::<T,I>::get(&assetId_1);
			// given amount of token1, calculate required amount of token2
			let amount_2 = amount_1 * ratio;

			// make sure sender has enough assets of both tokens for deposit, emit error otherwise
			ensure!(pallet_assets::Pallet::<T>::balance(assetId_1, &sender) >= amount_1, Error::<T,I>::BalanceLow);
			ensure!(pallet_assets::Pallet::<T>::balance(assetId_2, &sender) >= amount_2, Error::<T,I>::BalanceLow);

			// transfer token from sender to treasury account
			pallet_assets::Pallet::<T>::transfer(origin.clone(), assetId_1, T::PalletId::get().into_account_truncating(), amount_1);
			pallet_assets::Pallet::<T>::transfer(origin.clone(), assetId_2, T::PalletId::get().into_account_truncating(), amount_2);

			// increase total amount in liquidity pool
			LiqPool::<T, I>::insert(&assetId_1, LiqPool::<T,I>::get(&assetId_1) + amount_1);
			LiqPool::<T, I>::insert(&assetId_2, LiqPool::<T,I>::get(&assetId_2) + amount_2);

			// increase individual balance of user
			Tok1Bal::<T, I>::insert(&sender, Tok1Bal::<T,I>::get(&sender) + amount_1);
			Tok2Bal::<T, I>::insert(&sender, Tok2Bal::<T,I>::get(&sender) + amount_2);

			// liquidity added succesfully
			Self::deposit_event(Event::AddLiquidity { owner: sender, asset_id_1: assetId_1, amount1: amount_1, asset_id_2: assetId_2, amount2: amount_2 });
			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn remove_liquidity(origin: OriginFor<T>, assetId_1: T::AssetId, amount_1: T::Balance, assetId_2: T::AssetId) -> DispatchResult {
			// function allows user to remove liquidty, reverse add_liquidity
			let sender = ensure_signed(origin)?;

			// ratio of assets in liquidity pool has to remain constant
			let ratio = LiqPool::<T,I>::get(&assetId_2) / LiqPool::<T,I>::get(&assetId_1);
			let amount_2 = amount_1 * ratio;

			// Get liquidity balances
			let tok1bal = Tok1Bal::<T, I>::get(&sender);
			let tok2bal = Tok2Bal::<T, I>::get(&sender);

			// Verify that user has sufficient balances
			ensure!(tok1bal >= amount_1, Error::<T,I>::BalanceLow);
			ensure!(tok2bal >= amount_2, Error::<T,I>::BalanceLow);

			// Remove liquidity
			LiqPool::<T, I>::insert(&assetId_1, LiqPool::<T, I>::get(&assetId_1) - amount_1);
			LiqPool::<T, I>::insert(&assetId_2, LiqPool::<T, I>::get(&assetId_2) - amount_2);

			Tok1Bal::<T, I>::insert(&sender, tok1bal - amount_1);
			Tok2Bal::<T, I>::insert(&sender, tok2bal - amount_2);

			// transfer token ownership to sender
			let acc = Self::account_id();
			let lookup = <T as frame_system::Config>::Lookup::unlookup(sender.clone());
			pallet_assets::Pallet::<T>::transfer(RawOrigin::Signed(acc.clone()).into(), assetId_2, lookup.clone(), amount_2);
			pallet_assets::Pallet::<T>::transfer(RawOrigin::Signed(acc).into(), assetId_1, lookup, amount_1);

			// Emit an event that liquidity was removed
			Self::deposit_event(Event::RemoveLiquidity { owner: sender, asset_id_1: assetId_1, amount1: amount_1, asset_id_2: assetId_2, amount2: amount_2 });
			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn swap_tokens(origin: OriginFor<T>, assetId_input: T::AssetId, amount_input: T::Balance, assetId_output: T::AssetId) -> DispatchResult {

			let sender = ensure_signed(origin.clone())?;

			// charge 1% fees for staking rewards --- not working -> look up how to do arithmetics with balances
			// let fee = amount_input*0.01


			// check if user has enough token for input
			ensure!(pallet_assets::Pallet::<T>::balance(assetId_input, &sender) >= amount_input, Error::<T,I>::BalanceLow);

			// get amount of token2 to be acquired via helper function
			let amount_output = Self::enquire_rate(assetId_input, amount_input , assetId_output);

			// transfer funds of input token from signer to the treasury and update internal balance
			pallet_assets::Pallet::<T>::transfer(origin.clone(), assetId_input, T::PalletId::get().into_account_truncating(), amount_input);
			LiqPool::<T, I>::insert(&assetId_input, LiqPool::<T,I>::get(&assetId_input) + amount_input );


			// Remove liquidity of token2
			LiqPool::<T, I>::insert(&assetId_output, LiqPool::<T, I>::get(&assetId_output) - amount_output);

			// transfer token2 from treasury to user
			let acc = Self::account_id();
			let lookup = <T as frame_system::Config>::Lookup::unlookup(sender.clone());
			pallet_assets::Pallet::<T>::transfer(RawOrigin::Signed(acc).into(), assetId_output, lookup.clone(), amount_output);

			// distribute fees from treasuries to liquidity providers according to their staking amount
			// add: loop over TokBal and distribute fee according to staking amount per user
			// how to loop over keys in StorageMap?

			// swap successful, emit event
			Self::deposit_event(Event::TokenSwap { owner: sender, asset_id: assetId_input, amount1: amount_input, asset_id_2: assetId_output, amount2: amount_output});
			Ok(())
		}



	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		// Add public immutables and private mutables.

		/// The account ID of the treasury pot.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
		// helper function to ask for exchange rate
		pub fn enquire_rate(assetId_input: T::AssetId, amount_input: T::Balance, assetId_output: T::AssetId) -> T::Balance {
			// ask how much of token 2 can be acquired for a given amount of token 1
			// exchange rates follow principle of constant product, i.e. number of token1 * number of token2 stays constant

			// determine product before swap
			let prod = LiqPool::<T, I>::get(&assetId_input) * LiqPool::<T, I>::get(&assetId_output);

			// solve N(t1) * N(t2) = prod and return difference to current number of token2
			let return_value = prod/(LiqPool::<T, I>::get(&assetId_input) + amount_input) - LiqPool::<T, I>::get(&assetId_output);
			return_value
		}


	}
}
