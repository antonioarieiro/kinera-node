//** About **//
	// The basic representation of a movie in the system.
	//TODO check the use of references in the helper functions that do not need to use .clone()
	
	
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
			
			use frame_support::{
				pallet_prelude::*,
				traits::{
					Currency,
					ReservableCurrency,
				},
				sp_runtime::traits::{
					CheckedAdd,
					CheckedSub,
				}
			};
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
				type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

				type DefaultReputation: Get<u32>;
				type NameStringLimit: Get<u32>;
			}


			
	//** Types **//	
	
		//* Types *//

			type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

		//* Constants *//
		//* Enums *//
		//* Structs *//


			// Stats that are bound to a wallet. This is required by many features, to ensure safety.
			// The "..._public" boolean parameters and the name are both defined by the user upon creation.
			// The "total_..." and "claimable_..." balance parameters are each updated by the corresponding app feature.
			// To get the current locked balance, you must do "total_..." - "claimable_..." = "locked_...". 
			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)] //TODO add type info
			pub struct Stats<BoundedName, Balance> {
				pub is_name_public: bool,
				pub is_wallet_public: bool,
				pub name: BoundedName,
				
				pub reputation: u32,

				pub locked_tokens_moderation: Balance,
				pub claimable_tokens_moderation: Balance,
				pub locked_tokens_festival: Balance,
				pub claimable_tokens_festival: Balance,
				pub locked_tokens_ranking: Balance,
				pub claimable_tokens_ranking: Balance,
			}



	//** Storage **//

		#[pallet::storage]
		#[pallet::getter(fn get_wallet_stats)]
		pub type WalletStats<T: Config> = 
			StorageMap<
				_, 
				Blake2_128Concat, T::AccountId,
				Stats<
					BoundedVec<u8, T::NameStringLimit>,
					BalanceOf<T>,
				>,
			>; //TODO check why bounded vec has T::AccountId as the key

	
	
	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			AccountRegisteredAdress(T::AccountId),
			AccountRegisteredName(BoundedVec<u8, T::NameStringLimit>),

			AccountUnregisteredAddress(T::AccountId),
			AccountUnregisteredName(BoundedVec<u8, T::NameStringLimit>),

			AccountDataUpdatedAddress(T::AccountId),
			AccountDataUpdatedName(BoundedVec<u8, T::NameStringLimit>),
		}
	


	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			WalletAlreadyRegistered,
			WalletNotRegisteredStatTracker,
			DraftedModeratorNotRegistered,
			BadMetadata,
			Overflow,
			Underflow,
			WalletStatsRegistryRequired,
		}



	//** Hooks **//

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	

		
	//** Extrinsics **//
		
		#[pallet::call]
		impl<T:Config> Pallet<T> {

			#[pallet::weight(10_000)]
			pub fn register_new_wallet(
				origin: OriginFor<T>,
				is_name_public: bool,
				is_wallet_public: bool,
				name: BoundedVec<u8, T::NameStringLimit>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				ensure!(
					!WalletStats::<T>::contains_key(who.clone()), 
					Error::<T>::WalletAlreadyRegistered
				);

				let zero_balance = BalanceOf::<T>::from(0u32);
				let stats = Stats {
					is_wallet_public: is_wallet_public,
					is_name_public: is_name_public,
					name: name.clone(),
					reputation: T::DefaultReputation::get(),

					locked_tokens_moderation: zero_balance.clone(),
					claimable_tokens_moderation: zero_balance.clone(),
					locked_tokens_festival: zero_balance.clone(),
					claimable_tokens_festival: zero_balance.clone(),
					locked_tokens_ranking: zero_balance.clone(),
					claimable_tokens_ranking: zero_balance,
				};
				WalletStats::<T>::insert(who.clone(), stats.clone());

				// check if events should be emitted, depending on the privacy settings
				if is_wallet_public {
					Self::deposit_event(Event::AccountUnregisteredAddress(who));   
				}
				else if is_name_public {
					Self::deposit_event(Event::AccountUnregisteredName(name));   
				};   

				Ok(())
			}


			#[pallet::weight(10_000)]
			pub fn unregister_wallet(
				origin: OriginFor<T>,
				name: BoundedVec<u8, T::NameStringLimit>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;

				let stats = WalletStats::<T>::try_get(who.clone()).unwrap();

				WalletStats::<T>::remove(who.clone());

				//TODO check all active interactions in the following pallets:
				// social-space, festival, ranking-list, moderation

				// check if events should be emitted, depending on the privacy settings
				if stats.is_wallet_public {
					Self::deposit_event(Event::AccountUnregisteredAddress(who));   
				}
				else if stats.is_name_public {
					Self::deposit_event(Event::AccountUnregisteredName(name));   
				}

				Ok(())
			}







			#[pallet::weight(10_000)]
			pub fn update_wallet_data(
				origin: OriginFor<T>,
				is_name_public: bool,
				is_wallet_public: bool,
				name: BoundedVec<u8, T::NameStringLimit>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				ensure!(
					WalletStats::<T>::contains_key(who.clone()), 
					Error::<T>::WalletNotRegisteredStatTracker
				);

				WalletStats::<T>::try_mutate(who.clone(), |wallet_stats| -> DispatchResult {
					let stats = wallet_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;

					// update the wallet's data
					stats.is_name_public = is_name_public;
					stats.is_wallet_public = is_wallet_public;
					stats.name = name.clone();

					Ok(())
				})?;

				// check if events should be emitted, depending on the privacy settings
				if is_wallet_public {
					Self::deposit_event(Event::AccountDataUpdatedAddress(who));   
				}
				else if is_name_public {
					Self::deposit_event(Event::AccountDataUpdatedName(name));   
				}

				Ok(())
			}

		}
	
	
	
	//** Helpers **//
	
		impl<T:Config> Pallet<T> {
					
			pub fn create_moderator_btree(
				moderators: Vec<T::AccountId>,
			) -> Result<BTreeMap<T::AccountId, u32>, DispatchError> {
				
				let mut btree = BTreeMap::new();
				
				for moderator_id in moderators {
					ensure!(
						WalletStats::<T>::contains_key(moderator_id.clone()), 
						Error::<T>::DraftedModeratorNotRegistered
					);
					let total_reputation = WalletStats::<T>::try_get(&moderator_id).unwrap().reputation;
					btree.insert(moderator_id, total_reputation);
				}

				Ok(btree)
			}
				
			// apply either positive or negative value changes
			pub fn apply_reputation_value_change(
				who: T::AccountId,
				reputation_value: u32,
			) -> Result<u32, DispatchError> {
				
				ensure!(
					WalletStats::<T>::contains_key(who.clone()), 
					Error::<T>::WalletNotRegisteredStatTracker
				);

				let total_reputation = WalletStats::<T>::get(who).unwrap().reputation;

				total_reputation.checked_add(reputation_value).ok_or(Error::<T>::Overflow)?;

				Ok(total_reputation)
			}
				
			//
			pub fn is_wallet_registered(
				who: T::AccountId,
			) -> Result<bool, DispatchError> {

				Ok(WalletStats::<T>::contains_key(who))
			}
			
			


			//* Ranking Tokens *//

			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the ranking section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_locked_tokens_ranking(
				who: T::AccountId,
				locked_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_locked = wallet_stats.locked_tokens_ranking;

					// reset the locked tokens back to 0
					if locked_tokens == BalanceOf::<T>::from(0u32) {
						current_locked = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_locked = 
							current_locked
							.checked_sub(&locked_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_locked = 
							current_locked
							.checked_add(&locked_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}


			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the ranking section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_claimable_tokens_ranking(
				who: T::AccountId,
				claimable_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_claimable = wallet_stats.claimable_tokens_ranking;

					// reset the locked tokens back to 0
					if claimable_tokens == BalanceOf::<T>::from(0u32) {
						current_claimable = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_claimable = 
							current_claimable
							.checked_sub(&claimable_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_claimable = 
							current_claimable
							.checked_add(&claimable_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}
			



			//* Festival Tokens *//

			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the festival section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_locked_tokens_festival(
				who: T::AccountId,
				locked_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_locked = wallet_stats.locked_tokens_festival;

					// reset the locked tokens back to 0
					if locked_tokens == BalanceOf::<T>::from(0u32) {
						current_locked = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_locked = 
							current_locked
							.checked_sub(&locked_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_locked = 
							current_locked
							.checked_add(&locked_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}


			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the festival section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_claimable_tokens_festival(
				who: T::AccountId,
				claimable_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_claimable = wallet_stats.claimable_tokens_festival;

					// reset the locked tokens back to 0
					if claimable_tokens == BalanceOf::<T>::from(0u32) {
						current_claimable = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_claimable = 
							current_claimable
							.checked_sub(&claimable_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_claimable = 
							current_claimable
							.checked_add(&claimable_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}





			//* Moderation Tokens *//


			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the moderation section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_locked_tokens_moderation(
				who: T::AccountId,
				locked_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_locked = wallet_stats.locked_tokens_moderation;

					// reset the locked tokens back to 0
					if locked_tokens == BalanceOf::<T>::from(0u32) {
						current_locked = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_locked = 
							current_locked
							.checked_sub(&locked_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_locked = 
							current_locked
							.checked_add(&locked_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}


			//TODO update the "update_..." functions using & to acess memory of the exact variable, instead of having 3 funcs
			// Updates the values for the moderation section of tokens.
			// If the value is 0, the claimable tokens are reset to 0.
			// Any other value is added to the "claimable_tokens" pool if positive,
			// and subtracted if negative.
			pub fn update_claimable_tokens_moderation(
				who: T::AccountId,
				claimable_tokens: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {

				WalletStats::<T>::try_mutate_exists(who, |wal_stats| -> DispatchResult {
					let wallet_stats = wal_stats.as_mut().ok_or(Error::<T>::BadMetadata)?;
					let mut current_claimable = wallet_stats.claimable_tokens_moderation;

					// reset the locked tokens back to 0
					if claimable_tokens == BalanceOf::<T>::from(0u32) {
						current_claimable = BalanceOf::<T>::from(0u32);
					}
					else if is_slash {
						current_claimable = 
							current_claimable
							.checked_sub(&claimable_tokens)
							.ok_or(Error::<T>::Underflow)?;
					}
					else {
						current_claimable = 
							current_claimable
							.checked_add(&claimable_tokens)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				})?;

				Ok(())
			}


			// update_tokens_moderation




		}
}