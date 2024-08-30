use {
    crate::{state::*, utils::*},
    anchor_lang::prelude::*,
    anchor_spl::token_interface,
    std::mem::size_of,
};

pub fn handler_ip(ctx: Context<InitializePool>) -> Result<()> {
    check_token_program(ctx.accounts.token_program.key());

    // initialize pool state
    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.bump = ctx.bumps.pool_state;
    pool_state.amount = 0;
    pool_state.vault_bump = ctx.bumps.token_vault;
    pool_state.vault_auth_bump = ctx.bumps.pool_authority;
    pool_state.token_mint = ctx.accounts.token_mint.key();
    pool_state.staking_token_mint = ctx.accounts.staking_token_mint.key();
    pool_state.vault_authority = ctx.accounts.pool_authority.key();

    msg!("Pool state initialized");

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    // pool authority account
    /// CHECK: PDA, auth over all token vaults
    #[account(
        seeds = [VAULT_AUTH_SEED.as_bytes()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    // pool state account
    #[account(
        init,
        seeds = [token_mint.key().as_ref(), STAKE_POOL_STATE_SEED.as_bytes()],
        bump,
        payer = payer,
        space = 8 + size_of::<PoolState>(),
    )]
    pub pool_state: Account<'info, PoolState>,

    // token mint account
    #[account(
        mint::authority = payer,
        mint::token_program = token_program,
    )]
    pub token_mint: InterfaceAccount<'info, token_interface::Mint>,

    // pool token vault account
    #[account(
        init,
        payer = payer,
        seeds = [token_mint.key().as_ref(), pool_authority.key().as_ref(), VAULT_SEED.as_bytes()],
        bump,
        token::mint = token_mint,
        token::authority = pool_authority,
        token::token_program = token_program,
    )]
    pub token_vault: InterfaceAccount<'info, token_interface::TokenAccount>,

    // staking token mint account
    #[account(
        mut,
        mint::token_program = token_program,
    )]
    pub staking_token_mint: InterfaceAccount<'info, token_interface::Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
