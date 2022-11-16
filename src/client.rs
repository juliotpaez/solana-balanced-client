use crate::SolanaClientRpc;
use serde_json::Value;
use solana_account_decoder::parse_token::{UiTokenAccount, UiTokenAmount};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::{
    GetConfirmedSignaturesForAddress2Config, SerializableMessage, SerializableTransaction,
};
use solana_client::rpc_config::{
    RpcAccountInfoConfig, RpcBlockConfig, RpcBlockProductionConfig, RpcGetVoteAccountsConfig,
    RpcLargestAccountsConfig, RpcLeaderScheduleConfig, RpcProgramAccountsConfig,
    RpcRequestAirdropConfig, RpcSendTransactionConfig, RpcSimulateTransactionConfig,
    RpcTransactionConfig,
};
use solana_client::rpc_request::{RpcRequest, TokenAccountsFilter};
use solana_client::rpc_response::{
    RpcAccountBalance, RpcBlockProduction, RpcConfirmedTransactionStatusWithSignature,
    RpcContactInfo, RpcInflationGovernor, RpcInflationRate, RpcInflationReward, RpcKeyedAccount,
    RpcLeaderSchedule, RpcPerfSample, RpcResult, RpcSimulateTransactionResult, RpcSnapshotSlotInfo,
    RpcStakeActivation, RpcSupply, RpcVersionInfo, RpcVoteAccountStatus,
};
use solana_sdk::account::Account;
use solana_sdk::clock::{Epoch, Slot, UnixTimestamp};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::epoch_info::EpochInfo;
use solana_sdk::epoch_schedule::EpochSchedule;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction;
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedConfirmedTransactionWithStatusMeta, TransactionStatus,
    UiConfirmedBlock, UiTransactionEncoding,
};
use std::sync::Arc;
use tokio::sync::Mutex;

macro_rules! impl_method {
    ([$($endpoint:ident),+ $(,)?], $method:ident, $self:ident, $($args:ident),* $(,)?) => {{
        // Pick RPC.
        let rpc = {
            let mut inner = $self.inner.try_lock().unwrap();

            let mut index = inner.next_rpc_index;
            loop {
                let rpc = &mut inner.rpc_list[index];

                // Check RPC limits.
                if rpc.execute_endpoints(&[$(RpcRequest::$endpoint),+]) {
                    let rpc = rpc.rpc_client.clone();

                    // Update index.
                    if inner.move_rpc_index {
                        inner.next_rpc_index = index;
                    }

                    break rpc;
                }

                // Update index.
                index = (index + 1) % inner.rpc_list.len();

                // Break on loop.
                if index == inner.next_rpc_index {
                    break inner.default_rpc.clone();
                }
            }
        };

        // Send request
        rpc.$method($($args),*).await
    }};
}

pub type SolanaClientRef = Arc<SolanaClient>;
type ClientResult<T> = solana_client::client_error::Result<T>;

/// An asynchronous Solana client that can manage multiple RPC clients, each of them with its own
/// limits and configuration.
pub struct SolanaClient {
    pub inner: Mutex<SolanaClientInner>,
}

/// Data of [`SolanaClient`]. This is separated from the struct to allow
/// internal mutability.
pub struct SolanaClientInner {
    pub next_rpc_index: usize,
    pub move_rpc_index: bool,
    pub rpc_list: Vec<SolanaClientRpc>,
    pub default_rpc: Arc<RpcClient>,
}

impl SolanaClient {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a new `SolanaClient` with a default RPC client.
    pub fn new_with_default(default_rpc: Arc<RpcClient>) -> Self {
        Self {
            inner: Mutex::new(SolanaClientInner {
                next_rpc_index: 0,
                move_rpc_index: false,
                rpc_list: Vec::new(),
                default_rpc,
            }),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a new [`SolanaClientRpc`] to the RPC list.
    pub fn add_rpc(self, rpc: SolanaClientRpc) -> Self {
        let mut inner = self.inner.try_lock().unwrap();
        inner.rpc_list.push(rpc);

        drop(inner);

        self
    }

    /// Whether the index must move forward when an RPC cannot be used.
    pub fn set_move_rpc_index(self, move_index: bool) -> Self {
        let mut inner = self.inner.try_lock().unwrap();
        inner.move_rpc_index = move_index;

        drop(inner);

        self
    }

    // SOLANA RPC ENDPOINTS ---------------------------------------------------

    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &impl SerializableTransaction,
    ) -> ClientResult<Signature> {
        if transaction.uses_durable_nonce() {
            impl_method!(
                [GetLatestBlockhash, SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction,
                self,
                transaction,
            )
        } else {
            impl_method!(
                [SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction,
                self,
                transaction,
            )
        }
    }

    pub async fn send_and_confirm_transaction_with_spinner(
        &self,
        transaction: &impl SerializableTransaction,
    ) -> ClientResult<Signature> {
        if transaction.uses_durable_nonce() {
            impl_method!(
                [GetLatestBlockhash, SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner,
                self,
                transaction,
            )
        } else {
            impl_method!(
                [SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner,
                self,
                transaction,
            )
        }
    }

    pub async fn send_and_confirm_transaction_with_spinner_and_commitment(
        &self,
        transaction: &impl SerializableTransaction,
        commitment: CommitmentConfig,
    ) -> ClientResult<Signature> {
        if transaction.uses_durable_nonce() {
            impl_method!(
                [GetLatestBlockhash, SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner_and_commitment,
                self,
                transaction,
                commitment,
            )
        } else {
            impl_method!(
                [SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner_and_commitment,
                self,
                transaction,
                commitment,
            )
        }
    }

    pub async fn send_and_confirm_transaction_with_spinner_and_config(
        &self,
        transaction: &impl SerializableTransaction,
        commitment: CommitmentConfig,
        config: RpcSendTransactionConfig,
    ) -> ClientResult<Signature> {
        if transaction.uses_durable_nonce() {
            impl_method!(
                [GetLatestBlockhash, SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner_and_config,
                self,
                transaction,
                commitment,
                config
            )
        } else {
            impl_method!(
                [SendTransaction, GetSignatureStatuses],
                send_and_confirm_transaction_with_spinner_and_config,
                self,
                transaction,
                commitment,
                config
            )
        }
    }

    pub async fn send_transaction(
        &self,
        transaction: &impl SerializableTransaction,
    ) -> ClientResult<Signature> {
        impl_method!([SendTransaction], send_transaction, self, transaction)
    }

    pub async fn send_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSendTransactionConfig,
    ) -> ClientResult<Signature> {
        impl_method!(
            [SendTransaction],
            send_transaction_with_config,
            self,
            transaction,
            config
        )
    }

    pub async fn confirm_transaction(&self, signature: &Signature) -> ClientResult<bool> {
        impl_method!([GetSignatureStatuses], confirm_transaction, self, signature,)
    }

    pub async fn confirm_transaction_with_commitment(
        &self,
        signature: &Signature,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<bool> {
        impl_method!(
            [GetSignatureStatuses],
            confirm_transaction_with_commitment,
            self,
            signature,
            commitment_config,
        )
    }

    pub async fn confirm_transaction_with_spinner(
        &self,
        signature: &Signature,
        recent_blockhash: &Hash,
        commitment: CommitmentConfig,
    ) -> ClientResult<()> {
        impl_method!(
            [GetSignatureStatuses],
            confirm_transaction_with_spinner,
            self,
            signature,
            recent_blockhash,
            commitment,
        )
    }

    pub async fn simulate_transaction(
        &self,
        transaction: &impl SerializableTransaction,
    ) -> RpcResult<RpcSimulateTransactionResult> {
        impl_method!(
            [SimulateTransaction],
            simulate_transaction,
            self,
            transaction
        )
    }

    pub async fn simulate_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSimulateTransactionConfig,
    ) -> RpcResult<RpcSimulateTransactionResult> {
        impl_method!(
            [SimulateTransaction],
            simulate_transaction_with_config,
            self,
            transaction,
            config
        )
    }

    pub async fn get_highest_snapshot_slot(&self) -> ClientResult<RpcSnapshotSlotInfo> {
        impl_method!([GetHighestSnapshotSlot], get_highest_snapshot_slot, self,)
    }

    pub async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> ClientResult<Option<transaction::Result<()>>> {
        impl_method!(
            [GetSignatureStatuses],
            get_signature_status,
            self,
            signature,
        )
    }

    pub async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> RpcResult<Vec<Option<TransactionStatus>>> {
        impl_method!(
            [GetSignatureStatuses],
            get_signature_statuses,
            self,
            signatures,
        )
    }

    pub async fn get_signature_statuses_with_history(
        &self,
        signatures: &[Signature],
    ) -> RpcResult<Vec<Option<TransactionStatus>>> {
        impl_method!(
            [GetSignatureStatuses],
            get_signature_statuses_with_history,
            self,
            signatures,
        )
    }

    pub async fn get_signature_status_with_commitment(
        &self,
        signature: &Signature,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<Option<transaction::Result<()>>> {
        impl_method!(
            [GetSignatureStatuses],
            get_signature_status_with_commitment,
            self,
            signature,
            commitment_config
        )
    }

    pub async fn get_signature_status_with_commitment_and_history(
        &self,
        signature: &Signature,
        commitment_config: CommitmentConfig,
        search_transaction_history: bool,
    ) -> ClientResult<Option<transaction::Result<()>>> {
        impl_method!(
            [GetSignatureStatuses],
            get_signature_status_with_commitment_and_history,
            self,
            signature,
            commitment_config,
            search_transaction_history
        )
    }

    pub async fn get_slot(&self) -> ClientResult<Slot> {
        impl_method!([GetSlot], get_slot, self,)
    }

    pub async fn get_slot_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<Slot> {
        impl_method!([GetSlot], get_slot_with_commitment, self, commitment_config,)
    }

    pub async fn get_block_height(&self) -> ClientResult<u64> {
        impl_method!([GetBlockHeight], get_block_height, self,)
    }

    pub async fn get_block_height_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetBlockHeight],
            get_block_height_with_commitment,
            self,
            commitment_config,
        )
    }

    pub async fn get_slot_leaders(
        &self,
        start_slot: Slot,
        limit: u64,
    ) -> ClientResult<Vec<Pubkey>> {
        impl_method!([GetSlotLeaders], get_slot_leaders, self, start_slot, limit)
    }

    pub async fn get_block_production(&self) -> RpcResult<RpcBlockProduction> {
        impl_method!([GetBlockProduction], get_block_production, self,)
    }

    pub async fn get_block_production_with_config(
        &self,
        config: RpcBlockProductionConfig,
    ) -> RpcResult<RpcBlockProduction> {
        impl_method!(
            [GetBlockProduction],
            get_block_production_with_config,
            self,
            config
        )
    }

    pub async fn get_stake_activation(
        &self,
        stake_account: Pubkey,
        epoch: Option<Epoch>,
    ) -> ClientResult<RpcStakeActivation> {
        impl_method!(
            [GetStakeActivation],
            get_stake_activation,
            self,
            stake_account,
            epoch
        )
    }

    pub async fn supply(&self) -> RpcResult<RpcSupply> {
        impl_method!([GetSupply], supply, self,)
    }

    pub async fn supply_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<RpcSupply> {
        impl_method!([GetSupply], supply_with_commitment, self, commitment_config,)
    }

    pub async fn get_largest_accounts_with_config(
        &self,
        config: RpcLargestAccountsConfig,
    ) -> RpcResult<Vec<RpcAccountBalance>> {
        impl_method!(
            [GetLargestAccounts],
            get_largest_accounts_with_config,
            self,
            config,
        )
    }

    pub async fn get_vote_accounts(&self) -> ClientResult<RpcVoteAccountStatus> {
        impl_method!([GetVoteAccounts], get_vote_accounts, self,)
    }

    pub async fn get_vote_accounts_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<RpcVoteAccountStatus> {
        impl_method!(
            [GetVoteAccounts],
            get_vote_accounts_with_commitment,
            self,
            commitment_config,
        )
    }

    pub async fn get_vote_accounts_with_config(
        &self,
        config: RpcGetVoteAccountsConfig,
    ) -> ClientResult<RpcVoteAccountStatus> {
        impl_method!(
            [GetVoteAccounts],
            get_vote_accounts_with_config,
            self,
            config,
        )
    }

    pub async fn wait_for_max_stake(
        &self,
        commitment: CommitmentConfig,
        max_stake_percent: f32,
    ) -> ClientResult<()> {
        impl_method!(
            [GetVoteAccounts],
            wait_for_max_stake,
            self,
            commitment,
            max_stake_percent,
        )
    }

    pub async fn get_cluster_nodes(&self) -> ClientResult<Vec<RpcContactInfo>> {
        impl_method!([GetClusterNodes], get_cluster_nodes, self,)
    }

    pub async fn get_block(&self, slot: Slot) -> ClientResult<EncodedConfirmedBlock> {
        impl_method!([GetBlock], get_block, self, slot)
    }

    pub async fn get_block_with_encoding(
        &self,
        slot: Slot,
        encoding: UiTransactionEncoding,
    ) -> ClientResult<EncodedConfirmedBlock> {
        impl_method!([GetBlock], get_block_with_encoding, self, slot, encoding)
    }

    pub async fn get_block_with_config(
        &self,
        slot: Slot,
        config: RpcBlockConfig,
    ) -> ClientResult<UiConfirmedBlock> {
        impl_method!([GetBlock], get_block_with_config, self, slot, config)
    }

    pub async fn get_blocks(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
    ) -> ClientResult<Vec<Slot>> {
        impl_method!([GetBlocks], get_blocks, self, start_slot, end_slot,)
    }

    pub async fn get_blocks_with_commitment(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<Vec<Slot>> {
        impl_method!(
            [GetBlocks],
            get_blocks_with_commitment,
            self,
            start_slot,
            end_slot,
            commitment_config
        )
    }

    pub async fn get_blocks_with_limit(
        &self,
        start_slot: Slot,
        limit: usize,
    ) -> ClientResult<Vec<Slot>> {
        impl_method!(
            [GetBlocksWithLimit],
            get_blocks_with_limit,
            self,
            start_slot,
            limit,
        )
    }

    pub async fn get_blocks_with_limit_and_commitment(
        &self,
        start_slot: Slot,
        limit: usize,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<Vec<Slot>> {
        impl_method!(
            [GetBlocksWithLimit],
            get_blocks_with_limit_and_commitment,
            self,
            start_slot,
            limit,
            commitment_config
        )
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
    ) -> ClientResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        impl_method!(
            [GetSignaturesForAddress],
            get_signatures_for_address,
            self,
            address,
        )
    }

    pub async fn get_signatures_for_address_with_config(
        &self,
        address: &Pubkey,
        config: GetConfirmedSignaturesForAddress2Config,
    ) -> ClientResult<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        impl_method!(
            [GetSignaturesForAddress],
            get_signatures_for_address_with_config,
            self,
            address,
            config
        )
    }

    pub async fn get_transaction(
        &self,
        signature: &Signature,
        encoding: UiTransactionEncoding,
    ) -> ClientResult<EncodedConfirmedTransactionWithStatusMeta> {
        impl_method!([GetTransaction], get_transaction, self, signature, encoding)
    }

    pub async fn get_transaction_with_config(
        &self,
        signature: &Signature,
        config: RpcTransactionConfig,
    ) -> ClientResult<EncodedConfirmedTransactionWithStatusMeta> {
        impl_method!(
            [GetTransaction],
            get_transaction_with_config,
            self,
            signature,
            config
        )
    }

    pub async fn get_block_time(&self, slot: Slot) -> ClientResult<UnixTimestamp> {
        impl_method!([GetBlockTime], get_block_time, self, slot)
    }

    pub async fn get_epoch_info(&self) -> ClientResult<EpochInfo> {
        impl_method!([GetEpochInfo], get_epoch_info, self,)
    }

    pub async fn get_epoch_info_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<EpochInfo> {
        impl_method!(
            [GetEpochInfo],
            get_epoch_info_with_commitment,
            self,
            commitment_config
        )
    }

    pub async fn get_leader_schedule(
        &self,
        slot: Option<Slot>,
    ) -> ClientResult<Option<RpcLeaderSchedule>> {
        impl_method!([GetLeaderSchedule], get_leader_schedule, self, slot,)
    }

    pub async fn get_leader_schedule_with_commitment(
        &self,
        slot: Option<Slot>,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<Option<RpcLeaderSchedule>> {
        impl_method!(
            [GetLeaderSchedule],
            get_leader_schedule_with_commitment,
            self,
            slot,
            commitment_config
        )
    }

    pub async fn get_leader_schedule_with_config(
        &self,
        slot: Option<Slot>,
        config: RpcLeaderScheduleConfig,
    ) -> ClientResult<Option<RpcLeaderSchedule>> {
        impl_method!(
            [GetLeaderSchedule],
            get_leader_schedule_with_config,
            self,
            slot,
            config
        )
    }

    pub async fn get_epoch_schedule(&self) -> ClientResult<EpochSchedule> {
        impl_method!([GetEpochSchedule], get_epoch_schedule, self,)
    }

    pub async fn get_recent_performance_samples(
        &self,
        limit: Option<usize>,
    ) -> ClientResult<Vec<RpcPerfSample>> {
        impl_method!(
            [GetRecentPerformanceSamples],
            get_recent_performance_samples,
            self,
            limit,
        )
    }

    pub async fn get_identity(&self) -> ClientResult<Pubkey> {
        impl_method!([GetIdentity], get_identity, self,)
    }

    pub async fn get_inflation_governor(&self) -> ClientResult<RpcInflationGovernor> {
        impl_method!([GetInflationGovernor], get_inflation_governor, self,)
    }

    pub async fn get_inflation_rate(&self) -> ClientResult<RpcInflationRate> {
        impl_method!([GetInflationRate], get_inflation_rate, self,)
    }

    pub async fn get_inflation_reward(
        &self,
        addresses: &[Pubkey],
        epoch: Option<Epoch>,
    ) -> ClientResult<Vec<Option<RpcInflationReward>>> {
        impl_method!(
            [GetInflationReward],
            get_inflation_reward,
            self,
            addresses,
            epoch
        )
    }

    pub async fn get_version(&self) -> ClientResult<RpcVersionInfo> {
        impl_method!([GetVersion], get_version, self,)
    }

    pub async fn minimum_ledger_slot(&self) -> ClientResult<Slot> {
        impl_method!([MinimumLedgerSlot], minimum_ledger_slot, self,)
    }

    pub async fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Account> {
        impl_method!([GetAccountInfo], get_account, self, pubkey)
    }

    pub async fn get_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<Option<Account>> {
        impl_method!(
            [GetAccountInfo],
            get_account_with_commitment,
            self,
            pubkey,
            commitment_config,
        )
    }

    pub async fn get_account_with_config(
        &self,
        pubkey: &Pubkey,
        config: RpcAccountInfoConfig,
    ) -> RpcResult<Option<Account>> {
        impl_method!(
            [GetAccountInfo],
            get_account_with_config,
            self,
            pubkey,
            config,
        )
    }

    pub async fn get_max_retransmit_slot(&self) -> ClientResult<Slot> {
        impl_method!([GetMaxRetransmitSlot], get_max_retransmit_slot, self,)
    }

    pub async fn get_max_shred_insert_slot(&self) -> ClientResult<Slot> {
        impl_method!([GetMaxShredInsertSlot], get_max_shred_insert_slot, self,)
    }

    pub async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<Option<Account>>> {
        impl_method!([GetMultipleAccounts], get_multiple_accounts, self, pubkeys,)
    }

    pub async fn get_multiple_accounts_with_commitment(
        &self,
        pubkeys: &[Pubkey],
        commitment_config: CommitmentConfig,
    ) -> RpcResult<Vec<Option<Account>>> {
        impl_method!(
            [GetMultipleAccounts],
            get_multiple_accounts_with_commitment,
            self,
            pubkeys,
            commitment_config
        )
    }

    pub async fn get_multiple_accounts_with_config(
        &self,
        pubkeys: &[Pubkey],
        config: RpcAccountInfoConfig,
    ) -> RpcResult<Vec<Option<Account>>> {
        impl_method!(
            [GetMultipleAccounts],
            get_multiple_accounts_with_config,
            self,
            pubkeys,
            config
        )
    }

    pub async fn get_account_data(&self, pubkey: &Pubkey) -> ClientResult<Vec<u8>> {
        impl_method!([GetAccountInfo], get_account_data, self, pubkey,)
    }

    pub async fn get_minimum_balance_for_rent_exemption(
        &self,
        data_len: usize,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetMinimumBalanceForRentExemption],
            get_minimum_balance_for_rent_exemption,
            self,
            data_len,
        )
    }

    pub async fn get_balance(&self, pubkey: &Pubkey) -> ClientResult<u64> {
        impl_method!([GetBalance], get_balance, self, pubkey,)
    }

    pub async fn get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<u64> {
        impl_method!(
            [GetBalance],
            get_balance_with_commitment,
            self,
            pubkey,
            commitment_config,
        )
    }

    pub async fn get_program_accounts(
        &self,
        pubkey: &Pubkey,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        impl_method!([GetProgramAccounts], get_program_accounts, self, pubkey,)
    }

    pub async fn get_program_accounts_with_config(
        &self,
        pubkey: &Pubkey,
        config: RpcProgramAccountsConfig,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        impl_method!(
            [GetProgramAccounts],
            get_program_accounts_with_config,
            self,
            pubkey,
            config
        )
    }

    pub async fn get_stake_minimum_delegation(&self) -> ClientResult<u64> {
        impl_method!(
            [GetStakeMinimumDelegation],
            get_stake_minimum_delegation,
            self,
        )
    }

    pub async fn get_stake_minimum_delegation_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetStakeMinimumDelegation],
            get_transaction_count_with_commitment,
            self,
            commitment_config
        )
    }

    /// Request the transaction count.
    pub async fn get_transaction_count(&self) -> ClientResult<u64> {
        impl_method!([GetTransactionCount], get_transaction_count, self,)
    }

    pub async fn get_transaction_count_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetTransactionCount],
            get_transaction_count_with_commitment,
            self,
            commitment_config
        )
    }

    pub async fn get_first_available_block(&self) -> ClientResult<Slot> {
        impl_method!([GetFirstAvailableBlock], get_first_available_block, self,)
    }

    pub async fn get_genesis_hash(&self) -> ClientResult<Hash> {
        impl_method!([GetGenesisHash], get_genesis_hash, self,)
    }

    pub async fn get_health(&self) -> ClientResult<()> {
        impl_method!([GetHealth], get_health, self,)
    }

    pub async fn get_token_account(&self, pubkey: &Pubkey) -> ClientResult<Option<UiTokenAccount>> {
        impl_method!([GetAccountInfo], get_token_account, self, pubkey)
    }

    pub async fn get_token_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<Option<UiTokenAccount>> {
        impl_method!(
            [GetAccountInfo],
            get_token_account_with_commitment,
            self,
            pubkey,
            commitment_config
        )
    }

    pub async fn get_token_account_balance(&self, pubkey: &Pubkey) -> ClientResult<UiTokenAmount> {
        impl_method!(
            [GetTokenAccountBalance],
            get_token_account_balance,
            self,
            pubkey
        )
    }

    pub async fn get_token_account_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<UiTokenAmount> {
        impl_method!(
            [GetTokenAccountBalance],
            get_token_account_balance_with_commitment,
            self,
            pubkey,
            commitment_config
        )
    }

    pub async fn get_token_accounts_by_delegate(
        &self,
        delegate: &Pubkey,
        token_account_filter: TokenAccountsFilter,
    ) -> ClientResult<Vec<RpcKeyedAccount>> {
        impl_method!(
            [GetTokenAccountsByOwner],
            get_token_accounts_by_delegate,
            self,
            delegate,
            token_account_filter
        )
    }

    pub async fn get_token_accounts_by_delegate_with_commitment(
        &self,
        delegate: &Pubkey,
        token_account_filter: TokenAccountsFilter,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        impl_method!(
            [GetTokenAccountsByOwner],
            get_token_accounts_by_delegate_with_commitment,
            self,
            delegate,
            token_account_filter,
            commitment_config
        )
    }

    pub async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
        token_account_filter: TokenAccountsFilter,
    ) -> ClientResult<Vec<RpcKeyedAccount>> {
        impl_method!(
            [GetTokenAccountsByOwner],
            get_token_accounts_by_owner,
            self,
            owner,
            token_account_filter,
        )
    }

    pub async fn get_token_accounts_by_owner_with_commitment(
        &self,
        owner: &Pubkey,
        token_account_filter: TokenAccountsFilter,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<Vec<RpcKeyedAccount>> {
        impl_method!(
            [GetTokenAccountsByOwner],
            get_token_accounts_by_owner_with_commitment,
            self,
            owner,
            token_account_filter,
            commitment_config
        )
    }

    pub async fn get_token_supply(&self, mint: &Pubkey) -> ClientResult<UiTokenAmount> {
        impl_method!([GetTokenSupply], get_token_supply, self, mint,)
    }

    pub async fn get_token_supply_with_commitment(
        &self,
        mint: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> RpcResult<UiTokenAmount> {
        impl_method!(
            [GetTokenSupply],
            get_token_supply_with_commitment,
            self,
            mint,
            commitment_config,
        )
    }

    pub async fn request_airdrop(&self, pubkey: &Pubkey, lamports: u64) -> ClientResult<Signature> {
        impl_method!([RequestAirdrop], request_airdrop, self, pubkey, lamports,)
    }

    pub async fn request_airdrop_with_blockhash(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> ClientResult<Signature> {
        impl_method!(
            [RequestAirdrop],
            request_airdrop_with_blockhash,
            self,
            pubkey,
            lamports,
            recent_blockhash
        )
    }

    pub async fn request_airdrop_with_config(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        config: RpcRequestAirdropConfig,
    ) -> ClientResult<Signature> {
        impl_method!(
            [RequestAirdrop],
            request_airdrop_with_config,
            self,
            pubkey,
            lamports,
            config
        )
    }

    pub async fn poll_get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetBalance],
            poll_get_balance_with_commitment,
            self,
            pubkey,
            commitment_config,
        )
    }

    pub async fn wait_for_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        expected_balance: Option<u64>,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<u64> {
        impl_method!(
            [GetBalance],
            wait_for_balance_with_commitment,
            self,
            pubkey,
            expected_balance,
            commitment_config,
        )
    }

    /// Poll the server to confirm a transaction.
    pub async fn poll_for_signature(&self, signature: &Signature) -> ClientResult<()> {
        impl_method!([GetSignatureStatuses], poll_for_signature, self, signature,)
    }

    /// Poll the server to confirm a transaction.
    pub async fn poll_for_signature_with_commitment(
        &self,
        signature: &Signature,
        commitment_config: CommitmentConfig,
    ) -> ClientResult<()> {
        impl_method!(
            [GetSignatureStatuses],
            poll_for_signature_with_commitment,
            self,
            signature,
            commitment_config,
        )
    }

    /// Poll the server to confirm a transaction.
    pub async fn poll_for_signature_confirmation(
        &self,
        signature: &Signature,
        min_confirmed_blocks: usize,
    ) -> ClientResult<usize> {
        impl_method!(
            [GetSignatureStatuses],
            poll_for_signature_confirmation,
            self,
            signature,
            min_confirmed_blocks,
        )
    }

    pub async fn get_num_blocks_since_signature_confirmation(
        &self,
        signature: &Signature,
    ) -> ClientResult<usize> {
        impl_method!(
            [GetSignatureStatuses],
            get_num_blocks_since_signature_confirmation,
            self,
            signature
        )
    }

    pub async fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        impl_method!([GetLatestBlockhash], get_latest_blockhash, self,)
    }

    pub async fn get_latest_blockhash_with_commitment(
        &self,
        commitment: CommitmentConfig,
    ) -> ClientResult<(Hash, u64)> {
        impl_method!(
            [GetLatestBlockhash],
            get_latest_blockhash_with_commitment,
            self,
            commitment
        )
    }

    #[allow(deprecated)]
    pub async fn is_blockhash_valid(
        &self,
        blockhash: &Hash,
        commitment: CommitmentConfig,
    ) -> ClientResult<bool> {
        impl_method!(
            [IsBlockhashValid],
            is_blockhash_valid,
            self,
            blockhash,
            commitment
        )
    }

    pub async fn get_fee_for_message(
        &self,
        message: &impl SerializableMessage,
    ) -> ClientResult<u64> {
        impl_method!([GetFeeForMessage], get_fee_for_message, self, message,)
    }

    pub async fn get_new_latest_blockhash(&self, blockhash: &Hash) -> ClientResult<Hash> {
        impl_method!(
            [GetLatestBlockhash],
            get_new_latest_blockhash,
            self,
            blockhash,
        )
    }

    pub async fn send<T>(&self, request: RpcRequest, params: Value) -> ClientResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let rpc = {
            let mut inner = self.inner.try_lock().unwrap();

            let mut index = inner.next_rpc_index;
            loop {
                let rpc = &mut inner.rpc_list[index];

                if rpc.execute_endpoints(&[request]) {
                    let rpc = rpc.rpc_client.clone();

                    if inner.move_rpc_index {
                        inner.next_rpc_index = index;
                    }

                    break rpc;
                }

                index = (index + 1) % inner.rpc_list.len();

                if index == inner.next_rpc_index {
                    break inner.default_rpc.clone();
                }
            }
        };

        rpc.send(request, params).await
    }
}
