use chrono::Utc;
use solana_client::rpc_request::RpcRequest;
use std::collections::{HashMap, HashSet};

const ALL_ENDPOINTS: [RpcRequest; 57] = [
    RpcRequest::DeregisterNode,
    RpcRequest::GetAccountInfo,
    RpcRequest::GetBalance,
    RpcRequest::GetBlock,
    RpcRequest::GetBlockHeight,
    RpcRequest::GetBlockProduction,
    RpcRequest::GetBlocks,
    RpcRequest::GetBlocksWithLimit,
    RpcRequest::GetBlockTime,
    RpcRequest::GetClusterNodes,
    RpcRequest::GetEpochInfo,
    RpcRequest::GetEpochSchedule,
    RpcRequest::GetFeeForMessage,
    RpcRequest::GetFirstAvailableBlock,
    RpcRequest::GetGenesisHash,
    RpcRequest::GetHealth,
    RpcRequest::GetIdentity,
    RpcRequest::GetInflationGovernor,
    RpcRequest::GetInflationRate,
    RpcRequest::GetInflationReward,
    RpcRequest::GetLargestAccounts,
    RpcRequest::GetLatestBlockhash,
    RpcRequest::GetLeaderSchedule,
    RpcRequest::GetMaxRetransmitSlot,
    RpcRequest::GetMaxShredInsertSlot,
    RpcRequest::GetMinimumBalanceForRentExemption,
    RpcRequest::GetMultipleAccounts,
    RpcRequest::GetProgramAccounts,
    RpcRequest::GetRecentPerformanceSamples,
    RpcRequest::GetHighestSnapshotSlot,
    RpcRequest::GetSignaturesForAddress,
    RpcRequest::GetSignatureStatuses,
    RpcRequest::GetSlot,
    RpcRequest::GetSlotLeader,
    RpcRequest::GetSlotLeaders,
    RpcRequest::GetStorageTurn,
    RpcRequest::GetStorageTurnRate,
    RpcRequest::GetSlotsPerSegment,
    RpcRequest::GetStakeActivation,
    RpcRequest::GetStakeMinimumDelegation,
    RpcRequest::GetStoragePubkeysForSlot,
    RpcRequest::GetSupply,
    RpcRequest::GetTokenAccountBalance,
    RpcRequest::GetTokenAccountsByDelegate,
    RpcRequest::GetTokenAccountsByOwner,
    RpcRequest::GetTokenSupply,
    RpcRequest::GetTransaction,
    RpcRequest::GetTransactionCount,
    RpcRequest::GetVersion,
    RpcRequest::GetVoteAccounts,
    RpcRequest::IsBlockhashValid,
    RpcRequest::MinimumLedgerSlot,
    RpcRequest::RegisterNode,
    RpcRequest::RequestAirdrop,
    RpcRequest::SendTransaction,
    RpcRequest::SimulateTransaction,
    RpcRequest::SignVote,
];

/// Limit for rates like requests per second or compute units per interval.
#[derive(Debug)]
pub struct SolanaClientRateLimit {
    /// Map that contains the amount given to each endpoint.
    pub endpoint_amounts: HashMap<RpcRequest, u64>,

    /// Set of endpoints ignored by this limit.
    pub ignore_endpoints: HashSet<RpcRequest>,

    /// The last date this limit was updated.
    pub last_datetime: i64,

    /// Interval between rate resets expressed in milliseconds.
    pub interval: i64,

    /// The number of requests or compute units already consumed in the current interval.
    pub consumed_amount: u64,

    /// The maximum number of requests or compute units allowed per interval.
    pub maximum_amount: u64,

    /// The default amount for each endpoint if not specified in `endpoint_amounts`.
    pub default_amount_per_endpoint: u64,
}

impl SolanaClientRateLimit {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(interval: u64, maximum_amount: u64, default_amount_per_endpoint: u64) -> Self {
        Self {
            endpoint_amounts: HashMap::new(),
            ignore_endpoints: HashSet::new(),
            last_datetime: 0,
            interval: interval as i64,
            consumed_amount: 0,
            maximum_amount,
            default_amount_per_endpoint,
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a [`RpcRequest`] to the map of endpoints to their amounts.
    pub fn add_endpoint_amount(mut self, endpoint: RpcRequest, amount: u64) -> Self {
        self.endpoint_amounts.insert(endpoint, amount);
        self.ignore_endpoints.remove(&endpoint);
        self
    }

    /// Adds a [`RpcRequest`] to the list of ignored endpoints.
    pub fn ignore_endpoint(mut self, endpoint: RpcRequest) -> Self {
        self.ignore_endpoints.insert(endpoint);
        self.endpoint_amounts.remove(&endpoint);
        self
    }

    /// Adds all [`RpcRequest`] to the list of ignored endpoints.
    pub fn ignores_all_endpoints(mut self) -> Self {
        if self.ignore_endpoints.len() < ALL_ENDPOINTS.len() {
            self.ignore_endpoints
                .reserve(ALL_ENDPOINTS.len() - self.ignore_endpoints.len());
        }

        for endpoint in ALL_ENDPOINTS {
            self.ignore_endpoints.insert(endpoint);
            self.endpoint_amounts.remove(&endpoint);
        }

        self
    }

    /// Checks whether the endpoint can be executed according to the current limits.
    pub fn check_endpoint(&mut self, endpoint: RpcRequest) -> bool {
        self.check_endpoints(&[endpoint])
    }

    /// Checks whether the endpoints can be executed according to the current limits.
    pub fn check_endpoints(&mut self, endpoints: &[RpcRequest]) -> bool {
        let now = Utc::now().timestamp_millis() / self.interval;

        // Reset if needed.
        if now != self.last_datetime {
            self.last_datetime = now;
            self.consumed_amount = 0;
        }

        let mut consumed_amount = self.consumed_amount;

        for endpoint in endpoints {
            if self.ignore_endpoints.contains(endpoint) {
                continue;
            }

            let amount = *self
                .endpoint_amounts
                .get(endpoint)
                .unwrap_or(&self.default_amount_per_endpoint);

            consumed_amount += amount;

            if consumed_amount > self.maximum_amount {
                return false;
            }
        }

        true
    }

    /// Applies the changes of executing the endpoint.
    pub fn apply_endpoint(&mut self, endpoint: RpcRequest) {
        self.apply_endpoints(&[endpoint]);
    }

    /// Applies the changes of executing the endpoints.
    pub fn apply_endpoints(&mut self, endpoints: &[RpcRequest]) {
        for endpoint in endpoints {
            if self.ignore_endpoints.contains(endpoint) {
                continue;
            }

            let amount = *self
                .endpoint_amounts
                .get(endpoint)
                .unwrap_or(&self.default_amount_per_endpoint);
            self.consumed_amount += amount;
        }
    }
}
