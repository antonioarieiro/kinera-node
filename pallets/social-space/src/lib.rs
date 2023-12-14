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
			pub trait Config: frame_system::Config + pallet_stat_tracker::Config {
				type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
				#[pallet::constant]

				type DescStringLimit: Get<u32>;
				type LinkStringLimit: Get<u32>;
				type CommentStringLimit: Get<u32>;
				type MaxPostsPerSpace: Get<u32>;
				
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
			pub struct Space<AccountId, NameString, DescString, LinkString, BoundedFollowerList, BoundedPostList> {
				pub owner: AccountId,
				pub name: NameString,
				pub description: DescString,
				pub image_link: LinkString,
				pub following: BoundedFollowerList,
				pub followers: BoundedFollowerList,
				pub posts: BoundedPostList,
				pub comments: BoundedPostList,
			}
			
			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct SpacePosts<BoundedPostList> {
				pub posts: BoundedPostList,
			}

			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct Post<AccountId, PostId, TextString, LinkString, BlockNumber> {
				pub owner: AccountId,	
				pub id: PostId,
				pub text: TextString,
				pub image_ipfs: LinkString,
				pub video_ipfs: LinkString,
				pub likes: u32,
				pub dislikes: u32,
				pub date: BlockNumber,
			}

			#[derive(Default, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
			pub struct Comment<AccountId,BoundedString,PostId,CommentId,BlockNumber> {
				pub owner:AccountId,
				pub id: CommentId,
				pub text: BoundedString,
				pub post_id: PostId,
				pub parent_comment_id: CommentId,
				pub likes: u32,
				pub dislikes: u32,
				pub date: BlockNumber
			}    
	
	//** Storage **//

		// Social Space

			#[pallet::storage]
			#[pallet::getter(fn spaces)]
			pub type Spaces<T: Config> = StorageMap<
				_, 
				Blake2_128Concat, T::AccountId, 
				Space<
					T::AccountId,
					BoundedVec<u8, T::NameStringLimit>,
					BoundedVec<u8, T::DescStringLimit>,
					BoundedVec<u8, T::LinkStringLimit>,
					BoundedVec<T::AccountId, T::MaxFollowers>,
					BoundedVec<T::PostId, T::MaxPostsPerSpace>,
				>,
			>;


		// Post

			// Keeps track of how many posts exist. This is used since the id of posts is
			// dynamically incremented whenever a new post is created.
			#[pallet::storage]
			#[pallet::getter(fn next_post_id)]
			pub(super) type NextPostId<T: Config> = StorageValue<
				_, 
				T::PostId, 
				ValueQuery
			>;


			// Matches a wallet's account_id (and therefore its space) to the list of posts they have created.
			//TODO this is already stored in the "posts" field of the space, but only by ID
			//TODO this duplicates data
			#[pallet::storage]
			#[pallet::getter(fn posts_by_space)]
			pub type PostsBySpace<T: Config> = StorageMap<
				_, 
				Blake2_128Concat, T::AccountId, 
				SpacePosts<
					BoundedVec<
						Post<
							T::AccountId,
							T::PostId,
							BoundedVec<u8, T::CommentStringLimit>,
							BoundedVec<u8, T::LinkStringLimit>,
							T::BlockNumber,
						>,
						T::MaxPostsPerSpace
					>
				>
			>;


			#[pallet::storage]
			#[pallet::getter(fn posts)]
			pub type Posts<T: Config> = StorageMap<
				_, 
				Blake2_128Concat, T::PostId, 
				Post<
					T::AccountId,
					T::PostId,
					BoundedVec<u8, T::CommentStringLimit>,
					BoundedVec<u8, T::LinkStringLimit>,
					T::BlockNumber,
				>
			>;
			



		// Comments

			#[pallet::storage]
			#[pallet::getter(fn next_comment_id)]
			pub(super) type NextCommentId<T: Config> = StorageValue<
				_, 
				T::CommentId, 
				ValueQuery
			>;

			#[pallet::storage]
			#[pallet::getter(fn comments)]
			pub type Comments<T: Config> = StorageDoubleMap<
				_,
				Blake2_128Concat, T::PostId, 
				Blake2_128Concat, T::CommentId, 
				Comment<
					T::AccountId,
					BoundedVec<u8, T::CommentStringLimit>,
					T::PostId,
					T::CommentId,
					T::BlockNumber
				>
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
			WalletStatsRegistryRequired,
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

			// Create a social space for the wallet if it doesn't already exist.
			// bounded_name -> The social space's name. This is different 
			// from the stat tracker module's name.
			// bounded_description -> A brief description.
			// bounded_image_link -> A link to the avatar image source.
			#[pallet::weight(10_000)]
			pub fn create_space(
				origin: OriginFor<T>,
				bounded_name: BoundedVec<u8, T::NameStringLimit>,
				bounded_description: BoundedVec<u8, T::DescStringLimit>,
				bounded_image_link: BoundedVec<u8, T::LinkStringLimit>,
			) -> DispatchResult {
				
				// validate the origin
				let who = ensure_signed(origin)?;
				// ensure!(
				// 	pallet_stat_tracker::Pallet::<T>::is_wallet_registered(who.clone())?,
				// 	Error::<T>::WalletStatsRegistryRequired,
				// );
				ensure!(
					!Spaces::<T>::contains_key(who.clone()), 
					Error::<T>::ProfileAlreadyCreated
				);
				
				let bounded_following_list: BoundedVec<T::AccountId, T::MaxFollowers> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_follower_list: BoundedVec<T::AccountId, T::MaxFollowers> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
				let bounded_post_list: BoundedVec<T::PostId, T::MaxPostsPerSpace> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;


				let space = Space {
					owner: who.clone(),
					name: bounded_name,
					description: bounded_description,
					image_link: bounded_image_link,
					following: bounded_following_list,
					followers: bounded_follower_list,
					posts: bounded_post_list.clone(),
					comments: bounded_post_list.clone(),
				};
				
				Spaces::<T>::insert(who.clone(), space);

				//TODO already included in the space's "posts" param, but by id and not the actual space struct
				let bounded_post_list: BoundedVec<Post<T::AccountId,T::PostId,BoundedVec<u8, T::CommentStringLimit>,BoundedVec<u8, T::LinkStringLimit>,T::BlockNumber,>, T::MaxPostsPerSpace> =
					TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
				let space_posts = SpacePosts {
					posts: bounded_post_list,
				};

				PostsBySpace::<T>::insert(who.clone(), space_posts);

				
				Self::deposit_event(Event::SpaceCreated(who.clone()));
				Ok(())
			}



			// Create a post on your social space. It can be seen by other people and 
			// replied to. It can also link to an image or video.
			// bounded_text -> 
			// bounded_image_ipfs -> 
			// bounded_video_ipfs -> 
			#[pallet::weight(10_000)]
			pub fn create_post(
				origin: OriginFor<T>,
				bounded_text: BoundedVec<u8, T::CommentStringLimit>,
				bounded_image_ipfs: BoundedVec<u8, T::LinkStringLimit>,
				bounded_video_ipfs: BoundedVec<u8, T::LinkStringLimit>,
				)-> DispatchResult{
				
				let who = ensure_signed(origin)?;
				
				Spaces::<T>::try_mutate_exists(who.clone(), |sp| -> DispatchResult {
					let space = sp.as_mut().ok_or(Error::<T>::BadMetadata)?;

					let post_id =
					NextPostId::<T>::try_mutate(|id| -> Result<T::PostId, DispatchError> {
						let current_id = *id;
						*id = id
							.checked_add(&One::one())
							.ok_or(Error::<T>::Overflow)?;
						Ok(current_id)
					})?;
	
					let post = Post {
						owner: who.clone(),
						id: post_id.clone(),
						text: bounded_text,
						image_ipfs: bounded_image_ipfs,
						video_ipfs: bounded_video_ipfs,
						likes: Zero::zero(),
						dislikes: Zero::zero(),
						date : <frame_system::Pallet<T>>::block_number()
					};
	
					Posts::<T>::insert(post_id.clone(), post.clone());
					space.posts.try_push(post_id.clone()).unwrap();
					
					PostsBySpace::<T>::try_mutate_exists(who.clone(), |pbs| -> DispatchResult {
						let posts_by_space = pbs.as_mut().ok_or(Error::<T>::BadMetadata)?;
						
						posts_by_space.posts.try_push(post).unwrap();
						Ok(())
					})?;


					Self::deposit_event(Event::PostCreated(who.clone(),post_id));
					Ok(())
				})?;
				
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
			pub fn unfollow_space(
				origin: OriginFor<T>,
				other: T::AccountId,
				)-> DispatchResult {

				let who = ensure_signed(origin)?;

				Spaces::<T>::try_mutate_exists(who.clone(), |origin| -> DispatchResult {
					
					let origin_space = origin.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
					ensure!(origin_space.following.contains(&other.clone()), Error::<T>::AlreadyFollowing);

					Spaces::<T>::try_mutate_exists(other.clone(), |target| -> DispatchResult {

						let target_space = target.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
						ensure!(!target_space.followers.contains(&who.clone()), Error::<T>::AlreadyFollowing);

						//TODO optimize
						origin_space.following.retain(|user| user != &other.clone());
						target_space.followers.retain(|user| user != &who.clone());

						Ok(())
					})?;

					Ok(())
				})?;

				Ok(())
			}


			#[pallet::weight(10_000)]
			pub fn comment_on_post(
				origin: OriginFor<T>,
				post_id: T::PostId,
				text: BoundedVec<u8, T::CommentStringLimit>,
				)-> DispatchResult {
				
				let who = ensure_signed(origin)?;

				Self::do_comment_on_post(&who,post_id,text)?;
				

				Ok(())
			}

			
			#[pallet::weight(10_000)]
			pub fn comment_on_comment(
				origin: OriginFor<T>,
				post_id: T::PostId,
				parent_comment_id: T::CommentId,
				text: BoundedVec<u8, T::CommentStringLimit>,
				)-> DispatchResult {

				let who = ensure_signed(origin)?;

				Self::do_comment_on_comment(&who,post_id,parent_comment_id,text)?;

				Ok(())
				
			}

		}


	//** Helpers **//

       impl<T: Config> Pallet<T> {

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
					})?;

					Ok(())
				})?;

				Ok(())
			}

			pub fn do_unfollow_space(
				who: T::AccountId,
				other: T::AccountId
			) -> DispatchResult {

				Spaces::<T>::try_mutate_exists(who.clone(), |origin| -> DispatchResult {
					
					let origin_space = origin.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
					ensure!(origin_space.following.contains(&other.clone()), Error::<T>::AlreadyFollowing);

					Spaces::<T>::try_mutate_exists(other.clone(), |target| -> DispatchResult {

						let target_space = target.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
						ensure!(target_space.followers.contains(&who.clone()), Error::<T>::AlreadyFollowing);

						origin_space.following.retain(|user| user == &other);
						target_space.followers.retain(|user| user == &who);

						Ok(())
					})?;

					Ok(())
				})?;

				Ok(())
			}
			
			pub fn do_comment_on_post(
				who: &T::AccountId,
				post_id: T::PostId,
				bounded_text: BoundedVec<u8, T::CommentStringLimit>
			) -> DispatchResult {

				Spaces::<T>::try_mutate_exists(who.clone(), |sp| -> DispatchResult {
					let space = sp.as_mut().ok_or(Error::<T>::BadMetadata)?;

					ensure!(Posts::<T>::contains_key(post_id.clone()),Error::<T>::PostNotFound);

					let comment_id = NextCommentId::<T>::try_mutate(|id| -> Result<T::CommentId, DispatchError> {
						if *id==Zero::zero() {
							*id=One::one();
						}
	
						let current_id = *id;
						*id = id
							.checked_add(&One::one())
							.ok_or(Error::<T>::Overflow)?;
						
						Ok(current_id)
					})?;

					ensure!(!Comments::<T>::contains_key(post_id.clone(), comment_id.clone()), Error::<T>::NoneValue);
					
					let comment = Comment{
						owner: who.clone(),
						id: comment_id.clone(),
						text: bounded_text,
						post_id: post_id.clone(),
						parent_comment_id: Zero::zero(),
						likes: Zero::zero(),
						dislikes: Zero::zero(),
						date: <frame_system::Pallet<T>>::block_number()
					};
	
					Comments::<T>::insert(post_id.clone(),comment_id,comment);
					space.comments.try_push(post_id.clone()).unwrap();
					
					Self::deposit_event(Event::PostCommented(who.clone(),post_id));
					Ok(())
				})?;

				Ok(())
			}


			pub fn do_comment_on_comment(
				who: &T::AccountId,
				post_id: T::PostId,
				parent_comment_id : T::CommentId,
				bounded_text: BoundedVec<u8, T::CommentStringLimit>,
			) -> DispatchResult {


				Spaces::<T>::try_mutate_exists(who.clone(), |sp| -> DispatchResult {
					let space = sp.as_mut().ok_or(Error::<T>::BadMetadata)?;
				
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

					let comment = Comment{
						owner : who.clone(),
						id: comment_id.clone(),
						text : bounded_text,
						post_id : post_id.clone(),
						parent_comment_id : parent_comment_id.clone(),
						likes: Zero::zero(),
						dislikes: Zero::zero(),
						date : <frame_system::Pallet<T>>::block_number()
					};
	
					Comments::<T>::insert(post_id,comment_id.clone(),comment);
					space.comments.try_push(post_id.clone()).unwrap();
					
					Self::deposit_event(Event::CommmentReplied(who.clone(),parent_comment_id,comment_id));
					Ok(())
				})?;

				Ok(())   
			}
    
		}


}