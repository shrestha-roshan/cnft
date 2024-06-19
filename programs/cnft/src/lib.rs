use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{Metadata, MetadataAccount},
    token::Mint,
};
use mpl_bubblegum::types::{Collection, Creator, TokenProgramVersion, TokenStandard};
use mpl_bubblegum::{
    instructions::{
        CreateTreeConfigCpiBuilder, MintToCollectionV1Cpi, MintToCollectionV1CpiAccounts,
        MintToCollectionV1InstructionArgs,
    },
    types::MetadataArgs,
};
use solana_program::pubkey::Pubkey;
use spl_account_compression::{program::SplAccountCompression, Noop};

declare_id!("4bLGdf2gARVZNvgVaV3mWALr2ruSJPjAHESd5qfX3AVV");

pub const SEED: &str = "AUTH";

#[program]
pub mod cnft {

    use super::*;

    pub fn anchor_create_tree(
        ctx: Context<AnchorCreateTree>,
        max_depth: u32,
        max_buffer_size: u32,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[SEED.as_bytes(), &[ctx.bumps.pda]]];
        let compression_program = &ctx.accounts.compression_program.to_account_info();
        let tree_config = &ctx.accounts.tree_authority.to_account_info();
        let merkle_tree = &ctx.accounts.merkle_tree.to_account_info();
        let payer = &ctx.accounts.payer.to_account_info();
        let tree_creator = &ctx.accounts.pda.to_account_info();
        let log_wrapper = &ctx.accounts.log_wrapper.to_account_info();
        let system_program = &ctx.accounts.system_program.to_account_info();
        let bubblegum_program = &ctx.accounts.bubblegum_program.to_account_info();

        let binding = &mut CreateTreeConfigCpiBuilder::new(bubblegum_program);
        let cpi_create_tree = binding
            .compression_program(compression_program)
            .tree_config(tree_config)
            .merkle_tree(merkle_tree)
            .payer(payer)
            .tree_creator(tree_creator)
            .log_wrapper(log_wrapper)
            .system_program(system_program)
            .max_depth(max_depth)
            .max_buffer_size(max_buffer_size)
            .public(false);

        let _ = cpi_create_tree.invoke_signed(signer_seeds);

        Ok(())
    }

    pub fn mint_compressed_nft(ctx: Context<MintCompressedNft>) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[SEED.as_bytes(), &[ctx.bumps.pda]]];
        let tree_config = &ctx.accounts.tree_authority.to_account_info();
        let leaf_owner = &ctx.accounts.payer.to_account_info();
        let leaf_delegate = &ctx.accounts.payer.to_account_info();
        let merkle_tree = &ctx.accounts.merkle_tree.to_account_info();
        let payer = &ctx.accounts.payer.to_account_info();
        let tree_creator_or_delegate = &ctx.accounts.pda.to_account_info();
        let collection_authority = &ctx.accounts.pda.to_account_info();
        let bubblegum_program = &ctx.accounts.bubblegum_program.to_account_info();
        let collection_mint = &ctx.accounts.collection_mint.to_account_info();
        let collection_metadata = &ctx.accounts.collection_metadata.to_account_info();
        let collection_edition = &ctx.accounts.edition_account.to_account_info();
        let bubblegum_signer = &ctx.accounts.bubblegum_signer.to_account_info();
        let log_wrapper = &ctx.accounts.log_wrapper.to_account_info();
        let compression_program = &ctx.accounts.compression_program.to_account_info();
        let token_metadata_program = &ctx.accounts.token_metadata_program.to_account_info();
        let system_program = &ctx.accounts.system_program.to_account_info();

        // use collection nft metadata as the metadata for the compressed nft
        let metadata_account = &ctx.accounts.collection_metadata;

        let metadata = MetadataArgs {
            name: metadata_account.name.to_string(),
            symbol: metadata_account.symbol.to_string(),
            uri: metadata_account.uri.to_string(),
            collection: Some(Collection {
                key: ctx.accounts.collection_mint.key(),
                verified: false,
            }),
            primary_sale_happened: true,
            is_mutable: true,
            edition_nonce: None,
            token_standard: Some(TokenStandard::NonFungible),
            uses: None,
            token_program_version: TokenProgramVersion::Original,
            creators: vec![Creator {
                address: ctx.accounts.pda.key(), // set creator as pda
                verified: true,
                share: 100,
            }],
            seller_fee_basis_points: 0,
        };

        let cpi_mint = MintToCollectionV1Cpi::new(
            bubblegum_program,
            MintToCollectionV1CpiAccounts {
                tree_config,
                leaf_owner,
                leaf_delegate,
                merkle_tree,
                payer,
                tree_creator_or_delegate, // tree delegate is pda, required as a signer
                collection_authority, // collection authority is pda (nft metadata update authority)
                collection_authority_record_pda: Some(bubblegum_program),
                collection_mint,     // collection nft mint account
                collection_metadata, // collection nft metadata account
                collection_edition,  // collection nft master edition account
                bubblegum_signer,
                log_wrapper,
                compression_program,
                token_metadata_program,
                system_program,
            },
            MintToCollectionV1InstructionArgs { metadata: metadata },
        );

        let _ = cpi_mint.invoke_signed(signer_seeds);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AnchorCreateTree<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK:
    #[account(
        seeds = [SEED.as_bytes()],
        bump,
    )]
    pub pda: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub system_program: Program<'info, System>,
    /// CHECK:
    pub bubblegum_program: AccountInfo<'info>,
    pub compression_program: Program<'info, SplAccountCompression>,
}

#[derive(Accounts)]
pub struct MintCompressedNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK:
    #[account(
        seeds = [SEED.as_bytes()],
        bump,
    )]
    pub pda: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        seeds = ["collection_cpi".as_bytes()],
        seeds::program = bubblegum_program.key(),
        bump,
    )]
    pub bubblegum_signer: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    /// CHECK:
    pub bubblegum_program: AccountInfo<'info>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,

    pub collection_mint: Account<'info, Mint>,
    #[account(mut)]
    pub collection_metadata: Account<'info, MetadataAccount>,
    /// CHECK:
    pub edition_account: UncheckedAccount<'info>,
}
