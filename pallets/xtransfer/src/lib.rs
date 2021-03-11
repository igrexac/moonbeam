// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! This pallet exposes an interface for cross-chain token transfers from our parachain to
//! destination chains with 32 byte account IDs.
//! There are two dispatchables:
//! 1. `transfer_to_relay_chain` transfers relay chain tokens to an account on the relay chain
//! from our parachain
//! 2. `transfer_to_parachain` transfers tokens to an account on a parachain
//! Both transfers are sent from accounts on our chain (H160 account id).
//!
//! For transfers from other chains to our chain, their runtime must send us cross chain messages.
//! 1. For transfers from the relay chain, they queue DownwardMessages. These are processed by the
//! `DownwardMessageHandlers` associated type for our `cumulus_parachain_system` runtime impl.
//! 2. For transfers from other parachains, they send us HrmpMessages. These are processed by the
//! `HrmpMessageHandlers` associated type for our `cumulus_parachain_system` runtime impl.
//! Both `DownwardMessageHandlers` and `HrmpMessageHandlers` are set to `XcmHandler` in our runtime,
//! so they use the `MultiCurrency` impl of `TransactAsset` in `./support.rs` to process the
//! given message. The `Executor` associated type for this pallet uses the same logic to process
//! the cross-chain messages from our chain.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod support;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use cumulus_primitives::{relay_chain::Balance as RelayChainBalance, ParaId};
	use frame_support::{pallet_prelude::*, traits::Get, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::prelude::*;
	use xcm::v0::{
		Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm,
	};
	use xcm_executor::traits::LocationConversion;

	#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
	/// Identity of chain.
	pub enum ChainId {
		/// The relay chain.
		RelayChain,
		/// A parachain.
		ParaChain(ParaId),
	}

	#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
	/// Identity of cross chain currency.
	pub struct XCurrencyId {
		/// The reserve chain of the currency. For instance, the reserve chain
		/// of DOT is Polkadot.
		pub chain_id: ChainId,
		/// The identity of the currency.
		pub currency_id: Vec<u8>,
	}

	impl Into<MultiLocation> for XCurrencyId {
		fn into(self) -> MultiLocation {
			MultiLocation::X1(Junction::GeneralKey(self.currency_id))
		}
	}

	/// Pallet for executing cross-chain transfers
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The shape of AccountId for (most) substrate chains (not Moonbeam, which is H160 so 20 bytes)
	type AccountId32 = [u8; 32];

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Balances type
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Into<u128>;
		/// Convert local balance into relay chain balance type
		type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;
		/// Convert system::AccountId to key shape for Junction::AccountKey20 [u8; 20]
		type AccountKey20Convert: Convert<Self::AccountId, [u8; 20]>;
		/// Convert account to MultiLocation
		type ToMultiLocation: LocationConversion<Self::AccountId>;
		/// Relay chain identifier
		type RelayChainNetworkId: Get<NetworkId>;
		/// Moonbeam parachain identifier
		type ParaId: Get<ParaId>;
		/// XCM Executor
		type Executor: ExecuteXcm;
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transferred to relay chain. \[src, dest, amount\]
		TransferredToRelayChain(T::AccountId, AccountId32, T::Balance),
		/// Transfer to relay chain failed. \[src, dest, amount, error\]
		TransferToRelayChainFailed(T::AccountId, AccountId32, T::Balance, XcmError),
		/// Transferred to parachain. \[x_currency_id, src, para_id, dest, dest_network, amount\]
		TransferredToAccountId32Parachain(
			XCurrencyId,
			T::AccountId,
			ParaId,
			AccountId32,
			NetworkId,
			T::Balance,
		),
		/// Transfer to parachain failed. \[x_currency_id, src, para_id, dest,
		/// dest_network, amount, error\]
		TransferToAccountId32ParachainFailed(
			XCurrencyId,
			T::AccountId,
			ParaId,
			AccountId32,
			NetworkId,
			T::Balance,
			XcmError,
		),
		/// Transferred to parachain. \[x_currency_id, src, para_id, dest, dest_network, amount\]
		TransferredToAccountKey20Parachain(
			XCurrencyId,
			T::AccountId,
			ParaId,
			T::AccountId,
			NetworkId,
			T::Balance,
		),
		/// Transfer to parachain failed. \[x_currency_id, src, para_id, dest,
		/// dest_network, amount, error\]
		TransferToAccountKey20ParachainFailed(
			XCurrencyId,
			T::AccountId,
			ParaId,
			T::AccountId,
			NetworkId,
			T::Balance,
			XcmError,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Bad location.
		BadLocation,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer relay chain tokens to relay chain.
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_relay_chain(
			origin: OriginFor<T>,
			dest: AccountId32,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let xcm = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Junction::Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Junction::Parent),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: MultiLocation::X1(Junction::AccountId32 {
							network: T::RelayChainNetworkId::get(),
							id: dest.clone(),
						}),
					}],
				}],
			};

			let xcm_origin = T::ToMultiLocation::try_into_location(who.clone())
				.map_err(|_| Error::<T>::BadLocation)?;
			// TODO: revert state on xcm execution failure.
			match T::Executor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => {
					Self::deposit_event(Event::<T>::TransferredToRelayChain(who, dest, amount))
				}
				Err(err) => Self::deposit_event(Event::<T>::TransferToRelayChainFailed(
					who, dest, amount, err,
				)),
			}

			Ok(().into())
		}
		/// Transfer tokens to parachain that uses [u8; 32] for system::AccountId
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_account_id_32_parachain(
			origin: OriginFor<T>,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: AccountId32,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if para_id == T::ParaId::get() {
				return Ok(().into());
			}

			let destination = Self::account_id_32_destination(dest_network.clone(), &dest);

			let xcm = match x_currency_id.chain_id {
				ChainId::RelayChain => {
					Self::transfer_relay_chain_tokens_to_parachain(para_id, destination, amount)
				}
				ChainId::ParaChain(reserve_chain) => {
					if T::ParaId::get() == reserve_chain {
						Self::transfer_owned_tokens_to_parachain(
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					} else {
						Self::transfer_non_owned_tokens_to_parachain(
							reserve_chain,
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					}
				}
			};

			let xcm_origin = T::ToMultiLocation::try_into_location(who.clone())
				.map_err(|_| Error::<T>::BadLocation)?;
			// TODO: revert state on xcm execution failure.
			match T::Executor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => Self::deposit_event(Event::<T>::TransferredToAccountId32Parachain(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
				)),
				Err(err) => Self::deposit_event(Event::<T>::TransferToAccountId32ParachainFailed(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
					err,
				)),
			}

			Ok(().into())
		}
		/// Transfer tokens to parachain that uses [u8; 20] for system::AccountId
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_account_key_20_parachain(
			origin: OriginFor<T>,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: T::AccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if para_id == T::ParaId::get() {
				return Ok(().into());
			}

			let destination = Self::account_id_20_destination(dest_network.clone(), dest.clone());

			let xcm = match x_currency_id.chain_id {
				ChainId::RelayChain => {
					Self::transfer_relay_chain_tokens_to_parachain(para_id, destination, amount)
				}
				ChainId::ParaChain(reserve_chain) => {
					if T::ParaId::get() == reserve_chain {
						Self::transfer_owned_tokens_to_parachain(
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					} else {
						Self::transfer_non_owned_tokens_to_parachain(
							reserve_chain,
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					}
				}
			};

			let xcm_origin = T::ToMultiLocation::try_into_location(who.clone())
				.map_err(|_| Error::<T>::BadLocation)?;
			// TODO: revert state on xcm execution failure.
			match T::Executor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => Self::deposit_event(Event::<T>::TransferredToAccountKey20Parachain(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
				)),
				Err(err) => Self::deposit_event(Event::<T>::TransferToAccountKey20ParachainFailed(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
					err,
				)),
			}

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Form multilocation when recipient chain uses AccountId32 as system::AccountId type
		fn account_id_32_destination(network: NetworkId, id: &AccountId32) -> MultiLocation {
			MultiLocation::X1(Junction::AccountId32 {
				network,
				id: id.clone(),
			})
		}
		/// Form multilocation when recipient chain uses AccountKey20 as system::AccountId type
		fn account_id_20_destination(network: NetworkId, key: T::AccountId) -> MultiLocation {
			MultiLocation::X1(Junction::AccountKey20 {
				network,
				key: T::AccountKey20Convert::convert(key).clone(),
			})
		}
		/// Returns upward message to transfer tokens from relay chain to parachain
		fn transfer_relay_chain_tokens_to_parachain(
			para_id: ParaId,
			destination: MultiLocation,
			amount: T::Balance,
		) -> Xcm {
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Junction::Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Junction::Parent),
					effects: vec![Order::DepositReserveAsset {
						assets: vec![MultiAsset::All],
						// `dest` is children parachain(of parent).
						dest: MultiLocation::X1(Junction::Parachain { id: para_id.into() }),
						effects: vec![Order::DepositAsset {
							assets: vec![MultiAsset::All],
							dest: destination,
						}],
					}],
				}],
			}
		}
		/// Transfer parachain tokens "owned" by self parachain to another
		/// parachain.
		///
		/// NOTE - `para_id` must not be self parachain.
		fn transfer_owned_tokens_to_parachain(
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			destination: MultiLocation,
			amount: T::Balance,
		) -> Xcm {
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: x_currency_id.into(),
					amount: amount.into(),
				}],
				effects: vec![Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain { id: para_id.into() },
					),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: destination,
					}],
				}],
			}
		}
		/// Transfer parachain tokens not "owned" by self chain to another
		/// parachain.
		fn transfer_non_owned_tokens_to_parachain(
			reserve_chain: ParaId,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			destination: MultiLocation,
			amount: T::Balance,
		) -> Xcm {
			let deposit_to_dest = Order::DepositAsset {
				assets: vec![MultiAsset::All],
				dest: destination,
			};
			// If transfer to reserve chain, deposit to `dest` on reserve chain,
			// else deposit reserve asset.
			let reserve_chain_order = if para_id == reserve_chain {
				deposit_to_dest
			} else {
				Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain { id: para_id.into() },
					),
					effects: vec![deposit_to_dest],
				}
			};

			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: x_currency_id.into(),
					amount: amount.into(),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain {
							id: reserve_chain.into(),
						},
					),
					effects: vec![reserve_chain_order],
				}],
			}
		}
	}
}