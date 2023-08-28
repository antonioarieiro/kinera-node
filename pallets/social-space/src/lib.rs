//** About **//
	// Information regarding the pallet
	//TODO transform u8 into boudned string

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

			use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
			use frame_system::pallet_prelude::*;
			use sp_runtime::{RuntimeDebug, traits::{AtLeast32BitUnsigned, CheckedAdd, One,Zero}};
			use codec::{Decode, Encode, MaxEncodedLen};
			use scale_info::TypeInfo;
			use frame_support::BoundedVec;
			use core::convert::TryInto;
			use scale_info::prelude::vec::Vec;
	
		//* Config *//
		
			#[pallet::pallet]
			#[pallet::generate_store(pub(super) trait Store)]
			pub struct Pallet<T>(_);
	
			#[pallet::config]
			pub trait Config: frame_system::Config {
				type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
				#[pallet::constant]
				type StringLimit: Get<u32>;
				type MaxFollowers: Get<u32>;
				type PostId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
				type CommentId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
			}



	//** Types **//	
	
		//* Types *//
		//* Constants *//
		//* Enums *//
		//* Structs *//

			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct Space<AccountId,BoundedString, BoundedFollowerList> {
				pub owner: AccountId,
				pub name:BoundedString,
				pub description:BoundedString,
				pub image_link: BoundedString,
				pub following : BoundedFollowerList,
				pub followers : BoundedFollowerList,
			}

			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct Post<AccountId,PostId,BoundedString,BlockNumber> {
				pub owner: AccountId,
				pub id: PostId,
				pub content:BoundedString,
				pub image_ipfs: BoundedString,
				pub video_ipfs: BoundedString,
				pub likes : u32,
				pub dislikes : u32,
				pub date : BlockNumber
			}

			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct Comment<AccountId,BoundedString,PostId,CommentId,BlockNumber> {
				pub owner:AccountId,
				pub id: CommentId,
				pub content: BoundedString,
				pub post_id : PostId,
				pub parent_comment_id: CommentId,
				pub likes: u32,
				pub dislikes:u32,
				pub date : BlockNumber
			}    
	
	//** Storage **//

		#[pallet::storage]
		#[pallet::getter(fn spaces)]
		pub type Spaces<T: Config> = StorageMap<
			_, 
			Blake2_128Concat, T::AccountId, 
			Space<
				T::AccountId,
				BoundedVec<u8, T::StringLimit>,
				BoundedVec<T::AccountId, T::MaxFollowers>,
			>,
		>;

		#[pallet::storage]
		#[pallet::getter(fn next_post_id)]
		pub(super) type NextPostId<T: Config> = StorageValue<
			_, 
			T::PostId, ValueQuery
		>;


		#[pallet::storage]
		#[pallet::getter(fn next_comment_id)]
		pub(super) type NextCommentId<T: Config> = StorageValue<
			_, 
			T::CommentId, ValueQuery
		>;


		#[pallet::storage]
		#[pallet::getter(fn posts)]
		pub type Posts<T: Config> = StorageMap<
			_, 
			Blake2_128Concat, T::PostId, 
			Post<T::AccountId,T::PostId,BoundedVec<u8, T::StringLimit>,T::BlockNumber>
		>;

		
		#[pallet::storage]
		#[pallet::getter(fn comments)]
		pub type Comments<T: Config> = StorageDoubleMap<
			_,
			Blake2_128Concat,T::PostId, 
			Blake2_128Concat,T::CommentId, 
			Comment<T::AccountId,BoundedVec<u8, T::StringLimit>,T::PostId,T::CommentId,T::BlockNumber>
		>;



	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			SomethingStored(u32, T::AccountId),
			SpaceCreated(T::AccountId),
			SpaceFollowed(T::AccountId,T::AccountId),
			PostCreated(T::AccountId,T::PostId),
			PostCommented(T::AccountId,T::PostId),
			CommmentReplied(T::AccountId,T::CommentId,T::CommentId),
		}



	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			NoneValue,
			StorageOverflow,
			BadMetadata,
			Overflow,

			ProfileAlreadyCreated,
			SpaceNotFound,
			AlreadyFollowing,
			PostNotFound,
		}



	//** Hooks **//

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


	//** Extrinsics **//

		#[pallet::call]
		impl<T:Config> Pallet<T> {

			#[pallet::weight(10_000)]
			pub fn create_space(
				origin: OriginFor<T>,
				name:Vec<u8>,
				description:Vec<u8>,
			) -> DispatchResult {
				let who = ensure_signed(origin)?;

				Self::do_create_space(&who, name,description)?;

				Ok(())

			}
			#[pallet::weight(10_000)]
			pub fn create_post(
				origin: OriginFor<T>,
				content: Vec<u8>,
				image_ipfs: Vec<u8>,
				video_ipfs:Vec<u8>,
				)-> DispatchResult{
				
				let who = ensure_signed(origin)?;

				Self::do_create_post(&who,content,image_ipfs,video_ipfs)?;

				Ok(())

			}

			#[pallet::weight(10_000)]
			pub fn follow_space(
				origin: OriginFor<T>,
				target: T::AccountId,
				)-> DispatchResult {

				let who = ensure_signed(origin)?;

				Self::do_follow_space(who, target)?;

				Ok(())

			}

			#[pallet::weight(10_000)]
			pub fn comment_on_post(
				origin: OriginFor<T>,
				post_id: T::PostId,
				content: Vec<u8>,
				)-> DispatchResult {
				
				let who = ensure_signed(origin)?;

				Self::do_comment_on_post(&who,post_id,content)?;
				

				Ok(())
			}

			#[pallet::weight(10_000)]
			pub fn comment_on_comment(
				origin: OriginFor<T>,
				post_id: T::PostId,
				parent_comment_id: T::CommentId,
				content: Vec<u8>,
				)-> DispatchResult {

				let who = ensure_signed(origin)?;

				Self::do_comment_on_comment(&who,post_id,parent_comment_id,content)?;

				Ok(())
				
			}

		}


	//** Helpers **//

       impl<T: Config> Pallet<T> {

			pub fn do_create_space(
				who: &T::AccountId,
				name:Vec<u8>,
				description:Vec<u8>,
			) -> DispatchResult {

				ensure!(!Spaces::<T>::contains_key(who.clone()), Error::<T>::ProfileAlreadyCreated);

				let bounded_name: BoundedVec<u8, T::StringLimit> =
					name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_description: BoundedVec<u8, T::StringLimit> =
					description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_image_link: BoundedVec<u8, T::StringLimit> =
					description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_following_list: BoundedVec<T::AccountId, T::MaxFollowers> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_follower_list: BoundedVec<T::AccountId, T::MaxFollowers> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;


				let space = Space{
					owner:who.clone(),
					name:bounded_name,
					description:bounded_description,
					image_link: bounded_image_link,
					following: bounded_following_list,
					followers: bounded_follower_list,

				};
				
				Spaces::<T>::insert(who.clone(),space);
				Self::deposit_event(Event::SpaceCreated(who.clone()));

				Ok(())
			}

			pub fn do_create_post(
				who: &T::AccountId,
				content:Vec<u8>,
				image_ipfs:Vec<u8>,
				movie_ipfs:Vec<u8>,
			) -> DispatchResult {
				let post_id =
				NextPostId::<T>::try_mutate(|id| -> Result<T::PostId, DispatchError> {
					let current_id = *id;
					*id = id
					.checked_add(&One::one())
					.ok_or(Error::<T>::Overflow)?;
					Ok(current_id)
					})?;

				let bounded_content: BoundedVec<u8, T::StringLimit> =
					content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		
				let bounded_image: BoundedVec<u8, T::StringLimit> =
					image_ipfs.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
				let bounded_movie: BoundedVec<u8, T::StringLimit> =
					movie_ipfs.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
				
				let post = Post {
					owner: who.clone(),
					id: post_id.clone(),
					content: bounded_content,
					image_ipfs:bounded_image,
					video_ipfs: bounded_movie,
					likes: Zero::zero(),
					dislikes: Zero::zero(),
					date : <frame_system::Pallet<T>>::block_number()

				};
				
				Posts::<T>::insert(post_id.clone(),post);
				Self::deposit_event(Event::PostCreated(who.clone(),post_id));

				Ok(())    
			
			}

			pub fn do_follow_space(
				who: T::AccountId,
				other: T::AccountId
			) -> DispatchResult {

				Spaces::<T>::try_mutate_exists(who.clone(), |origin| -> DispatchResult {
					
					let origin_space = origin.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
					ensure!(!origin_space.following.contains(&other.clone()), Error::<T>::AlreadyFollowing);

					Spaces::<T>::try_mutate_exists(other.clone(), |target| -> DispatchResult {

						let target_space = target.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
						ensure!(!target_space.followers.contains(&who.clone()), Error::<T>::AlreadyFollowing);

						origin_space.following.try_push(other.clone()).unwrap();
						target_space.followers.try_push(who.clone()).unwrap();

						Ok(())
					});

					Ok(())
				});

				Ok(())
			}
			
			pub fn do_comment_on_post(
				who: &T::AccountId,
				post_id: T::PostId,
				content: Vec<u8>
			) -> DispatchResult {

				let comment_id=NextCommentId::<T>::try_mutate(|id| -> Result<T::CommentId, DispatchError> {
					
					if *id==Zero::zero() {
						
						*id=One::one();
					}

					let current_id = *id;
					*id = id
					.checked_add(&One::one())
					.ok_or(Error::<T>::Overflow)?;
					Ok(current_id)
					})?;

					
				let bounded_content: BoundedVec<u8, T::StringLimit> =
					content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
				
				let comment = Comment{
					owner : who.clone(),
					id : comment_id.clone(),
					content : bounded_content,
					post_id : post_id.clone(),
					parent_comment_id : Zero::zero(),
					likes: Zero::zero(),
					dislikes: Zero::zero(),
					date : <frame_system::Pallet<T>>::block_number()
		
				};
				
				//ensure such comment does no exists
				ensure!(!Comments::<T>::contains_key(post_id.clone(),comment_id.clone()),Error::<T>::NoneValue);
				
				//ensure post exists
				ensure!(Posts::<T>::contains_key(post_id.clone()),Error::<T>::PostNotFound);

				Comments::<T>::insert(post_id.clone(),comment_id,comment);
				Self::deposit_event(Event::PostCommented(who.clone(),post_id));

				Ok(())
			}


			pub fn do_comment_on_comment(
				who: &T::AccountId,
				post_id: T::PostId,
				parent_comment_id : T::CommentId,
				content: Vec<u8>
			) -> DispatchResult {

				//ensure post exists and parent comment exists 
				ensure!(Posts::<T>::contains_key(post_id.clone()) && Comments::<T>::contains_key(post_id.clone(),parent_comment_id.clone()),Error::<T>::PostNotFound);

				let comment_id=NextCommentId::<T>::try_mutate(|id| -> Result<T::CommentId, DispatchError> {
					
					if *id==Zero::zero() {
						
						*id=One::one();
					}

					let current_id = *id;
					*id = id
					.checked_add(&One::one())
					.ok_or(Error::<T>::Overflow)?;
					Ok(current_id)
					})?;


				let bounded_content: BoundedVec<u8, T::StringLimit> =
					content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
			
				
				let comment = Comment{
					owner : who.clone(),
					id: comment_id.clone(),
					content : bounded_content,
					post_id : post_id.clone(),
					parent_comment_id : parent_comment_id.clone(),
					likes: Zero::zero(),
					dislikes: Zero::zero(),
					date : <frame_system::Pallet<T>>::block_number()
		
				};

				Comments::<T>::insert(post_id,comment_id.clone(),comment);
				Self::deposit_event(Event::CommmentReplied(who.clone(),parent_comment_id,comment_id));
				Ok(())   
			}
    
		}


}