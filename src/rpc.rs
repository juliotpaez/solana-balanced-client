use crate::SolanaClientRateLimit;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use std::sync::Arc;

/// A single Solana RPC client. Includes its limits to calculate usage.
pub struct SolanaClientRpc {
    pub rpc_client: Arc<RpcClient>,

    /// The list of all limits applied to the RPC client.
    /// All limits must succeed in order to send a request.
    pub limits: Vec<SolanaClientRateLimit>,
}

impl SolanaClientRpc {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Creates a `SolanaClientRpc` without limits.
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self {
            rpc_client,
            limits: Vec::new(),
        }
    }

    // METHODS ----------------------------------------------------------------

    /// Adds a [`SolanaClientRateLimit`] to the list of limits.
    pub fn add_limit(mut self, limit: SolanaClientRateLimit) -> Self {
        self.limits.push(limit);
        self
    }

    /// Process the execution of a Solana RPC endpoint.
    pub fn execute_endpoint(&mut self, endpoint: RpcRequest) -> bool {
        for limit in &mut self.limits {
            if !limit.check_endpoint(endpoint) {
                return false;
            }
        }

        for limit in &mut self.limits {
            limit.apply_endpoint(endpoint);
        }

        true
    }

    /// Process the execution of a Solana RPC endpoints.
    pub fn execute_endpoints(&mut self, endpoints: &[RpcRequest]) -> bool {
        for limit in &mut self.limits {
            if !limit.check_endpoints(endpoints) {
                return false;
            }
        }

        for limit in &mut self.limits {
            limit.apply_endpoints(endpoints);
        }

        true
    }
}
