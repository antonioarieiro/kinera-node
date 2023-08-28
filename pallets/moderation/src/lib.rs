//** About **//
	// This pallet handles the moderation process. It manages all interactions related to assigning moderators, 
	// drafting them for reports, handling voting consensus and reward splitting.
	// A reported content's id is matched to unique report structures that return its specific information,
	// or information regarding each moderation "tier" or court.
	
	
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
					ExistenceRequirement::{
						KeepAlive, 
					},
				},
				BoundedVec,
				PalletId,
			};
			use frame_system::pallet_prelude::*;
			use codec::{Decode, Encode, MaxEncodedLen};
			use sp_runtime::{
				RuntimeDebug, 
				traits::{
					AtLeast32BitUnsigned, 
					CheckedAdd, 
					CheckedSub, 
					One,
					AccountIdConversion,
					CheckedDiv, 
					Saturating, 
				},
			};
			use scale_info::{
				TypeInfo,
				prelude::vec::Vec,
			};
			use core::convert::TryInto;
			use sp_std::{
				collections::btree_map::BTreeMap,
				vec,
			};

		//* Config *//

			#[pallet::pallet]
			#[pallet::generate_store(pub(super) trait Store)]
			pub struct Pallet<T>(_);

			#[pallet::config]
			pub trait Config: frame_system::Config + pallet_stat_tracker::Config {
				type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
			
				#[pallet::constant]
				type JustificationLimit: Get<u32>;

				type ContentId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
				type MaxReportsByModerator: Get<u32>;
				type TotalTierOneModerators: Get<u32>;
				type MaxReportsByTier: Get<u32>; // TODO implement dynamically with ModeratorLimitByTier
			
				type MinimumTokensForModeration: Get<BalanceOf<Self>>; 
				type MovieCollateral: Get<BalanceOf<Self>>; 

				type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
				type PalletId: Get<PalletId>;
			}

	//** Types **//	
	
		//* Types *//

			type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
		
		//* Constants *//
		//* Enums *//

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum InfringimentType {
				Violence,
				Discrimination,
				LackOfConsent,
				Impersonation,
				Terrorism,
				Copyright,
				FakeNews,
				Pornography,
				Extreme,
				Naming,
				Categorization,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ContentType {
				Festival,
				Movie,
				Category,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ReportStatus {
				InResolution,
				MajorityVotedFor, // TODO rename to in favour
				MajorityVotedAgainst,
				AppealedByReporter,
				AppealedByReportee,
				Accepted,
				Refused,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum Tiers {
				TierOne,
				TierTwo,
				TierThree,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum VoteChoice {
				For,
				Against,
				Abstinence,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ModeratorRank {
				Junior,
				Senior,
			}

		//* Structs *//

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Report <AccountId, ContentType, ReportStatus,  InfringimentType, BoundedString> {
				pub	reporter_id: AccountId,
				pub reportee_id: AccountId,
				pub content_type: ContentType,
				pub infringiment: InfringimentType,
				pub justification: BoundedString,
				pub status: ReportStatus,
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct ReportOutcome<VoteList, BalanceOf> {
				pub staked_tokens: BalanceOf,
				pub required_votes: u32,
				pub votes_for: u32,
				pub votes: VoteList
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Moderator<ReportList, BalanceOf, ModeratorRank> {
				pub collateral_tokens: BalanceOf,
				pub assigned_reports: ReportList,
				pub rank: ModeratorRank,
				pub total_points: u32,
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Vote <AccountId> {
				pub voter: AccountId,
				pub is_for: VoteChoice,
			}



	//** Storage **//

		//* Reports *//

			// matches a report id to the report data
			#[pallet::storage]
			#[pallet::getter(fn report)]
			pub type Reports<T: Config> =
				StorageMap<
					_, 
					Blake2_128Concat, T::ContentId, 
					Report<T::AccountId, ContentType, ReportStatus, InfringimentType, BoundedVec<u8, T::JustificationLimit>>,
				>;

			// matches a report id to its current status
			#[pallet::storage]
			#[pallet::getter(fn report_tier_verdict)]
			pub type ReportVerdicts<T: Config> =
				StorageDoubleMap<
					_,
					Blake2_128Concat, T::ContentId, 
					Blake2_128Concat, Tiers,
					ReportOutcome<BoundedVec<Vote<T::AccountId>, T::MaxReportsByTier>, BalanceOf<T>>,
					OptionQuery,
				>;

		
		//* Moderators *//

			// match a moderator's id to its moderator information
			#[pallet::storage]
			pub type Moderators<T: Config> =
				StorageMap<
					_, 
					Blake2_128Concat, T::AccountId, 
					Moderator<
						BoundedVec<T::ContentId, T::MaxReportsByModerator>, 
						BalanceOf<T>,
						ModeratorRank,
					>,
					OptionQuery,
				>;

	
	
	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			ModeratorRegistered(T::AccountId),
			ModerationActivitySuspended(T::AccountId),
			ModerationRewardsClaimed(T::AccountId, BalanceOf<T>),

			VoteSubmitted(),
			
			ReportCreated(T::ContentId, T::AccountId, ContentType, InfringimentType),
			ReportClosed(T::ContentId, ReportStatus),
			ReportAppealed(T::ContentId),
			ReportAppealAccepted(T::ContentId),
			ReportAppealRefused(T::ContentId),
			FestivalReported,
			MovieReported,
			CategoryReported,
			CommentReported,

			JuryDraftedTierOne,
			ModerationStartedTierOne,
			TierOneFinalized,

			EscalateToTierTwo,
			JuryDraftedTierTwo,
			ModerationStartedTierTwo,
			TierTwoFinalized,

			EscalateToTierThree,
			JuryDraftedTierThree,
			ModerationStartedTierThree,
			TierThreeFinalized,
		}
	


	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			Overflow,
			Underflow,
			BadMetadata,
			
			AlreadyRegisteredAsModerator,
			NonexistentModerator,
			InvalidModeratorData,
			ModeratorNotDraftedForReport,
			NotEnoughModeratorsRegistered,
			
			UserCannotAcceptVerdict,
			
			NonexistentReport,
			ReportsAwaitingVote,
			ReportAlreadyOngoing,
			ReportAppealLimitReached,

			InsuficientBalance,
		}



	//** Hooks **//

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
			// TODO check if any report is past due
			// TODo update veredicts after time limit if no appeal is made
		}
	

		
	//** Extrinsics **//
		
		#[pallet::call]
		impl<T: Config> Pallet<T> {
			

			#[pallet::weight(10_000)]
			pub fn apply_for_moderation(
				origin: OriginFor<T>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				Self::do_check_if_already_moderator(&who)?;
				// TODO call tracker and check if enough reputation
				
				Self::do_transfer_funds_to_treasury(&who, T::MinimumTokensForModeration::get())?;
				Self::do_create_moderator(&who)?; //TODO extract the config get
				
				Self::deposit_event(Event::ModeratorRegistered(who.clone()));
				Ok(())
			}	


			#[pallet::weight(10_000)]
			pub fn suspend_moderation_activity(
				origin: OriginFor<T>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				Self::do_can_moderator_suspend(&who)?;
				Self::do_suspend_moderation(&who)?;

				Self::deposit_event(Event::ModerationActivitySuspended(who.clone()));
				Ok(())
			}	


			#[pallet::weight(10_000)]
			pub fn claim_moderation_rewards(
				origin: OriginFor<T>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				let reward = Self::do_claim_moderation_reward(&who)?;

				Self::deposit_event(Event::ModerationRewardsClaimed(who, reward));
				Ok(())
			}	


			#[pallet::weight(10_000)]
			pub fn create_report(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				reportee_id: T::AccountId,
				content_type: ContentType,
				infringiment: InfringimentType,
				justification: BoundedVec<u8, T::JustificationLimit>,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				Self::do_validate_report_data(content_id, &justification)?; //TODO check if report is ongoing

				let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;
				Self::do_transfer_funds_to_treasury(&who, reward_pool.0)?;

				let moderator_btree = Self::do_get_moderator_btree()?;
				let drafted_moderators = Self::do_draft_moderators_from_btree(moderator_btree, T::TotalTierOneModerators::get())?;

				Self::do_create_report(&who, content_id, &reportee_id, content_type, infringiment, justification)?;
				Self::do_create_report_verdict(content_id, Tiers::TierOne, reward_pool.1)?;
				Self::do_assign_report_to_moderators(content_id, drafted_moderators)?;

				Self::deposit_event(Event::ReportCreated(content_id, who.clone(), content_type, infringiment));
				Ok(())
			}


			#[pallet::weight(10_000)]
			pub fn submit_vote(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				vote: VoteChoice,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				Self::do_can_moderator_vote(&who, &content_id)?;

				let tier_data = Self::do_get_current_report_tier_data(&content_id)?;
				Self::do_create_vote(content_id, tier_data.0, &who, vote)?;
				Self::do_deallocate_moderator_from_report(&who, &content_id)?;
				
				if Self::do_are_all_votes_submitted(content_id, tier_data.0)? == true {
					let consensus = Self::do_calculate_vote_consensus(content_id, tier_data.0)?;
					Self::do_update_report_status(content_id, consensus)?;
				}

				Self::deposit_event(Event::VoteSubmitted());
				Ok(())
			}


			#[pallet::weight(10_000)]
			pub fn submit_report_consensus_decision(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				decision: bool,
			) -> DispatchResult {
				
				let who = ensure_signed(origin)?;
				let consensus = Self::do_check_if_report_verdict_is_acceptable(&content_id)?;
				let is_reporter = Self::do_check_if_reporter_or_reportee(&who, &content_id)?;
				Self::do_can_user_accept_verdict(is_reporter, &content_id)?;
				let tier_data = Self::do_get_current_report_tier_data(&content_id)?;
				let report_status : ReportStatus;
				
				if decision == false { // appeal verdict
					ensure!(tier_data.2 == true, Error::<T>::ReportAppealLimitReached); // tier_data.2 == is_appealable
					
					let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;
					Self::do_transfer_funds_to_treasury(&who, reward_pool.0)?;
					report_status = Self::do_get_report_status_on_appeal(is_reporter)?;
					Self::deposit_event(Event::ReportAppealed(content_id));
				}
					
				else { // accept verdict
					let reporter_id = Self::do_get_reporter(content_id)?;
					let reportee_id = Self::do_get_reportee(content_id)?;
					report_status = Self::do_get_report_status_on_accept(is_reporter)?;
					
					if !is_reporter {
						let reportee_slash = Self::do_convert_collateral_to_balance()?;
						Self::do_grab_reportee_collateral(content_id, reportee_slash)?;
					}

					for tier in tier_data.3 { // iterate all existing tiers
						let report_voters = Self::do_get_report_voters_by_vote(content_id, tier, consensus)?;
						
						let reward_pool = Self::do_get_total_moderation_pool(content_id, tier)?;
						let majority_voter_reward = Self::do_calculate_majority_voter_reward(reward_pool, report_voters.0.len() as u32)?;
						
						Self::do_distribute_rewards_to_majority_voters(report_voters.0, majority_voter_reward)?;
						Self::do_slash_tokens_from_minority_voters(report_voters.1)?;
						//TODO handle reputation/token threshold for each voter
						
						if is_reporter {
							let reportee_reward = Self::do_calculate_reportee_reward(reward_pool)?;
							Self::do_transfer_funds_from_treasury(&reportee_id, reportee_reward)?;
						}
						else {
							let reporter_reward = Self::do_calculate_reporter_reward(reward_pool)?;
							Self::do_transfer_funds_from_treasury(&reporter_id, reporter_reward)?;
							// TODO suspend content
						}
					} 
					Self::deposit_event(Event::ReportClosed(content_id, report_status));
				}
				Self::do_update_report_status(content_id, report_status)?;
				
				Ok(())
			}


			#[pallet::weight(10_000)]
			pub fn submit_report_appeal_decision(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				decision: bool,
			) -> DispatchResult {
			
				let who = ensure_signed(origin)?;
				let consensus = Self::do_check_if_report_appeal_is_acceptable(&content_id)?;
				let is_reporter = Self::do_check_if_reporter_or_reportee(&who, &content_id)?;
				Self::do_can_user_accept_appeal(is_reporter, &content_id)?;
				let tier_data = Self::do_get_current_report_tier_data(&content_id)?;

				if decision == false {
					let reporter_id = Self::do_get_reporter(content_id)?;
					let reportee_id = Self::do_get_reportee(content_id)?;

					if !is_reporter {
						let reportee_slash = Self::do_convert_collateral_to_balance()?;
						Self::do_grab_reportee_collateral(content_id, reportee_slash)?;
					}
					
					for tier in tier_data.3 { // TODO extract to helper funcs
						let report_voters = Self::do_get_report_voters_by_vote(content_id, tier, consensus)?;
						let reward_pool = Self::do_get_total_moderation_pool(content_id, tier)?;
						let majority_voter_reward = Self::do_calculate_majority_voter_reward(reward_pool, report_voters.0.len() as u32)?;
						
						Self::do_distribute_rewards_to_majority_voters(report_voters.0, majority_voter_reward)?;
						Self::do_slash_tokens_from_minority_voters(report_voters.1)?;
						
						if is_reporter {
							let reportee_reward = Self::do_calculate_reportee_reward(reward_pool)?;
							Self::do_transfer_funds_from_treasury(&reportee_id, reportee_reward)?;
							Self::do_update_report_status(content_id, ReportStatus::Refused)?;
						}
						else {
							let reporter_reward = Self::do_calculate_reporter_reward(reward_pool)?;
							Self::do_transfer_funds_from_treasury(&reporter_id, reporter_reward)?;
							Self::do_update_report_status(content_id, ReportStatus::Accepted)?;
						}
					}
					
					let appeal_fee = BalanceOf::<T>::from(Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?.0);
					if is_reporter {
						Self::do_transfer_funds_from_treasury(&reportee_id, appeal_fee)?;
					}
					else {
						Self::do_transfer_funds_from_treasury(&reporter_id, appeal_fee)?;
					}
					Self::deposit_event(Event::ReportAppealRefused(content_id));

				}

				else {
					let appeal_fee = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?.0;
					if is_reporter {
						let reporter_id = Self::do_get_reporter(content_id)?;
						Self::do_transfer_funds_to_treasury(&reporter_id, appeal_fee)?;
					}
					else {
						let reportee_id = Self::do_get_reportee(content_id)?;
						Self::do_transfer_funds_to_treasury(&reportee_id, appeal_fee)?;
					}

					let moderator_btree = Self::do_get_moderator_btree()?;
					let drafted_moderators = Self::do_draft_moderators_from_btree(moderator_btree, T::TotalTierOneModerators::get())?;
					let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;

					Self::do_create_report_verdict(content_id, tier_data.1, reward_pool.1)?;
					Self::do_assign_report_to_moderators(content_id, drafted_moderators)?;
					Self::do_update_report_status(content_id, ReportStatus::InResolution)?;
					Self::deposit_event(Event::ReportAppealAccepted(content_id));
				}
				
				Ok(())
			}



		}
	
	
	
	//** Helpers **//
	
		impl<T: Config> Pallet<T> {

			//* Moderator *//


				pub fn do_check_if_already_moderator(
					who: &T::AccountId, // TODO verify if & is needed
				) -> Result<(), DispatchError> {
					
					frame_support::ensure!(!Moderators::<T>::contains_key(who), Error::<T>::AlreadyRegisteredAsModerator);
					Ok(())
				} 


				pub fn do_create_moderator(
					who: &T::AccountId, // TODO verify if & is needed
				) -> Result<(), DispatchError> {
					
					let empty_bounded_reports: BoundedVec<T::ContentId, T::MaxReportsByModerator>
						= TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
					let moderator = Moderator {	
						collateral_tokens: T::MinimumTokensForModeration::get(),
						assigned_reports: empty_bounded_reports,
						rank: ModeratorRank::Junior,
						total_points: 0u32,
					};
					
					Moderators::<T>::insert(who, moderator.clone());
					pallet_stat_tracker::Pallet::<T>::register_new_wallet(who)?;
					
					Ok(())
				} 


				pub fn do_get_moderator_btree(
				) -> Result<BTreeMap<T::AccountId, u32>, DispatchError> { 
					
					let moderator_data: Vec<T::AccountId> = Moderators::<T>::iter().map(|(x, _)| x).collect(); //TODO optimize
					// TODO add iter keys, this version does not support them (https://github.com/paritytech/substrate/pull/9238))
					
					ensure!(moderator_data.size_hint() >= T::TotalTierOneModerators::get() as usize, Error::<T>::NotEnoughModeratorsRegistered);
					let btree = pallet_stat_tracker::Pallet::<T>::create_moderator_btree(moderator_data).unwrap();

					Ok(btree)
				} 	
				

				pub fn do_draft_moderators_from_btree(
					btree: BTreeMap<T::AccountId, u32>,
					required_moderators: u32,
				) -> Result<Vec<T::AccountId>, DispatchError> { // BTreeMap<T::AccountId, u32>

					let drafted_moderators = btree.into_keys().take(required_moderators as usize).collect();
					Ok(drafted_moderators)
				} 	
				

				pub fn do_assign_report_to_moderators(
					content_id: T::ContentId,
					moderators: Vec<T::AccountId>,
				) -> Result<(), DispatchError> {
					
					for moderator_id in moderators.iter() { // assign the report to each selected moderator
						Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
							let moderator  = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
							moderator.assigned_reports.try_push(content_id).unwrap();
							//TODO optionally send a push notification in the next hook
							Ok(())
						})?;
					}
					
					Ok(())
				} 


				pub fn do_deallocate_moderator_from_report(
					moderator_id: &T::AccountId,
					content_id: &T::ContentId,
				) -> Result<(), DispatchError> {
					
					Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
						let mod_data = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
						mod_data.assigned_reports.retain(|assigned_id| assigned_id != content_id);
						Ok(())
					})?;
					
					Ok(())
				} 


				pub fn do_can_moderator_suspend(
					who: &T::AccountId,
				) -> Result<(), DispatchError> {
					
					let moderator = Moderators::<T>::try_get(who).unwrap();
					frame_support::ensure!(moderator.assigned_reports.len() == 0, Error::<T>::ReportsAwaitingVote);

					Ok(())
				} 
				

				pub fn do_suspend_moderation(
					who: &T::AccountId,
				) -> Result<(), DispatchError> {
					
					let moderator = Moderators::<T>::try_get(who).unwrap();
					Self::do_transfer_funds_from_treasury(&who, moderator.collateral_tokens)?;
					Moderators::<T>::remove(who);
					Ok(())
				} 


				pub fn do_can_moderator_vote(
					who: &T::AccountId,
					content_id: &T::ContentId,
				) -> Result<(), DispatchError> {
					
					let assigned_reports = Moderators::<T>::try_get(who).unwrap().assigned_reports;
					ensure!(assigned_reports.contains(content_id), Error::<T>::ModeratorNotDraftedForReport);

					Ok(())
				} 



			//* Report *//

				pub fn do_validate_report_data(
					content_id: T::ContentId,
					_justification:&BoundedVec<u8, T::JustificationLimit>,
				) -> Result<(), DispatchError> {
					
					ensure!(!Reports::<T>::contains_key(content_id.clone()), Error::<T>::ReportAlreadyOngoing);
					//TODO check if the justification is not empty
					Ok(())
				} 
				

				pub fn do_check_if_report_verdict_is_acceptable(
					content_id: &T::ContentId,
				) -> Result<ReportStatus, DispatchError> {
					
					let report = Reports::<T>::try_get(content_id).unwrap();
					ensure!(
						(report.status == ReportStatus::MajorityVotedFor || report.status == ReportStatus::MajorityVotedAgainst), 
						Error::<T>::NonexistentReport
					);
					Ok(report.status)
				} 
				
				
				pub fn do_check_if_report_appeal_is_acceptable(
					content_id: &T::ContentId,
				) -> Result<ReportStatus, DispatchError> {
					
					let report = Reports::<T>::try_get(content_id).unwrap();
					ensure!(
						(report.status == ReportStatus::AppealedByReporter || report.status == ReportStatus::AppealedByReportee), 
						Error::<T>::NonexistentReport
					);
					Ok(report.status)
				} 
				
					
				pub fn do_create_report(
					who: &T::AccountId,
					content_id: T::ContentId,
					reportee_id: &T::AccountId,
					content_type: ContentType,
					infringiment: InfringimentType,
					justification:BoundedVec<u8, T::JustificationLimit>,
				) -> Result<(), DispatchError> {

					//TODO hash content_id together with the content_type
					let report = Report {
							reporter_id: who.clone(),
							reportee_id: reportee_id.clone(),
							content_type: content_type.clone(),
							infringiment: infringiment.clone(),
							justification: justification.clone(),
							status: ReportStatus::InResolution,
						};
					Reports::<T>::insert(content_id, report.clone());
			
					Ok(())
				} 
				
					
				pub fn do_create_report_verdict(
					content_id: T::ContentId,
					tier: Tiers,
					reward_pool: BalanceOf<T>,
				) -> Result<(), DispatchError> {
					
					let empty_bounded_votes: BoundedVec<Vote<T::AccountId>, T::MaxReportsByTier>
						= TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?; // new empty BoundedVec
					let report_outcome = ReportOutcome {
						staked_tokens: reward_pool,
						required_votes: T::TotalTierOneModerators::get(),
						votes_for: 0,
						votes: empty_bounded_votes,
					};
					ReportVerdicts::<T>::insert(content_id, tier, report_outcome);
			
					Ok(())
				} 
			

				pub fn do_get_current_report_tier_data(
					content_id: &T::ContentId,
				) -> Result<(Tiers, Tiers, bool, Vec<Tiers>), DispatchError> {
					
					ensure!(ReportVerdicts::<T>::contains_key(content_id, Tiers::TierOne), Error::<T>::NonexistentReport);
					let mut current_tier = Tiers::TierOne;
					let mut next_tier = Tiers::TierTwo; // TODO refactor this function
					let mut is_appealable = true;
					let mut all_tiers : Vec<Tiers> = vec![Tiers::TierOne];

					if ReportVerdicts::<T>::contains_key(content_id, Tiers::TierThree) { //TODO ensure the report exists
						current_tier = Tiers::TierThree; 
						next_tier = Tiers::TierThree;
						all_tiers.append(&mut vec![Tiers::TierTwo, Tiers::TierThree]);
						is_appealable = false
					}
					else if ReportVerdicts::<T>::contains_key(content_id, Tiers::TierTwo) {
						current_tier = Tiers::TierTwo; 
						next_tier = Tiers::TierThree;
						all_tiers.append(&mut vec![Tiers::TierTwo]);
					}
					// ReportVerdicts::<T>::iter_key_prefix(|content_id| {});  // TODO not implemented for this version

					Ok((current_tier, next_tier, is_appealable, all_tiers))
				} 
			
				
				pub fn do_update_report_status(
					content_id: T::ContentId,
					report_status: ReportStatus,
				) -> Result<(), DispatchError> {
					
					Reports::<T>::try_mutate_exists(content_id, |report| -> DispatchResult {
						let mut rep = report.as_mut().ok_or(Error::<T>::NonexistentReport)?;
						rep.status = report_status;
						Ok(())
					})
				} 
				
			
				pub fn do_check_if_reporter_or_reportee(
					who: &T::AccountId,
					content_id: &T::ContentId,
				) -> Result<bool, DispatchError> {
					
					let mut is_reporter = true;

					let report = Reports::<T>::try_get(content_id).unwrap();
					if report.reportee_id == *who {
						is_reporter = false;
					}
					else { ensure!(report.reporter_id == *who, Error::<T>::UserCannotAcceptVerdict) }

					Ok(is_reporter)
				} 		
			
			
				pub fn do_can_user_accept_verdict(
					is_reporter: bool,
					content_id: &T::ContentId,
				) -> Result<(), DispatchError> {
					
					let report = Reports::<T>::try_get(content_id).unwrap();
					if report.status == ReportStatus::MajorityVotedFor {
						ensure!(!is_reporter, Error::<T>::UserCannotAcceptVerdict);
					} else if report.status == ReportStatus::MajorityVotedAgainst {
						ensure!(is_reporter, Error::<T>::UserCannotAcceptVerdict);
					}

					Ok(())
				} 		
				
			
				pub fn do_can_user_accept_appeal(
					is_reporter: bool,
					content_id: &T::ContentId,
				) -> Result<(), DispatchError> {
					
					let report = Reports::<T>::try_get(content_id).unwrap();
					if report.status == ReportStatus::AppealedByReporter {
						ensure!(!is_reporter, Error::<T>::UserCannotAcceptVerdict);
					} else if report.status == ReportStatus::AppealedByReportee {
						ensure!(is_reporter, Error::<T>::UserCannotAcceptVerdict);
					}

					Ok(())
				} 		
				

				pub fn do_get_reporter(
					content_id: T::ContentId,
				) -> Result<T::AccountId, DispatchError> {

					let report = Reports::<T>::get(content_id).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report.reporter_id) // return (reporter/reportee's reward, majority voter rewards per capita)
				}

				
				pub fn do_get_reportee(
					content_id: T::ContentId,
				) -> Result<T::AccountId, DispatchError> {

					let report = Reports::<T>::get(content_id).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report.reportee_id) // return (reporter/reportee's reward, majority voter rewards per capita)
				}

				
				pub fn do_get_report_status_on_accept(
					is_reporter: bool,
				) -> Result<ReportStatus, DispatchError> {

					let report_status : ReportStatus;
					if is_reporter {
						report_status = ReportStatus::Refused;
					}
					else {
						report_status = ReportStatus::Accepted;
					}

					Ok(report_status)
				}

				
				pub fn do_get_report_status_on_appeal(
					is_reporter: bool,
				) -> Result<ReportStatus, DispatchError> {

					let report_status : ReportStatus;
					if is_reporter {
						report_status = ReportStatus::AppealedByReporter;
					}
					else {
						report_status = ReportStatus::AppealedByReportee;
					}

					Ok(report_status)
				}



			//* Vote *//
			
				pub fn do_create_vote(
					content_id: T::ContentId,
					tier: Tiers,
					who: &T::AccountId,
					is_for: VoteChoice,
				) -> Result<(), DispatchError> {

					let vote = Vote {
						voter: who.clone(),
						is_for: is_for,
					};

					ReportVerdicts::<T>::try_mutate_exists(content_id, tier, |report_outcome| -> DispatchResult {
						let outcome = report_outcome.as_mut().ok_or(Error::<T>::NonexistentReport)?;
						outcome.votes.try_push(vote).unwrap();
						if is_for == VoteChoice::For { 
							outcome.votes_for = outcome.votes_for.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;
						};
						Ok(())
					}) //TODO add ok_or
				} 
				
				
				pub fn do_are_all_votes_submitted(
					content_id: T::ContentId,
					tier: Tiers,
				) -> Result<bool, DispatchError> {

					let mut are_all_votes_submitted = false;
					let verdict = ReportVerdicts::<T>::get(content_id, tier).ok_or(Error::<T>::NonexistentReport)?;
					if verdict.votes.len() == (verdict.required_votes as usize) {are_all_votes_submitted = true;}

					Ok(are_all_votes_submitted)
				} 
				

				pub fn do_calculate_vote_consensus(
					content_id: T::ContentId,
					tier: Tiers,
				) -> Result<ReportStatus, DispatchError> {
					
					let mut result = ReportStatus::MajorityVotedAgainst;
					let vote_consensus = ReportVerdicts::<T>::get(content_id, tier).ok_or(Error::<T>::NonexistentReport)?;
					if (vote_consensus.votes_for * 1000) > ((vote_consensus.required_votes * 1000) / 2) as u32  { // TODO use arithmetic
						result = ReportStatus::MajorityVotedFor;
					}
					
					Ok(result)
				}


				pub fn do_get_report_voters_by_vote(
					content_id: T::ContentId,
					tier: Tiers,
					consensus: ReportStatus,
				) -> Result<(Vec<T::AccountId>, Vec<T::AccountId>), DispatchError> {
					
					let mut majority_votes = ReportVerdicts::<T>::try_get(content_id, tier).unwrap().votes.clone();
					let mut minority_votes = majority_votes.clone();

					if consensus == ReportStatus::MajorityVotedFor || consensus == ReportStatus::AppealedByReportee {
						majority_votes.retain(|vote| vote.is_for == VoteChoice::For); 
						minority_votes.retain(|vote| vote.is_for == VoteChoice::Against); 
						// TODO use drain_filter (currently unstable) instead of retain
					} 
					else if consensus == ReportStatus::MajorityVotedAgainst || consensus == ReportStatus::AppealedByReporter {
						majority_votes.retain(|vote| vote.is_for == VoteChoice::Against); 
						minority_votes.retain(|vote| vote.is_for == VoteChoice::For); 
					}
					
					let majority_voters: Vec<T::AccountId> = majority_votes.into_iter().map(|vote| vote.voter ).collect();
					let minority_voters: Vec<T::AccountId> = minority_votes.into_iter().map(|vote| vote.voter ).collect();
					
					Ok((majority_voters, minority_voters))
				}



			//* Treasury *//

				pub fn do_transfer_funds_to_treasury(
					who: &T::AccountId,
					amount: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					let treasury = &Self::account_id();
					T::Currency::transfer(
						who, treasury,
						BalanceOf::<T>::from(amount), KeepAlive,
					)?;

					Ok(())
				}


				pub fn do_transfer_funds_from_treasury(
					who: &T::AccountId,
					amount: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					let treasury = &Self::account_id();
					T::Currency::transfer(
						&treasury, who,
						amount, KeepAlive,
					)?;

					Ok(())
				}
			

				pub fn do_grab_reportee_collateral(
					content_id: T::ContentId,
					collateral: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					let report = Reports::<T>::get(content_id).ok_or(Error::<T>::NonexistentReport)?;
					let reportee = report.reportee_id;

					T::Currency::unreserve(&reportee, collateral); //TODO handle insufficient balance
					let treasury = &Self::account_id();
					T::Currency::transfer(
						&reportee, treasury,
						collateral, KeepAlive,
					)?;

					Ok(())
				}
			

				pub fn do_claim_moderation_reward(
					moderator_id: &T::AccountId,
				) -> Result<BalanceOf<T>, DispatchError> {

					let mut reward = BalanceOf::<T>::from(0u32);
					let min_collateral =  T::MinimumTokensForModeration::get();

					Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
						let moderator = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
						let mod_collateral = moderator.collateral_tokens;
						
						if mod_collateral > min_collateral {
							reward = mod_collateral.checked_sub(&min_collateral).ok_or(Error::<T>::Overflow)?;
							moderator.collateral_tokens = mod_collateral.checked_sub(&reward).ok_or(Error::<T>::Overflow)?;
						}

						Ok(())
					})?;

					Ok(reward)
				}



			//* Reward Distribution *//

				pub fn do_get_total_moderation_pool(
					content_id: T::ContentId,
					tier: Tiers,
				) -> Result<BalanceOf<T>, DispatchError> {

					let report_outcome = ReportVerdicts::<T>::get(content_id, tier).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report_outcome.staked_tokens) // return (reporter/reportee's reward, majority voter rewards per capita)
				}
			

				pub fn do_distribute_rewards_to_majority_voters(
					majority_voters: Vec<T::AccountId>,
					reward: BalanceOf<T>,
				) -> Result<(), DispatchError> {	
					
					for moderator_id in majority_voters.iter() {
						Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
							let moderator = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
							moderator.collateral_tokens = moderator.collateral_tokens.checked_add(&reward).ok_or(Error::<T>::Overflow)?;
							moderator.total_points = moderator.total_points.checked_add(1u32).ok_or(Error::<T>::Overflow)?;
							if moderator.total_points > 9 {
								moderator.rank = ModeratorRank::Senior;
							}
							Ok(())
						})?;
					}

					Ok(())
				}


				pub fn do_slash_tokens_from_minority_voters(
					minority_voters: Vec<T::AccountId>,
				) -> Result<(), DispatchError> {	
					let moderator_fee: BalanceOf<T> = Self::do_calculate_moderator_fee()?;
					for moderator_id in minority_voters.iter() {
						Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
							let moderator  = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
							moderator.collateral_tokens = moderator.collateral_tokens.checked_sub(&moderator_fee).ok_or(Error::<T>::Overflow)?;
							moderator.total_points = moderator.total_points.checked_sub(1u32).ok_or(Error::<T>::Overflow)?;
							if moderator.total_points < 10 {
								moderator.rank = ModeratorRank::Junior;
							}
							Ok(())
						})?;
					}
					Ok(())
				}

			//* Reward Calculation *//

				pub fn do_calculate_report_pool(
					total_moderators: u32,
				) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
					// one part reportee, one part reporter, one part split among moderators
					let moderator_fee = Self::do_calculate_moderator_fee()?;
					let report_pool_third = moderator_fee.saturating_mul(total_moderators.into()); // in tier one this is the same as MovieCollateral
					let report_pool_total = moderator_fee.saturating_mul(3u32.into());
					Ok((report_pool_third, report_pool_total))
				}


				pub fn do_calculate_reportee_reward(
					reward_pool: BalanceOf<T>,
				) -> Result<BalanceOf<T>, DispatchError> {

					let half_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(2u32))
						.ok_or(Error::<T>::Overflow)?;
					let one_third_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(3u32))
						.ok_or(Error::<T>::Overflow)?;
					let reportee_reward = 
						half_pool
						.checked_sub(&one_third_pool)
						.ok_or(Error::<T>::Overflow)?;
					Ok(reportee_reward)
				}


				pub fn do_calculate_reporter_reward(
					reward_pool: BalanceOf<T>,
				) -> Result<BalanceOf<T>, DispatchError> {

					let half_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(2u32))
						.ok_or(Error::<T>::Overflow)?;
					Ok(half_pool)
				}


				pub fn do_calculate_majority_voter_reward(
					reward_pool: BalanceOf<T>,
					total_majority_voters: u32,
				) -> Result<BalanceOf<T>, DispatchError> {

					let tokens_per_voter = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(total_majority_voters))
						.ok_or(Error::<T>::Overflow)?;
					let moderator_fee = BalanceOf::<T>::from(Self::do_calculate_moderator_fee()?);
					
					let majority_voter_reward = 
						tokens_per_voter
						.checked_sub(&moderator_fee)
						.ok_or(Error::<T>::Overflow)?;
					Ok(majority_voter_reward)
				}


				pub fn do_calculate_moderator_fee(
				) -> Result<BalanceOf<T>, DispatchError> {
					// let opaque = value.saturating_mul(_1000_balance).checked_div(total_value)?;
					
					let movie_collateral: BalanceOf<T> = T::MovieCollateral::get();
					let moderator_fee = 
						movie_collateral
						.checked_div(&BalanceOf::<T>::from(T::TotalTierOneModerators::get()))
						.ok_or(Error::<T>::Overflow)?;
					
					Ok(moderator_fee)
				}

				pub fn do_convert_collateral_to_balance(
				) -> Result<BalanceOf<T>, DispatchError> {

					let collateral_balance = BalanceOf::<T>::from(T::MovieCollateral::get());
					Ok(collateral_balance)
				}

			//* Utils *//

				//TODO implement hash_content_id

				// The account ID of the vault
				fn account_id() -> T::AccountId {
					<T as Config>::PalletId::get().try_into_account().unwrap()
				}
		
		}
	
}