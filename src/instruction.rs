use {
    crate::{
        state::{EDITION, EDITION_MARKER_BIT_SIZE, PREFIX},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct CreateMasterEditionArgs {
    /// If set, means that no more than this number of editions can ever be minted. This is immutable.
    pub max_supply: Option<u64>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct MintNewEditionFromMasterEditionViaTokenArgs {
    pub edition: u64,
}

/// Instructions supported by the Metadata program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum MetadataInstruction {
    /// Allows updating the primary sale boolean on Metadata solely through owning an account
    /// containing a token from the metadata's mint and being a signer on this transaction.
    /// A sort of limited authority for limited update capability that is required for things like
    /// Metaplex to work without needing full authority passing.
    ///
    ///   0. `[writable]` Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[signer]` Owner on the token account
    ///   2. `[]` Account containing tokens from the metadata's mint
    UpdatePrimarySaleHappenedViaToken,

    /// Register a Metadata as a Master Edition V2, which means Edition V2s can be minted.
    /// Henceforth, no further tokens will be mintable from this primary mint. Will throw an error if more than one
    /// token exists, and will throw an error if less than one token exists in this primary mint.
    ///   0. `[writable]` Unallocated edition V2 account with address as pda of ['metadata', program id, mint, 'edition']
    ///   1. `[writable]` Metadata mint
    ///   2. `[signer]` Update authority
    ///   3. `[signer]` Mint authority on the metadata's mint - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[signer]` payer
    ///   5. `[]` Metadata account
    ///   6. `[]` Token program
    ///   7. `[]` System program
    ///   8. `[]` Rent info
    CreateMasterEdition(CreateMasterEditionArgs),

    /// Given a token account containing the master edition token to prove authority, and a brand new non-metadata-ed mint with one token
    /// make a new Metadata + Edition that is a child of the master edition denoted by this authority token.
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number])
    ///   where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
    ///   5. `[signer]` Mint authority of new mint
    ///   6. `[signer]` payer
    ///   7. `[signer]` owner of token account containing master token (#8)
    ///   8. `[]` token account containing token from master metadata mint
    ///   9. `[]` Update authority info for new metadata
    ///   10. `[]` Master record metadata account
    ///   11. `[]` Token program
    ///   12. `[]` System program
    ///   13. `[]` Rent info
    MintNewEditionFromMasterEditionViaToken(MintNewEditionFromMasterEditionViaTokenArgs),

    /// Proxy Call to Mint Edition using a Store Token Account as a Vault Authority.
    ///
    ///   0. `[writable]` New Metadata key (pda of ['metadata', program id, mint id])
    ///   1. `[writable]` New Edition (pda of ['metadata', program id, mint id, 'edition'])
    ///   2. `[writable]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition']
    ///   3. `[writable]` Mint of new token - THIS WILL TRANSFER AUTHORITY AWAY FROM THIS KEY
    ///   4. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number])
    ///   where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
    ///   5. `[signer]` Mint authority of new mint
    ///   6. `[signer]` payer
    ///   7. `[signer]` Vault authority
    ///   8. `[]` Safety deposit token store account
    ///   9. `[]` Safety deposit box
    ///   10. `[]` Vault
    ///   11. `[]` Update authority info for new metadata
    ///   12. `[]` Master record metadata account
    ///   13. `[]` Token program
    ///   14. `[]` Token vault program
    ///   15. `[]` System program
    ///   16. `[]` Rent info
    MintNewEditionFromMasterEditionViaVaultProxy(MintNewEditionFromMasterEditionViaTokenArgs),
}

/// creates a update_primary_sale_happened_via_token instruction
#[allow(clippy::too_many_arguments)]
pub fn update_primary_sale_happened_via_token(
    program_id: Pubkey,
    metadata: Pubkey,
    owner: Pubkey,
    token: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(metadata, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(token, false),
        ],
        data: MetadataInstruction::UpdatePrimarySaleHappenedViaToken
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a create_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn create_master_edition(
    program_id: Pubkey,
    edition: Pubkey,
    mint: Pubkey,
    update_authority: Pubkey,
    mint_authority: Pubkey,
    metadata: Pubkey,
    payer: Pubkey,
    max_supply: Option<u64>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(edition, false),
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(update_authority, true),
        AccountMeta::new_readonly(mint_authority, true),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::CreateMasterEdition(CreateMasterEditionArgs { max_supply })
            .try_to_vec()
            .unwrap(),
    }
}

/// creates a mint_new_edition_from_master_edition instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_new_edition_from_master_edition_via_token(
    program_id: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
    master_edition: Pubkey,
    new_mint: Pubkey,
    new_mint_authority: Pubkey,
    payer: Pubkey,
    token_account_owner: Pubkey,
    token_account: Pubkey,
    new_metadata_update_authority: Pubkey,
    metadata: Pubkey,
    metadata_mint: Pubkey,
    edition: u64,
) -> Instruction {
    let edition_number = edition.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();
    let as_string = edition_number.to_string();
    let (edition_mark_pda, _) = Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            metadata_mint.as_ref(),
            EDITION.as_bytes(),
            as_string.as_bytes(),
        ],
        &program_id,
    );

    let accounts = vec![
        AccountMeta::new(new_metadata, false),
        AccountMeta::new(new_edition, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(new_mint, false),
        AccountMeta::new(edition_mark_pda, false),
        AccountMeta::new_readonly(new_mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(token_account_owner, true),
        AccountMeta::new_readonly(token_account, false),
        AccountMeta::new_readonly(new_metadata_update_authority, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaToken(
            MintNewEditionFromMasterEditionViaTokenArgs { edition },
        )
        .try_to_vec()
        .unwrap(),
    }
}

/// creates a mint_edition_proxy instruction
#[allow(clippy::too_many_arguments)]
pub fn mint_edition_from_master_edition_via_vault_proxy(
    program_id: Pubkey,
    new_metadata: Pubkey,
    new_edition: Pubkey,
    master_edition: Pubkey,
    new_mint: Pubkey,
    edition_mark_pda: Pubkey,
    new_mint_authority: Pubkey,
    payer: Pubkey,
    vault_authority: Pubkey,
    safety_deposit_store: Pubkey,
    safety_deposit_box: Pubkey,
    vault: Pubkey,
    new_metadata_update_authority: Pubkey,
    metadata: Pubkey,
    token_program: Pubkey,
    token_vault_program_info: Pubkey,
    edition: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(new_metadata, false),
        AccountMeta::new(new_edition, false),
        AccountMeta::new(master_edition, false),
        AccountMeta::new(new_mint, false),
        AccountMeta::new(edition_mark_pda, false),
        AccountMeta::new_readonly(new_mint_authority, true),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(vault_authority, true),
        AccountMeta::new_readonly(safety_deposit_store, false),
        AccountMeta::new_readonly(safety_deposit_box, false),
        AccountMeta::new_readonly(vault, false),
        AccountMeta::new_readonly(new_metadata_update_authority, false),
        AccountMeta::new_readonly(metadata, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(token_vault_program_info, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: MetadataInstruction::MintNewEditionFromMasterEditionViaVaultProxy(
            MintNewEditionFromMasterEditionViaTokenArgs { edition },
        )
        .try_to_vec()
        .unwrap(),
    }
}
