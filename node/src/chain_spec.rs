use hex_literal::hex;
use node_primitives::*;
use node_template_runtime::{
	constants::currency::*, opaque::SessionKeys, BabeConfig, BalancesConfig, CouncilConfig,
	DemocracyConfig, ElectionsConfig, TagsModuleConfig, RankingListModuleConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig, MaxNominations,
	SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
	NominationPoolsConfig, 
	BABE_GENESIS_EPOCH_CONFIG, wasm_binary_unwrap,
	CategoryStringLimit, TagStringLimit, RankingStringLimit, MaxTags, 
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
	BoundedVec,
};
// use pallet_festival::{CategoryStringLimit, TagStringLimit, MaxTags};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

fn session_keys(babe: BabeId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online }
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

pub fn staging_network_config() -> ChainSpec {
	let boot_nodes = vec![];

	ChainSpec::from_genesis(
		"Substrate Stencil",
		"stencil_network",
		ChainType::Live,
		staging_network_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		None,
		Default::default(),
	)
}

fn staging_network_config_genesis() -> GenesisConfig {
	// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	let initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)> = vec![
		(
			// 5Grpw9i5vNyF6pbbvw7vA8pC5Vo8GMUbG8zraLMmAn32kTNH
			hex!["d41e0bf1d76de368bdb91896b0d02d758950969ea795b1e7154343ee210de649"].into(),
			// 5DLMZF33f61KvPDbJU5c2dPNQZ3jJyptsacpvsDhwNS1wUuU
			hex!["382bd29103cf3af5f7c032bbedccfb3144fe672ca2c606147974bc2984ca2b14"].into(),
			// 5Dhd2QbrSE4dyNn3YUg8j5TY3fG7ZAWZMoRRF9KUc7VPVGmC
			hex!["48640c12bc1b351cf4b051ac1cf7b5740765d02e34989d0a9dd935ce054ebb21"]
				.unchecked_into(),
			// 5C6rkxAZB437B5Bf1yS4B4qjW4HZPeBp8Kzx2Se9FLKhfyHY
			hex!["01a474a93a0cf830fb40b1d17fd1fc7c6b4a95fa11f90345558574a72da0d4b1"]
				.unchecked_into(),
			// 5DscuovXyY1o7DxYroYjYgipn87eqYLyQA3HJ21Utb7TqAai
			hex!["50041e469c63c994374a2829b0b0829213abd53be5113e751043318a9d7c0757"]
				.unchecked_into(),
		),
		(
			// 5CFDk3yCSgQ2goiaksMfRMFRS7ZU28BZqPQDeAsgZUa6FRzt
			hex!["08050f1b6bcd4651004df427c884073652bafd54e5ca25cea69169532db2910b"].into(),
			// 5F1ks2enazaPktQa3HURLK8GywzNZaGirovPtFvvbv91TLhJ
			hex!["8275157f2a1d8373106cb00078a73a92a3303f3bf6eb72c3a67413bd943b020b"].into(),
			// 5CQ7gVQj96m8y79qPCqrM291rSNREfZ1Tf2fiLPSJReWTNy2
			hex!["0ecddcf7643a98de200b80fe7b18ebd38987fa106c5ed84fc004fa75ea4bac67"]
				.unchecked_into(),
			// 5FyNaMc6GaioN7K9QzPJDEtGThJ1HmcruRdgtiRxaoAwn2VD
			hex!["acdfcce0e40406fac1a8198c623ec42ea13fc627e0274bbb6c21e0811482ce13"]
				.unchecked_into(),
			// 5EUhcM9WPJGvhCz1UptA7ye8TgktGqbhaeSohCkAfW76q5bS
			hex!["6ac58683d639d3992a0090ab15f8c1dcf5a5ab7652fc9de60845441f9fc93903"]
				.unchecked_into(),
		),
		(
			// 5F6YideXfGcskpdFUczu3nZcJFmU9WKHgjjNVQjqgeVGRs66
			hex!["861c6d95051f942bb022f13fc2125b2974933d8ab1441bfdee9855e9d8051556"].into(),
			// 5F92x4qKNYaHtfp5Yy7kb9r6gHCHkN3YSvNuedERPHgrURTn
			hex!["8801f479e09a78515f1badee0169864dae45648109091e29b03a7b4ea97ec018"].into(),
			// 5CLqVJSpfAdMYW1FHygEV8iEi8XFornEcCzrhw9WmFbbp8Qp
			hex!["0c4d9de1e313572750abe19140db56433d20e4668e09de4df81a36566a8f2528"]
				.unchecked_into(),
			// 5HEQh8yEv4QU7joBCKYdjJJ57qU1gDAm4Xv5QZKfFnSbXpeo
			hex!["e493d74f9fa7568cca9dd294c9619a54c2e1b6bd3ecf3677fa7f9076b98c3fcd"]
				.unchecked_into(),
			// 5GUEUCusMfW9c229gyuDG6XUH9pi3Cs4EZR9STtw8opfKuS6
			hex!["c2e2a133b23995a48ff46cc704ef61929ee4a29b5fa468e41019ac63f3694e1f"]
				.unchecked_into(),
		),
		(
			// 5FxxpyvEnE2sVujvhr6x4A4G171uv4WKSLvrUNst9M8MfdpV
			hex!["ac8fdba5bbe008f65d0e85181daa5443c2eb492fea729a5981b2161467f8655c"].into(),
			// 5FxFAYsTNf31D5AGbXW9ETZPUZofpreHjJkdKehidcvDt5X4
			hex!["ac039bef73f76755d3747d711554f7fb0f16022da51483e0d600c9c7c8cbf821"].into(),
			// 5GdjiBeMEFqTE6mWod3UqPrtkQTscRGtAcmdSbR26vGiXpwB
			hex!["ca2245b6fa117fab9353a2031104d1d5d62e311957f375762324e65d71127465"]
				.unchecked_into(),
			// 5DMfkaaR4tzmarUsRMkrbnFNmVnYtYjTPFJsjvA4X15WAZZB
			hex!["392c51bf0c08f89cb1e091782d81359475d780986968ba7f6fa60f41feda6bf7"]
				.unchecked_into(),
			// 5HGzdyJakxDdnERv3nvNjd6Xmz5R39NEuuJ2B3miubDY6BHD
			hex!["e68c9a2ee25e1999a4e87906aea429f3e5f3fc8dc9cd89f423d82860c6937b2e"]
				.unchecked_into(),
		),
	];

	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = hex![
		// 5FemZuvaJ7wVy4S49X7Y9mj7FyTR4caQD5mZo2rL7MXQoXMi
		"9eaf896d76b55e04616ff1e1dce7fc5e4a417967c17264728b3fd8fee3b12f3c"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		initial_authorities,
		vec![],
		root_key,
		endowed_accounts
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	//TODO optimize
	//TODO festivals and movies use copies of the ranking list's tags/categories
	
	// Movie


	// Default Ranking Lists
		
		let rl_all_time_best: (
			BoundedVec<u8, RankingStringLimit>,
			BoundedVec<u8, RankingStringLimit>,
			BlockNumber,
			BoundedVec<(
				BoundedVec<u8, CategoryStringLimit>,
				BoundedVec<u8, TagStringLimit>,
			), MaxTags>,
		) = (
			"All Times, Best Films".as_bytes().to_vec().try_into().unwrap(),
			"The best films on the platform.".as_bytes().to_vec().try_into().unwrap(),
			BlockNumber::from(4800u32),
			TryInto::try_into(vec![(
				"Just FUN".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
			)]).unwrap(),
		);
		
		let rl_best_south_america: (
			BoundedVec<u8, RankingStringLimit>,
			BoundedVec<u8, RankingStringLimit>,
			BlockNumber,
			BoundedVec<(
				BoundedVec<u8, CategoryStringLimit>,
				BoundedVec<u8, TagStringLimit>,
			), MaxTags>,
		) = (
			"Best South America Cinema".as_bytes().to_vec().try_into().unwrap(),
			"The best films from South America.".as_bytes().to_vec().try_into().unwrap(),
			BlockNumber::from(4800u32),
			TryInto::try_into(vec![(
				"Just FUN".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
			)]).unwrap(),
		);
		
		let rl_best_asian_fiction: (
			BoundedVec<u8, RankingStringLimit>,
			BoundedVec<u8, RankingStringLimit>,
			BlockNumber,
			BoundedVec<(
				BoundedVec<u8, CategoryStringLimit>,
				BoundedVec<u8, TagStringLimit>,
			), MaxTags>,
		) = (
			"Best Asiatic Fiction".as_bytes().to_vec().try_into().unwrap(),
			"The best Asian Fiction movies.".as_bytes().to_vec().try_into().unwrap(),
			BlockNumber::from(4800u32),
			TryInto::try_into(vec![(
				"Just FUN".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
			)]).unwrap(),
		);
		
		let rl_best_crypto: (
			BoundedVec<u8, RankingStringLimit>,
			BoundedVec<u8, RankingStringLimit>,
			BlockNumber,
			BoundedVec<(
				BoundedVec<u8, CategoryStringLimit>,
				BoundedVec<u8, TagStringLimit>,
			), MaxTags>,
		) = (
			"Best Crypto Youtuber/TikTok Channels, Online".as_bytes().to_vec().try_into().unwrap(),
			"The best content regarding Web3 and Crypto.".as_bytes().to_vec().try_into().unwrap(),
			BlockNumber::from(4800u32),
			TryInto::try_into(vec![(
				"Just FUN".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
			)]).unwrap(),
		);
		
		let rl_documentaries_and_fiction: (
			BoundedVec<u8, RankingStringLimit>,
			BoundedVec<u8, RankingStringLimit>,
			BlockNumber,
			BoundedVec<(
				BoundedVec<u8, CategoryStringLimit>,
				BoundedVec<u8, TagStringLimit>,
			), MaxTags>,
		) = (
			"Documentaries and Fiction, Online".as_bytes().to_vec().try_into().unwrap(),
			"The best films on the platform.".as_bytes().to_vec().try_into().unwrap(),
			BlockNumber::from(4800u32),
			TryInto::try_into(vec![(
				"Just FUN".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
			)]).unwrap(),
		);
		
		// initialize the vec with the pre-built tuples
		let initial_ranking_lists = vec![
			rl_all_time_best, 
			rl_best_south_america,
			rl_best_asian_fiction,
			rl_best_crypto,
			rl_documentaries_and_fiction,
		];
			
		


	// Tags Ranking Lists

		let rl_cinema_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Fiction".as_bytes().to_vec().try_into().unwrap(),
				"Science Fiction".as_bytes().to_vec().try_into().unwrap(),
				"Drama".as_bytes().to_vec().try_into().unwrap(),
				"Documentary".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_stars_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Influencers".as_bytes().to_vec().try_into().unwrap(),
				"Educator".as_bytes().to_vec().try_into().unwrap(),
				"Pivots".as_bytes().to_vec().try_into().unwrap(),
				"Web3".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_just_fun_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Art".as_bytes().to_vec().try_into().unwrap(),
				"Dance".as_bytes().to_vec().try_into().unwrap(),
				"Disasters".as_bytes().to_vec().try_into().unwrap(),
				"Kids".as_bytes().to_vec().try_into().unwrap(),
				"Sports".as_bytes().to_vec().try_into().unwrap(),
				"Products".as_bytes().to_vec().try_into().unwrap(),
				"Professionals".as_bytes().to_vec().try_into().unwrap(),
				"News".as_bytes().to_vec().try_into().unwrap(),
				"Services".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();
			
		let rl_we_festivals_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Information".as_bytes().to_vec().try_into().unwrap(),
				"Offbeat Films/Marginal".as_bytes().to_vec().try_into().unwrap(),
				"Experimental Cinema".as_bytes().to_vec().try_into().unwrap(),
				"Video Art".as_bytes().to_vec().try_into().unwrap(),
				"Video Clips".as_bytes().to_vec().try_into().unwrap(),
				"We Festivals Others".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();
			
		let rl_artificial_intelligence_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"AI Drama".as_bytes().to_vec().try_into().unwrap(),
				"AI Documentary".as_bytes().to_vec().try_into().unwrap(),
				"AI Fiction".as_bytes().to_vec().try_into().unwrap(),
				"AI Fake".as_bytes().to_vec().try_into().unwrap(),
				"AI Science Fiction".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_gaming_streamers_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Action/Adventure".as_bytes().to_vec().try_into().unwrap(),
				"Role-Playing".as_bytes().to_vec().try_into().unwrap(),
				"Strategy".as_bytes().to_vec().try_into().unwrap(),
				"Sports".as_bytes().to_vec().try_into().unwrap(),
				"Puzzle".as_bytes().to_vec().try_into().unwrap(),
				"Simulation".as_bytes().to_vec().try_into().unwrap(),
				"Racing".as_bytes().to_vec().try_into().unwrap(),
				"Fighting".as_bytes().to_vec().try_into().unwrap(),
				"Platformer".as_bytes().to_vec().try_into().unwrap(),
				"MMO (Massive Multiplayer Online)".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();



	// Tags Moderation

		let mod_violence_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_discrimination_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_lack_of_consent_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_impersonation_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_terrorism_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_copyright_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_misinformation_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_pornography_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_extreme_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_naming_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_categorization_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();


	// setup the final map with all categories and tags by type
	let initial_categories_and_tags: Vec <(
		(BoundedVec<u8, CategoryStringLimit>, BoundedVec<u8, CategoryStringLimit>),
		BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
	)> = vec![
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),

		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),

		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags),

		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Violence".as_bytes().to_vec().try_into().unwrap()), mod_violence_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Discrimination".as_bytes().to_vec().try_into().unwrap()), mod_discrimination_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "LackOfConsent".as_bytes().to_vec().try_into().unwrap()), mod_lack_of_consent_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Impersonation".as_bytes().to_vec().try_into().unwrap()), mod_impersonation_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Terrorism".as_bytes().to_vec().try_into().unwrap()), mod_terrorism_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Copyright".as_bytes().to_vec().try_into().unwrap()), mod_copyright_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Misinformation".as_bytes().to_vec().try_into().unwrap()), mod_misinformation_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Pornography".as_bytes().to_vec().try_into().unwrap()), mod_pornography_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Extreme".as_bytes().to_vec().try_into().unwrap()), mod_extreme_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Naming".as_bytes().to_vec().try_into().unwrap()), mod_naming_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Categorization".as_bytes().to_vec().try_into().unwrap()), mod_categorization_tags),
	];

	
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary_unwrap().to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone()))
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			// TODO: ForceEra::ForceNone
			..Default::default()
		},
		babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: GrandpaConfig { authorities: vec![] },
		im_online: ImOnlineConfig { keys: vec![] },
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		treasury: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * DOLLARS,
			min_join_bond: 1 * DOLLARS,
			..Default::default()
		},

		tags_module: TagsModuleConfig {
			category_to_tag_map: initial_categories_and_tags.iter().cloned().map(|x| x).collect(),
		},

		ranking_list_module: RankingListModuleConfig {
			default_ranking_lists: initial_ranking_lists.iter().cloned().map(|x| x).collect(),
		},

	}
}
