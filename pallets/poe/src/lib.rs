#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // The struct on which we build all of our Pallet logic.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /* Placeholder for defining custom types. */

    // TODO: Update the `config` block below
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxBytesInHash: Get<u32>;
    }

    // TODO: Update the `event` block below
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		ClaimTransferred(T::AccountId, T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
	}

    // TODO: Update the `error` block below
    #[pallet::error]
    pub enum Error<T> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
		TransferToSelf,
	}

    // TODO: add #[pallet::storage] block
	#[pallet::storage]
	/// Maps each proof to its owner and block number when the proof was made
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxBytesInHash>,
		(T::AccountId, T::BlockNumber),
		OptionQuery,
	>;

	// Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,
            proof: BoundedVec<u8, T::MaxBytesInHash>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			Self::add_claim(&sender, &proof)?;
            Self::deposit_event(Event::ClaimCreated(sender, proof));		
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            proof: BoundedVec<u8, T::MaxBytesInHash>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
			Self::remove_claim(&sender, &proof)?;
            Self::deposit_event(Event::ClaimRevoked(sender, proof));
            Ok(())
        }

		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			to: T::AccountId,
            proof: BoundedVec<u8, T::MaxBytesInHash>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(from != to, <Error<T>>::TransferToSelf);
			Self::remove_claim(&from, &proof)?;
			Self::add_claim(&to, &proof)?;
			Self::deposit_event(Event::ClaimTransferred(from, to, proof));
			Ok(())
		}
    }

	impl<T: Config> Pallet<T> {
		// #[transactional]
		pub fn add_claim(
			account_id: &T::AccountId,
            proof: &BoundedVec<u8, T::MaxBytesInHash>,
		) -> Result<(), Error<T>> {
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(proof, (account_id, current_block));
			Ok(())
		}

		// #[transactional]
		pub fn remove_claim(
			account_id: &T::AccountId,
            proof: &BoundedVec<u8, T::MaxBytesInHash>,
		) -> Result<(), Error<T>> {
			ensure!(Proofs::<T>::contains_key(proof), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(proof).expect("All proofs must have an owner!");
			ensure!(*account_id == owner, Error::<T>::NotProofOwner);
			Proofs::<T>::remove(proof);
			Ok(())
		}
	}
}