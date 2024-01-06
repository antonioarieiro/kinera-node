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
				sp_runtime::{
					traits::{
						CheckedAdd,
						CheckedSub,
						CheckedDiv,
						Saturating,
					},
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

			// Allows the desambiguation of feature types.
			// Particularly useful for updating tokens values 
			// related to wallets.	
			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum FeatureType {
				Festival,
				RankingList,
				Moderation,
				Movie,
			}
			
			// Allows the desambiguation of token types.
			// Particularly useful for updating tokens values 
			// related to wallets.		
			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum TokenType {
				Locked,
				Claimable,
			}

		//* Structs *//


			// Stats that are bound to a wallet. This is required by many features, to ensure safety.
			// The "..._public" boolean parameters and the name are both defined by the user upon creation.
			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)] //TODO add type info
			pub struct Stats<BoundedName> {
				pub is_name_public: bool,
				pub is_wallet_public: bool,
				pub name: BoundedName,
			}
			
			
			// The "total_..." and "claimable_..." balance parameters are each updated by the corresponding app feature.
			// To get the current locked balance, you must do "total_..." - "claimable_..." = "locked_...". 
			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo,)] //TODO add type info
			pub struct Tokens<Balance, TokenImbalance> {
				pub reputation: u32,
				pub locked_tokens_moderation: Balance,
				pub claimable_tokens_moderation: Balance,
				pub locked_tokens_festival: Balance,
				pub claimable_tokens_festival: Balance,
				pub locked_tokens_ranking: Balance,
				pub claimable_tokens_ranking: Balance,
				pub imbalance_tokens_ranking: TokenImbalance,
				pub locked_tokens_movie: Balance,
				pub claimable_tokens_movie: Balance,
			}





	//** Storage **//


		// Contains stats related to the identification of this address.
		// When an entery is created for WalletStats, an entry is automatically
		// created in WalletTokens.
		#[pallet::storage]
		#[pallet::getter(fn get_wallet_stats)]
		pub type WalletStats<T: Config> = 
			StorageMap<
				_, 
				Blake2_128Concat, T::AccountId,
				Stats<
					BoundedVec<u8, T::NameStringLimit>,
				>,
			>; //TODO check why bounded vec has T::AccountId as the key


		// Keeps track of the amount of tokens (and reputation) a wallet has.
		// It is independent from the "WalletStats" storage, meaning an entry
		// can exist by itself without being registed in "WalletStats".
		#[pallet::storage]
		#[pallet::getter(fn get_wallet_tokens)]
		pub type WalletTokens<T: Config> = 
			StorageMap<
				_, 
				Blake2_128Concat, T::AccountId,
				Tokens<
					BalanceOf<T>,
					(BalanceOf<T>, BalanceOf<T>),
				>,
			>; //TODO check why bounded vec has T::AccountId as the key

	
	
	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			AccountRegisteredAddress(T::AccountId),
			AccountRegisteredName(BoundedVec<u8, T::NameStringLimit>),

			AccountUnregisteredAddress(T::AccountId),
			AccountUnregisteredName(BoundedVec<u8, T::NameStringLimit>),

			AccountDataUpdatedAddress(T::AccountId),
			AccountDataUpdatedName(BoundedVec<u8, T::NameStringLimit>),

			TokensClaimed(T::AccountId),
		}
	


	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			WalletAlreadyRegistered,
			WalletNotRegisteredStatTracker,
			WalletStatsNotFound,
			WalletTokensNotFound,

			DraftedModeratorNotRegistered,

			BadMetadata,
			Overflow,
			Underflow,
			WalletStatsRegistryRequired,
		}



	//** Hooks **//

		// #[pallet::hooks]
		// impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	

		
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
				
				let stats = Stats {
					is_wallet_public: is_wallet_public,
					is_name_public: is_name_public,
					name: name.clone(),
				};
				WalletStats::<T>::insert(who.clone(), stats.clone());

				if !WalletTokens::<T>::contains_key(who.clone()) {
					let zero_balance = BalanceOf::<T>::from(0u32);
					let tokens = Tokens {
						reputation: T::DefaultReputation::get(),
						locked_tokens_moderation: zero_balance.clone(),
						claimable_tokens_moderation: zero_balance.clone(),
						locked_tokens_festival: zero_balance.clone(),
						claimable_tokens_festival: zero_balance.clone(),
						locked_tokens_ranking: zero_balance.clone(),
						claimable_tokens_ranking: zero_balance.clone(),
						imbalance_tokens_ranking: (zero_balance.clone(), zero_balance.clone()),
						locked_tokens_movie: zero_balance.clone(),
						claimable_tokens_movie: zero_balance,
					};
					WalletTokens::<T>::insert(who.clone(), tokens.clone());
				};

				// check if events should be emitted, depending on the privacy settings
				if is_wallet_public {
					Self::deposit_event(Event::AccountRegisteredAddress(who));   
				}
				else if is_name_public {
					Self::deposit_event(Event::AccountRegisteredName(name));   
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
					let stats = wallet_stats.as_mut().ok_or(Error::<T>::WalletStatsNotFound)?;

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




			// #[pallet::weight(10_000)]
			// pub fn claim_all_tokens(
			// 	origin: OriginFor<T>,
			// ) -> DispatchResult {
				
			// 	let who = ensure_signed(origin)?;

			// 	WalletTokens::<T>::try_mutate_exists(who.clone(), |wallet_tokens| -> DispatchResult {
			// 		let tokens = wallet_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;

			// 		let zero_balance = BalanceOf::<T>::from(0u32);
					
			// 		// add all the claimable tokens into the same var
			// 		let mut total_tokens = 
			// 			zero_balance.clone()
			// 			.checked_add(tokens.claimable_tokens_moderation)
			// 			.ok_or(Error::<T>::Overflow)?;
			// 		total_tokens = 
			// 			total_tokens
			// 			.checked_add(tokens.claimable_tokens_festival)
			// 			.ok_or(Error::<T>::Overflow)?;
			// 		total_tokens = 
			// 			total_tokens
			// 			.checked_add(tokens.claimable_tokens_ranking)
			// 			.ok_or(Error::<T>::Overflow)?;

			// 		// ensure the transfer works
			// 		ensure!(
			// 			T::Currency::transfer(
			// 				&Self::account_id(), 
			// 				&who.clone(),
			// 				total_tokens.clone(), 
			// 				AllowDeath
			// 			) == Ok(()),
			// 			Error::<T>::NotEnoughBalance
			// 		);

			// 		// reset the total claimable tokens 
			// 		tokens.claimable_tokens_moderation = zero_balance.clone();
			// 		tokens.claimable_tokens_festival = zero_balance.clone();
			// 		tokens.claimable_tokens_ranking = zero_balance;

			// 		Self::deposit_event(Event::TokensClaimed(who));   
			// 		Ok(())
			// 	})?;



			// 	Ok(())
			// }

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
					let total_reputation = WalletTokens::<T>::try_get(&moderator_id).unwrap().reputation;
					btree.insert(moderator_id, total_reputation);
				}

				Ok(btree)
			}
				
			// apply either positive or negative value changes
			//TODO refactor into the token function
			pub fn apply_reputation_value_change(
				who: T::AccountId,
				reputation_value: u32,
			) -> Result<u32, DispatchError> {
				
				ensure!(
					WalletTokens::<T>::contains_key(who.clone()), 
					Error::<T>::WalletNotRegisteredStatTracker
				);

				let total_reputation = WalletTokens::<T>::get(who).unwrap().reputation;

				total_reputation.checked_add(reputation_value).ok_or(Error::<T>::Overflow)?;

				Ok(total_reputation)
			}
				
			// True if the wallet is registered in the "WalletStats" storage.
			// This always i mplies that an entry also exists e the 
			// "WalletTokens" storage.
			pub fn is_wallet_registered(
				who: T::AccountId,
			) -> Result<bool, DispatchError> {

				Ok(WalletStats::<T>::contains_key(who))
			}
			



			// Used to abstract the "update_token" functions.
			// This allows a single function to manage all 
			// the different types of tokens.
			pub fn do_update_wallet_tokens(
				who: T::AccountId,
				feature_type: FeatureType,
				token_type: TokenType,
				token_change: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult {
				
				// this is the first instance where the wallet's tokens are updated
				if !WalletTokens::<T>::contains_key(who.clone()) {
					Self::do_update_wallet_tokens_doesnt_exist(
						who, 
						feature_type, token_type, 
						token_change,
					)?;
				}

				// the wallet already contains an entry, retrieve & update
				else {
					Self::do_update_wallet_tokens_exists(
						who, 
						feature_type, token_type, 
						token_change, is_slash,
					)?;
				}

				Ok(())
			}


			//
			pub fn do_update_wallet_tokens_doesnt_exist(
				who: T::AccountId,
				feature_type: FeatureType,
				token_type: TokenType,
				token_change: BalanceOf<T>,
			) -> DispatchResult  {

				let zero_balance = BalanceOf::<T>::from(0u32);
				let mut wallet_tokens = Tokens {
					reputation: T::DefaultReputation::get(),
					locked_tokens_moderation: zero_balance.clone(),
					claimable_tokens_moderation: zero_balance.clone(),
					
					locked_tokens_festival: zero_balance.clone(),
					claimable_tokens_festival: zero_balance.clone(),
					
					locked_tokens_ranking: zero_balance.clone(),
					claimable_tokens_ranking: zero_balance.clone(),
					imbalance_tokens_ranking: (zero_balance.clone(), zero_balance.clone()),
					
					locked_tokens_movie: zero_balance.clone(),
					claimable_tokens_movie: zero_balance,
				};

				match (feature_type, token_type) {
					(FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = token_change.clone(),
					(FeatureType::Festival, TokenType::Claimable) => wallet_tokens.claimable_tokens_festival = token_change.clone(),

					(FeatureType::RankingList, TokenType::Locked) => wallet_tokens.locked_tokens_ranking = token_change.clone(),
					(FeatureType::RankingList, TokenType::Claimable) => wallet_tokens.claimable_tokens_ranking = token_change.clone(),
					
					(FeatureType::Moderation, TokenType::Locked) => wallet_tokens.locked_tokens_moderation = token_change.clone(),
					(FeatureType::Moderation, TokenType::Claimable) => wallet_tokens.claimable_tokens_moderation = token_change.clone(),
					
					(FeatureType::Movie, TokenType::Locked) => wallet_tokens.locked_tokens_movie = token_change.clone(),
					(FeatureType::Movie, TokenType::Claimable) => wallet_tokens.claimable_tokens_movie = token_change.clone(),
				};

				//TODO add negative integer use case 
				// (FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = 
				// zero_balance.clone()
				// .checked_sub(&total_tokens.clone())
				// .ok_or(Error::<T>::Underflow)?,

				WalletTokens::<T>::insert(who.clone(), wallet_tokens.clone());

				Ok(())
			}


			//
			pub fn do_update_wallet_tokens_exists(
				who: T::AccountId,
				feature_type: FeatureType,
				token_type: TokenType,
				token_change: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult  {

				WalletTokens::<T>::try_mutate(who, |wal_tokens| -> DispatchResult {
					let wallet_tokens = wal_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
					
					// dynamically select which token variable to update
					match (feature_type, token_type) {
						(FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = 
							Self::do_calculate_token_value(
								wallet_tokens.locked_tokens_festival.clone(),
								token_change,
								is_slash,
							)?,
						(FeatureType::Festival, TokenType::Claimable) => wallet_tokens.claimable_tokens_festival = 
							Self::do_calculate_token_value(
								wallet_tokens.claimable_tokens_festival.clone(),
								token_change, is_slash,
							)?,
	
						(FeatureType::RankingList, TokenType::Locked) => wallet_tokens.locked_tokens_ranking = 
							Self::do_calculate_token_value(
								wallet_tokens.locked_tokens_ranking.clone(),
								token_change, is_slash,
							)?, 
						(FeatureType::RankingList, TokenType::Claimable) => wallet_tokens.claimable_tokens_ranking = 
							Self::do_calculate_token_value(
								wallet_tokens.claimable_tokens_ranking.clone(),
								token_change, is_slash,
							)?,
						
						(FeatureType::Moderation, TokenType::Locked) => wallet_tokens.locked_tokens_moderation = 
							Self::do_calculate_token_value(
								wallet_tokens.locked_tokens_moderation.clone(),
								token_change, is_slash,
							)?,
						(FeatureType::Moderation, TokenType::Claimable) => wallet_tokens.claimable_tokens_moderation = 
							Self::do_calculate_token_value(
								wallet_tokens.claimable_tokens_moderation.clone(),
								token_change, is_slash,
							)?,

						(FeatureType::Movie, TokenType::Locked) => wallet_tokens.locked_tokens_movie = 
							Self::do_calculate_token_value(
								wallet_tokens.locked_tokens_movie.clone(),
								token_change, is_slash,
							)?,
						(FeatureType::Movie, TokenType::Claimable) => wallet_tokens.claimable_tokens_movie = 
							Self::do_calculate_token_value(
								wallet_tokens.claimable_tokens_movie.clone(),
								token_change, is_slash,
							)?,
					};

					Ok(())
				})?;

				Ok(())
			}


			pub fn do_calculate_token_value(
				mut current_tokens: BalanceOf<T>,
				token_change: BalanceOf<T>,
				is_slash: bool,
			) -> Result<BalanceOf::<T>, DispatchError>  {

				// reset the locked tokens back to 0
				// if token_change == BalanceOf::<T>::from(0u32) {
				// 	current_tokens = BalanceOf::<T>::from(0u32);
				// }
				if is_slash {
					current_tokens =
						current_tokens.clone()
						.checked_sub(&token_change)
						.ok_or(Error::<T>::Underflow)?;
				}
				else {
					current_tokens =
						current_tokens.clone()
						.checked_add(&token_change)
						.ok_or(Error::<T>::Overflow)?;
				}

				Ok(current_tokens)
			}








			// Takes an "imbalance" (a fraction of staked/total) and updates 
			// it depending on the current imbalance.
			// if the nominator is higher than the the denominator, the quoeficient
			// as an integer is returned alongside the new imbalance.
			pub fn do_update_wallet_imbalance(
				who: T::AccountId,
				feature_type: FeatureType,
				new_earned: BalanceOf<T>,
				total_earning_needed: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult  {

				// this is the first instance where the wallet's tokens are updated
				if !WalletTokens::<T>::contains_key(who.clone()) {
					Self::do_handle_imbalance_wallet_doesnt_exist(
						who, feature_type, 
						new_earned, total_earning_needed,
						is_slash,
					)?;
				}

				// the wallet already contains an entry, retrieve & update
				else {
					Self::do_handle_imbalance_wallet_exists(
						who,
						feature_type, 
						new_earned, total_earning_needed,
						is_slash,
					)?;
				}

				Ok(())
			}



			pub fn do_handle_imbalance_wallet_doesnt_exist(
				who: T::AccountId,
				feature_type: FeatureType,
				new_earned: BalanceOf<T>,
				total_earning_needed: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult  {

				let zero_balance = BalanceOf::<T>::from(0u32);
				let mut wallet_tokens = Tokens {
					reputation: T::DefaultReputation::get(),
					locked_tokens_moderation: zero_balance.clone(),
					claimable_tokens_moderation: zero_balance.clone(),
					
					locked_tokens_festival: zero_balance.clone(),
					claimable_tokens_festival: zero_balance.clone(),
					
					locked_tokens_ranking: zero_balance.clone(),
					claimable_tokens_ranking: zero_balance.clone(),
					imbalance_tokens_ranking: (zero_balance.clone(), zero_balance.clone()),
					
					locked_tokens_movie: zero_balance.clone(),
					claimable_tokens_movie: zero_balance,
				};

				match feature_type {
					FeatureType::RankingList => {
						let (new_balance, new_imbalance) = 
							Self::do_calculate_imbalance_value(
								BalanceOf::<T>::from(0u32),
								new_earned,
								total_earning_needed,
								is_slash,
							)?;
						
						wallet_tokens.imbalance_tokens_ranking = new_imbalance;

						WalletTokens::<T>::insert(who.clone(), wallet_tokens.clone());

						// if new_balance > BalanceOf::<T>::from(0u32) {
						// 	wallet_tokens.claimable_tokens_festival = 
						// 		Self::do_calculate_token_value(
						// 			wallet_tokens.claimable_tokens_festival.clone(),
						// 			token_change, is_slash,
						// 		)?,
						// }
					},
					
					_ => ()
				};

				//TODO add negative integer use case 

				

				Ok(())
			}





			// it depending on the current imbalance.
			// if the nominator is higher than the the denominator, the quoeficient
			// as an integer is returned alongside the new imbalance.
			pub fn do_handle_imbalance_wallet_exists(
				who: T::AccountId,
				feature_type: FeatureType,
				new_earned: BalanceOf<T>,
				total_earning_needed: BalanceOf<T>,
				is_slash: bool,
			) -> DispatchResult  {

				WalletTokens::<T>::try_mutate(who.clone(), |wal_tokens| -> DispatchResult {
					let wallet_tokens = wal_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
					
					// dynamically select which token variable to update
					match feature_type {
						FeatureType::RankingList => {
							let (current_earned, _) = 
								wallet_tokens.imbalance_tokens_ranking;
							
							let (new_balance, new_imbalance) = 
								Self::do_calculate_imbalance_value(
									current_earned,
									new_earned,
									total_earning_needed,
									is_slash,
								)?;
							
							wallet_tokens.imbalance_tokens_ranking = new_imbalance;

							if new_balance > BalanceOf::<T>::from(0u32) {
								wallet_tokens.claimable_tokens_festival = 
									// new_balance;
									Self::do_calculate_token_value(
										wallet_tokens.claimable_tokens_festival,
										new_balance, is_slash,
									)?;
							}
						},

						_ => ()

					};

					Ok(())
				})?;

				Ok(())
			}












			// Takes an "imbalance" (a fraction of staked/total) and updates 
			// it depending on the current imbalance.
			// if the nominator is higher than the the denominator, the quoeficient
			// as an integer is returned alongside the new imbalance.
			pub fn do_calculate_imbalance_value(
				mut current_earned: BalanceOf<T>,
				new_earned: BalanceOf<T>,
				total_earning_needed: BalanceOf<T>,
				is_slash: bool,
			) -> Result<(BalanceOf::<T>, (BalanceOf::<T>, BalanceOf::<T>)), 
			DispatchError> {

				let mut new_balance = BalanceOf::<T>::from(0u32);

				// reset the locked tokens back to 0
				// if new_earned == BalanceOf::<T>::from(0u32)
				// && total_earning_needed == BalanceOf::<T>::from(0u32) {
				// 	current_earned = BalanceOf::<T>::from(0u32);
				// }
				//TODO
				// else if is_slash {
				// 	current_imbalance =
				// 		current_tokens.clone()
				// 		.checked_sub(&token_change)
				// 		.ok_or(Error::<T>::Underflow)?;
				// }


				if !is_slash {
					current_earned =
						current_earned
						.checked_add(&new_earned)
						.ok_or(Error::<T>::Overflow)?;
					
					if current_earned > total_earning_needed {
						new_balance = 
							current_earned
							.checked_div(&total_earning_needed)
							.ok_or(Error::<T>::Overflow)?;
						
						let imbalance = new_balance.saturating_mul(
							total_earning_needed
						);
						
						current_earned =
							current_earned
							.checked_sub(&imbalance)
							.ok_or(Error::<T>::Underflow)?;
					}
				}

				Ok((new_balance, (current_earned, total_earning_needed)))
			}



		}
}