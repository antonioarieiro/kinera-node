//** About **//
	// Information regarding the pallet
    // note_1: The reason the festival duration storages are seperate is to facilitate block iteration during hooks.
    //TODO organize errors
    //TODO implemenmt tag functionalities
    //TODO finish block assignments (tag TODO #Block)

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
                        AllowDeath,
                        KeepAlive,
                    },
                },
                PalletId
            };
            use frame_system::pallet_prelude::*;
            use codec::{Decode, Encode, MaxEncodedLen};
            use sp_runtime::{RuntimeDebug, traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedSub, One}};
            use scale_info::prelude::vec::Vec;
            use core::convert::TryInto;
            use frame_support::BoundedVec;
            use scale_info::TypeInfo;
            use sp_std::{collections::btree_map::BTreeMap,vec};
            use pallet_movie;


        //* Config *//
        
            #[pallet::pallet]
            #[pallet::generate_store(pub(super) trait Store)]
            pub struct Pallet<T>(_);

            #[pallet::config]
            pub trait Config: frame_system::Config + pallet_movie::Config {
                type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
                type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
            
                type FestivalId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
                type CategoryId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
                
                type NameStringLimit: Get<u32>;
                type DescStringLimit: Get<u32>;
                type CategoryStringLimit: Get<u32>;
                type TagStringLimit: Get<u32>;
                type MaxMoviesInFest: Get<u32>;
                type MaxOwnedFestivals: Get<u32>;
                type MinFesBlockDuration: Get<u32>;
                type MaxFestivalsPerBlock: Get<u32>;
                type MaxTags: Get<u32>;
                type MaxVotes: Get<u32>;
                
                type FestBlockSafetyMargin: Get<u32>;

                type PalletId: Get<PalletId>;
            }
      
      
      

    //** Types **//	
    
        //* Types *//
            
            type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
        
        //* Constants *//
        //* Enums *//
            
            #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
            pub enum FestivalStatus {
                New,
                Approved,
                Declined,
                Active,
                Inactive,
            }
        
        //* Structs *//

            #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            pub struct Festival<FestivalId, BoundedNameString, BoundedDescString, FilmsInFest, FestivalStatus, BalanceOf, VoteList> {
                pub id: FestivalId,
                pub name: BoundedNameString,
                pub description: BoundedDescString,
                pub films: FilmsInFest,
                pub status: FestivalStatus,
                pub min_entry: BalanceOf,
                pub total_lockup: BalanceOf,
                pub vote_list: VoteList,
            }

            #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            pub struct BlockAssignment<BoundedFestivals> {
                pub to_start: BoundedFestivals,
                pub to_end: BoundedFestivals,
            }

            #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            pub struct Category<BoundedTagList> {
                pub tag_list: BoundedTagList,
            }

            #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            #[scale_info(skip_type_params(T))]
            pub struct Tag<BoundedCategoryString, BoundedTagString> {
                pub parent_category: BoundedCategoryString,
                pub description: BoundedTagString,
            }

            #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            pub struct Vote<AccountId, MovieId, Balance> {
                pub voter: AccountId,
                pub vote_for: MovieId,
                pub amount: Balance,
            }

            // #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
            // pub struct CuratingVote<BoundedString,BalanceOf> {
            //     pub vote: BoundedString,
            //     pub bet: BalanceOf,
            // }




    //** Storage **//

        //* Festivals *//   

            #[pallet::storage]
            #[pallet::getter(fn next_festival_id)]
            pub(super) type NextFestivalId<T: Config> = 
                StorageValue<
                    _, 
                    T::FestivalId, 
                    ValueQuery
                >;
    
            #[pallet::storage]
            #[pallet::getter(fn get_festival)]
            pub type Festivals<T: Config> = 
                StorageMap<
                    _, 
                    Blake2_128Concat, T::FestivalId, 
                    Festival<
                        T::FestivalId, 
                        BoundedVec<u8, T::NameStringLimit>, 
                        BoundedVec<u8, T::DescStringLimit>, 
                        BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, 
                        FestivalStatus,
                        BalanceOf<T>,
                        BoundedVec<Vote<T::AccountId, BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>>, T::MaxMoviesInFest>,
                    >,
                    OptionQuery
                >;

            #[pallet::storage]
            #[pallet::getter(fn get_owned_festivals)]
            pub(super) type FestivalOwners<T: Config> = 
                StorageMap<
                    _,
                    Blake2_128Concat, T::AccountId,
                    BoundedVec<T::FestivalId, T::MaxOwnedFestivals>,
                >;

        //* Block Assignments *// 

            // Stores either the start/end of festivals. 
            // To be iterated during hooks.
            #[pallet::storage]
            #[pallet::getter(fn get_block_assignments)]
            pub(super) type BlockAssignments<T: Config> = 
                StorageMap<
                    _,
                    Blake2_128Concat, T::BlockNumber,
                    BlockAssignment<BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>>,
                >;

        //* Category *// 
            
            #[pallet::storage]
            #[pallet::getter(fn get_category)]
            pub type Categories <T: Config> = 
                StorageMap<
                    _, 
                    Blake2_128Concat, BoundedVec<u8, T::CategoryStringLimit>, 
                    Category<
                        BoundedVec<BoundedVec<u8, T::TagStringLimit>, T::MaxTags>, 
                    >,
                    OptionQuery
                >;
            
            #[pallet::storage]
            #[pallet::getter(fn get_tag)]
            pub type Tags <T: Config> = 
                StorageMap<
                    _, 
                    Blake2_128Concat, BoundedVec<u8, T::TagStringLimit>, 
                    Tag<
                        BoundedVec<u8, T::CategoryStringLimit>, 
                        BoundedVec<u8, T::DescStringLimit>, 
                    >,
                    OptionQuery
                >;




    //** Genesis **//
        
        #[pallet::genesis_config]
        pub struct GenesisConfig<T: Config> {
            pub category_to_tag_map: Vec<(
                BoundedVec<u8, T::CategoryStringLimit>,
                BoundedVec<BoundedVec<u8, T::TagStringLimit>, T::MaxTags>
            )>
        }


        #[cfg(feature = "std")]
        impl<T: Config> Default for GenesisConfig<T> {
            fn default() -> Self {
                Self { 
                    category_to_tag_map: Default::default() 
                }
            }
        }


        #[pallet::genesis_build]
        impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
            fn build(&self) {
                for (category_id, bounded_tag_id_list) in &self.category_to_tag_map {
                    
                    // initialize the category
                    let category = Category {
                        tag_list: bounded_tag_id_list.clone(),
                    };

                    <Categories<T>>::insert(category_id.clone(), category);


                    // create an entry for each of the category's tags and bind them together
                    let tag_description: BoundedVec<u8, T::DescStringLimit>
                        = TryInto::try_into(Vec::new()).unwrap();

                    for tag_id in bounded_tag_id_list {
                        let tag = Tag {
                            parent_category: category_id.clone(),
                            description: tag_description.clone(),
                        };

                        <Tags<T>>::insert(tag_id, tag);
                    }
                }
            }
        }



    //** Events **//

        #[pallet::event]
        #[pallet::generate_deposit(pub(super) fn deposit_event)]
        pub enum Event<T: Config> {
            FestivalCreated(T::AccountId, T::FestivalId),
            MovieAddedToFestival(T::FestivalId, BoundedVec<u8, T::LinkStringLimit>, T::AccountId),
            VotedForMovieInFestival(T::FestivalId, BoundedVec<u8, T::LinkStringLimit>, T::AccountId),
            CategoryCreated(T::AccountId, BoundedVec<u8, T::CategoryStringLimit>),
            TagCreated(T::AccountId, BoundedVec<u8, T::TagStringLimit>, BoundedVec<u8, T::CategoryStringLimit>),
        }



    //** Errors **//
        
        #[pallet::error]
        pub enum Error<T> {
            Overflow,
            Underflow,
            BadMetadata,
            InsufficientBalance,
            
            PastStartDate,
            FestivalPeriodTooShort,

            NonexistentCategory,
            CategoryAlreadyExists,

            TagAlreadyExists,
            NonexistentTag,

            MovieAlreadyInFestival,
            MovieNotInFestival,
            InvalidFestival,

            NonexistentFestival,
            FestivalNotActive,

            VoteAmountTooLow,

            InvalidBlockPeriod,
        }





    //** Hooks **//

        #[pallet::hooks]
        impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
            
            fn on_initialize(_now: T::BlockNumber) -> Weight {
                0
            }

            fn on_finalize(now: T::BlockNumber){
                Self::hook_deactivate_festival(now);
                Self::hook_activate_festival(now);
            }
        }



    //** Extrinsics **//

        #[pallet::call]
        impl<T: Config> Pallet<T> {

            #[pallet::weight(10_000)]
            pub fn create_festival(
                origin: OriginFor<T>,
                bounded_name: BoundedVec<u8, T::NameStringLimit>,
                _bounded_description: BoundedVec<u8, T::DescStringLimit>,
                //TODO categories:Vec<T::CategoryId>,
                min_entry: BalanceOf<T>,
                start_block: T::BlockNumber,
                end_block: T::BlockNumber,
            ) -> DispatchResult {
                
                let who = ensure_signed(origin)?;
                Self::do_validate_festival(start_block, end_block)?;
                
                let festival_id = Self::do_create_festival(bounded_name, _bounded_description, min_entry)?;
                Self::do_bind_owners_to_festival(who.clone(), festival_id)?;
                Self::do_bind_duration_to_festival(festival_id, start_block, end_block)?;

                Self::deposit_event(Event::FestivalCreated(who.clone(), festival_id));
                Ok(())
            }

            
            #[pallet::weight(10_000)]
            pub fn create_category(
                origin: OriginFor<T>,
                bounded_name: BoundedVec<u8, T::CategoryStringLimit>,
            ) -> DispatchResult {
                
                let who = ensure_signed(origin)?;
                Self::do_validate_category(bounded_name.clone())?;

                Self::do_create_category(bounded_name.clone())?;

                Self::deposit_event(Event::CategoryCreated(who.clone(), bounded_name));
                Ok(())
            }

            
            #[pallet::weight(10_000)]
            pub fn create_tag(
                origin: OriginFor<T>,
                bounded_tag: BoundedVec<u8, T::TagStringLimit>,
                bounded_category: BoundedVec<u8, T::CategoryStringLimit>,
                bounded_description: BoundedVec<u8, T::DescStringLimit>,
            ) -> DispatchResult {
                
                let who = ensure_signed(origin)?;
                Self::do_validate_tag(bounded_tag.clone(), bounded_category.clone())?; //TODO validate description

                Self::do_create_tag(bounded_tag.clone(), bounded_category.clone(), bounded_description.clone())?;

                Self::deposit_event(Event::TagCreated(who.clone(), bounded_tag, bounded_category));
                Ok(())
            }
        
        
            #[pallet::weight(10_000)]
            pub fn add_internal_movie_to_festival(
                origin: OriginFor<T>,
                festival_id: T::FestivalId,
                movie_id: BoundedVec<u8, T::LinkStringLimit>,
            )-> DispatchResult{
            
                let who = ensure_signed(origin)?;

                Self::do_add_internal_movie_to_festival(festival_id, movie_id.clone())?;

                Self::deposit_event(Event::MovieAddedToFestival(festival_id, movie_id, who.clone()));
                Ok(())
            }
        
            #[pallet::weight(10_000)]
            pub fn add_external_movie_to_festival(
                origin: OriginFor<T>,
                festival_id: T::FestivalId,
                source: pallet_movie::ExternalSource,
                movie_link: BoundedVec<u8, T::LinkStringLimit>,
            )-> DispatchResult{
            
                let who = ensure_signed(origin)?;

                let movie_exists = pallet_movie::Pallet::<T>::do_does_external_movie_exist(movie_link.clone())?;
                if !movie_exists {
                    pallet_movie::Pallet::<T>::do_create_external_movie(
                        &who.clone(), 
                        source.clone(),
                        movie_link.clone()
                    )?;
                }

                Self::do_add_external_movie_to_festival(festival_id, movie_link.clone())?;

                Self::deposit_event(Event::MovieAddedToFestival(festival_id, movie_link, who.clone()));
                Ok(())
            }


            #[pallet::weight(10_000)]
            pub fn vote_for_movie_in_festival(
                origin: OriginFor<T>,
                festival_id: T::FestivalId,
                movie_id : BoundedVec<u8, T::LinkStringLimit>,
                amount : BalanceOf<T>,
            )-> DispatchResult{
                
                let who = ensure_signed(origin)?;

                Self::do_vote_for_movie_in_festival(&who,festival_id, movie_id.clone(), amount)?;

                Self::deposit_event(Event::VotedForMovieInFestival(festival_id, movie_id, who.clone()));
                Ok(())
            }

        }


        
    //** Helpers **//

        impl<T: Config> Pallet<T> {

            //* Festival *//
            
                pub fn do_validate_festival(
                    start_block : T::BlockNumber,
                    end_block: T::BlockNumber
                ) -> Result<(), DispatchError> {

                    //TODO add more validations
                    //TODO verificar datas - alex
                    let safe_start_time = start_block
                        .checked_sub(&T::BlockNumber::from(T::FestBlockSafetyMargin::get()))
                        .ok_or(Error::<T>::InvalidBlockPeriod)?;
                
                    ensure!(frame_system::Pallet::<T>::block_number() < safe_start_time, Error::<T>::PastStartDate);
                    ensure!(end_block-safe_start_time >= T::BlockNumber::from(T::FestBlockSafetyMargin::get()), Error::<T>::FestivalPeriodTooShort);
            
                    Ok(())
                }

                pub fn do_create_festival(
                    name: BoundedVec<u8, T::NameStringLimit>,
                    description: BoundedVec<u8, T::DescStringLimit>,
                    min_ticket_price: BalanceOf<T>,
                ) -> Result<T::FestivalId, DispatchError> {

                    let festival_id =
                        NextFestivalId::<T>::try_mutate(|id| -> Result<T::FestivalId, DispatchError> {
                            let current_id = *id;
                            *id = id
                                .checked_add(&One::one())
                                .ok_or(Error::<T>::Overflow)?;
                            Ok(current_id)
                        })
                    ?;
            
                    let bounded_film_list: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    let bounded_vote_list: BoundedVec<Vote<T::AccountId, BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>>, T::MaxMoviesInFest>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    let zero_lockup = BalanceOf::<T>::from(0u32);
                    
                    let festival = Festival {
                        id: festival_id.clone(),
                        name: name,
                        description: description,
                        films: bounded_film_list,
                        status: FestivalStatus::New,
                        min_entry: min_ticket_price,
                        total_lockup: zero_lockup,
                        vote_list: bounded_vote_list,
                    };

                    Festivals::<T>::insert(festival_id, festival);
                    
                    Ok(festival_id)
                }


                pub fn do_bind_owners_to_festival(
                    who : T::AccountId,
                    festival_id : T::FestivalId,
                ) -> Result<(), DispatchError> {

                    //TODO check contains
                    //TODO add new entry

                    let mut bounded_festival_list: BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                    bounded_festival_list.try_push(festival_id).unwrap();
                        
                    FestivalOwners::<T>::insert(who, bounded_festival_list);

                    Ok(())
                }


                pub fn do_bind_duration_to_festival(
                    festival_id : T::FestivalId,
                    start_block : T::BlockNumber,
                    end_block: T::BlockNumber
                ) -> Result<(), DispatchError> {
                    
                    //TODO #Block
                    //TODO check contains
                    //TODO add new entry


                    // handle the festival's starting block
                    if BlockAssignments::<T>::contains_key(start_block) == true {
					
                        let mut start_assignments = BlockAssignments::<T>::try_get(start_block).unwrap(); //TODO optimize
                        start_assignments.to_start.try_push(festival_id).unwrap();
                    }
                    else {
                        let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
                            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                        bounded_start_list.try_push(festival_id).unwrap();
                        
                        let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
                            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                        
                        let assignment = BlockAssignment {
                            to_start: bounded_start_list.clone(),
                            to_end: bounded_end_list.clone(),
                        };
                        BlockAssignments::<T>::insert(start_block.clone(), assignment);
                    }

                    
                    // handle the festival's ending block
                    if BlockAssignments::<T>::contains_key(end_block) == true {
					
                        let mut end_assignments = BlockAssignments::<T>::try_get(end_block).unwrap(); //TODO optimize
                        end_assignments.to_end.try_push(festival_id).unwrap();
                    }
                    else {
                        let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
                            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                        
                        let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
                            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                        bounded_end_list.try_push(festival_id).unwrap();
                        
                        let assignment = BlockAssignment {
                            to_start: bounded_start_list.clone(),
                            to_end: bounded_end_list.clone(),
                        };
                        BlockAssignments::<T>::insert(end_block.clone(), assignment);
                    }

                    // BlockAssignment<BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>>,


                    // let ranking_list = RankingList {
                    //     name: bounded_name,
                    //     description: bounded_description,
                    //     status:RankingListStatus::Ongoing,
                    //     category: category,
                    //     list_deadline: list_deadline_block,
                    //     list_duration: list_duration,
                    //     movies_in_list: movies_in_list,
                    //     votes_by_user: votes_by_user,
                    // };
                    // RankingLists::<T>::insert(ranking_list_id.clone(), ranking_list);

                    


                    Ok(())
                }



                pub fn do_create_empty_block_assignments(
                    festival_id : T::FestivalId,
                ) -> Result<(), DispatchError> {

                    let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                    bounded_start_list.try_push(festival_id).unwrap();
                    
                    let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
                    bounded_end_list.try_push(festival_id).unwrap();
                    
                    
                    let assignment = BlockAssignment {
                        to_start: bounded_start_list.clone(),
                        to_end: bounded_end_list.clone(),
                    };
                    
                    Ok(())
                }




                fn hook_activate_festival(
                    now : T::BlockNumber,
                ) -> DispatchResult {
                    
                    let fests = BlockAssignments::<T>::try_get(now);
                    ensure!(fests.is_ok(), Error::<T>::NonexistentFestival);
                    let festivals = fests.unwrap();
                    for festival_id in festivals.to_start.iter() {
                        Festivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult{
                            let fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;

                            ensure!(fest.status == FestivalStatus::New , Error::<T>::FestivalNotActive);
                            fest.status = FestivalStatus::Active;
                            Ok(())
                        })?;
                    }
                    
                    Ok(())
                }


            //* Hooks *//

                fn hook_deactivate_festival(
                    now : T::BlockNumber,
                ) -> DispatchResult {
                    
                    let fests = BlockAssignments::<T>::try_get(now);
                    ensure!(fests.is_ok(), Error::<T>::NonexistentFestival);
                    let festivals = fests.unwrap();
                    for festival_id in festivals.to_end.iter() {
                        Festivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult{
                            let fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
                            
                            ensure!(fest.status == FestivalStatus::Active , Error::<T>::FestivalNotActive);
                        
                            if fest.vote_list.len() > 0 {
                                Self::do_resolve_market(festival_id.clone())?;
                            }
                            
                            fest.status = FestivalStatus::Inactive;
                            Ok(())
                        })?;
                    }
                    
                    Ok(())
                }



            //* Category *//

                pub fn do_validate_category (
                    bounded_name:  BoundedVec<u8, T::CategoryStringLimit>,
                )-> Result<(), DispatchError> {
    
                    ensure!(!Categories::<T>::contains_key(bounded_name), Error::<T>::CategoryAlreadyExists);
                    Ok(())
                }


                pub fn do_create_category (
                    bounded_name:  BoundedVec<u8, T::CategoryStringLimit>,
                )-> Result<(), DispatchError> {
                        
                    let bounded_tag_list: BoundedVec<BoundedVec<u8, T::TagStringLimit>, T::MaxTags>
                        = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;

                    let category = Category {
                        tag_list: bounded_tag_list,
                    };
    
                    Categories::<T>::insert(bounded_name, category);
                    Ok(())
                }



            //* Tag *//

                pub fn do_validate_tag (
                    bounded_tag:  BoundedVec<u8, T::TagStringLimit>,
                    bounded_category:  BoundedVec<u8, T::CategoryStringLimit>,
                )-> Result<(), DispatchError> {
    
                    ensure!(Categories::<T>::contains_key(bounded_category), Error::<T>::NonexistentCategory);
                    ensure!(!Tags::<T>::contains_key(bounded_tag), Error::<T>::TagAlreadyExists);
                    Ok(())
                }


                pub fn do_create_tag (
                    bounded_tag:  BoundedVec<u8, T::TagStringLimit>,
                    bounded_category:  BoundedVec<u8, T::CategoryStringLimit>,
                    bounded_description: BoundedVec<u8, T::DescStringLimit>,
                )-> Result<(), DispatchError> {
                        
                    let tag = Tag {
                        parent_category: bounded_category.clone(),
                        description: bounded_description,
                    };

                    Categories::<T>::try_mutate_exists(bounded_category, |category| -> DispatchResult {
                        let cat = category.as_mut().ok_or(Error::<T>::BadMetadata)?;
                        ensure!(!cat.tag_list.contains(&bounded_tag), Error::<T>::TagAlreadyExists);
                        
                        cat.tag_list.try_push(bounded_tag.clone()).unwrap();
                        Ok(())
                    })?;

                    Tags::<T>::insert(bounded_tag, tag);

                    Ok(())
                }


            //** Movie **//

                pub fn do_add_internal_movie_to_festival (
                    festival_id: T::FestivalId,
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                )-> Result<(), DispatchError> {
                    
                    pallet_movie::Pallet::<T>::do_ensure_internal_movie_exist(movie_id.clone())?;
                    
                    Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
                        let fes = festival.as_mut().ok_or(Error::<T>::BadMetadata)?;
                        ensure!(!fes.films.contains(&movie_id), Error::<T>::MovieAlreadyInFestival);
                        
                        fes.films.try_push(movie_id).unwrap();
                        Ok(())
                    })?;
                    
                    Ok(())
                }

                pub fn do_add_external_movie_to_festival (
                    festival_id: T::FestivalId,
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                )-> Result<(), DispatchError> {
                    
                    Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
                        let fes = festival.as_mut().ok_or(Error::<T>::BadMetadata)?;
                        ensure!(!fes.films.contains(&movie_id), Error::<T>::MovieAlreadyInFestival);
                        
                        fes.films.try_push(movie_id).unwrap();
                        Ok(())
                    })?;
                    
                    Ok(())
                }


                pub fn do_vote_for_movie_in_festival(
                    who: &T::AccountId,
                    festival_id: T::FestivalId,
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                    amount: BalanceOf<T>,
                )-> Result<(), DispatchError> {
                    
                    let vote = Vote {
                        voter: who.clone(),
                        vote_for: movie_id.clone(),
                        amount: amount,
                    };

                    Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
                        
                        let fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;   

                        ensure!(fest.status == FestivalStatus::Active, Error::<T>::FestivalNotActive);
                        ensure!(fest.min_entry <= amount, Error::<T>::VoteAmountTooLow);
                        ensure!(fest.films.contains(&movie_id.clone()), Error::<T>::MovieNotInFestival);
                        ensure!( <T as Config>::Currency::transfer(
                            who,
                            &Self::account_id(),
                            amount,
                            AllowDeath,
                        ) == Ok(()), Error::<T>::InsufficientBalance);
                        
                        fest.total_lockup.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
                        fest.vote_list.try_push(vote).unwrap();

                        Ok(())
                    })
                        
                }
            


            /* Treasury */

                fn account_id() -> T::AccountId {
                    <T as Config>::PalletId::get().try_into_account().unwrap()
                }

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


            /* Votes */

            fn do_resolve_market(
                festival_id : T::FestivalId
            ) -> DispatchResult {
                
                let winning_opts = Self::do_get_winning_options(festival_id).unwrap();
                let winners_lockup = Self::do_get_winners_total_lockup(festival_id, winning_opts.clone()).unwrap();
                
                let festival = Festivals::<T>::try_get(festival_id).unwrap();
                let total_lockup = festival.total_lockup;
                for vote in festival.vote_list { 
                    if winning_opts.contains(&vote.vote_for.clone()) {
                        Self::do_transfer_funds_from_treasury(
                            &vote.voter, 
                            Self::do_calculate_simple_reward(total_lockup, vote.amount, winners_lockup)?,
                        )?;
                    }
                }

                //TODO add event
                Ok(())
            }


            fn do_get_winning_options(
                festival_id : T::FestivalId
            ) -> Result<Vec<BoundedVec<u8, T::LinkStringLimit>>,DispatchError>{
            
                let mut accumulator = BTreeMap::new();

                let fes_votes = Festivals::<T>::try_get(festival_id).unwrap().vote_list;
                for vote in fes_votes {
                    // amount -amount = 0 with Balance trait
                    let movie_id = vote.vote_for;
                    let amount = vote.amount;
                    let stat =  accumulator.entry(movie_id).or_insert(amount - amount);
                    *stat += amount;
                }

                let first_winner= 
                        accumulator
                        .iter()
                        .clone()
                        .max_by_key(|p| p.1)
                        .unwrap();
                
                let mut winners = vec![first_winner.0.clone()];
                
                for (movie, lockup) in &accumulator {
                    if lockup == first_winner.1 && movie != first_winner.0 {
                        winners.push(movie.clone());
                    }
                }
                
                Ok(winners)
            }


            fn do_get_winners_total_lockup(
                festival_id: T::FestivalId, 
                winning_movies:Vec<BoundedVec<u8, T::LinkStringLimit>>
            ) -> Result<BalanceOf<T>,DispatchError> {
                
                let mut winners_lockup : <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance=0u32.into();

                let fes_votes = Festivals::<T>::try_get(festival_id).unwrap().vote_list;

                for vote in fes_votes {
                    if winning_movies.contains(&vote.vote_for.clone()) {
                        winners_lockup += vote.amount;
                    }   
                }
            
                Ok(winners_lockup)
            }

            fn do_calculate_simple_reward(
                total_lockup: BalanceOf<T>,
                user_lockup: BalanceOf<T>,
                winner_lockup: BalanceOf<T>) ->Result<BalanceOf<T>,DispatchError> {
                
                Ok(( user_lockup * 1000u32.into() )/( winner_lockup)*(total_lockup/1000u32.into()))
            }





        }
}
    