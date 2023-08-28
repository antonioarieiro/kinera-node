//** About **//
	// The basic representation of a movie in the system.
	
	
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	
	//** Config **//

		//* Imports *//
			
			use frame_support::pallet_prelude::*;
			use frame_system::pallet_prelude::*;

			use codec::{Decode, Encode, MaxEncodedLen};
			use sp_std::{
				collections::btree_map::BTreeMap,
				vec::Vec,
			};

		//* Config *//

			#[pallet::pallet]
			#[pallet::generate_store(pub(super) trait Store)]
			pub struct Pallet<T>(_);

			#[pallet::config]
			pub trait Config: frame_system::Config {
				type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
				type DefaultReputation: Get<u32>;
			}


			
	//** Types **//	
	
		//* Types *//
		//* Constants *//
		//* Enums *//
		//* Structs *//

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)] //TODO add type info
			pub struct Stats {
				pub reputation: u32,
			}



	//** Storage **//

		#[pallet::storage]
		#[pallet::getter(fn wallet_reputation)]
		pub type WalletReputation<T: Config> = 
			StorageMap<
				_, 
				Blake2_128Concat, T::AccountId,
				Stats,
			>;

	
	
	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			AccountRegistered(T::AccountId),
		}
	


	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			WalletAlreadyRegistered,
		}



	//** Hooks **//

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	

		
	//** Extrinsics **//
		
		#[pallet::call]
		impl<T:Config> Pallet<T> {

		}
	
	
	
	//** Helpers **//
	
		impl<T:Config> Pallet<T> {
					
			pub fn register_new_wallet(
				who: &T::AccountId,
			) -> DispatchResult {
				
				// WalletReputation::<T>::try_get(who).unwrap(); // TODO check if moderator doesnt exist already

				let stats = Stats {
					reputation: T::DefaultReputation::get(),
				};
				WalletReputation::<T>::insert(who, stats.clone());

				Self::deposit_event(Event::AccountRegistered(who.clone()));   
				Ok(())
			}


			pub fn create_moderator_btree(
				moderators: Vec<T::AccountId>,
			) -> Result<BTreeMap<T::AccountId, u32>, DispatchError> {
				
				let mut btree = BTreeMap::new();
				
				for moderator_id in moderators {
					Self::register_new_wallet(&moderator_id)?;
					let total_reputation = WalletReputation::<T>::try_get(&moderator_id).unwrap().reputation;
					btree.insert(moderator_id, total_reputation);
				}

				Ok(btree)
			}

		}
	
}