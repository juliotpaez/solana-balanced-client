use solana_balanced_client::{SolanaClient, SolanaClientRateLimit, SolanaClientRpc};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use std::sync::Arc;

/// Example of creating a client with the QuickNode's free-tier configuration.
///
/// Info at:
/// - https://www.quicknode.com/pricing
/// - https://www.quicknode.com/api-credits/sol
#[tokio::main]
async fn main() {
    let default_rpc = Arc::new(RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));
    let quicknode_rpc = Arc::new(RpcClient::new("<your-quicknode-rpc-url>".to_string()));
    let client = SolanaClient::new_with_default(default_rpc).add_rpc(
        SolanaClientRpc::new(quicknode_rpc)
            // Credits / month limits.
            .add_limit(
                SolanaClientRateLimit::new(
                    30 * 24 * 60 * 60 * 1000, /* 30 days */
                    10_000_000,               /* credits per month */
                    1,                        /* default cost in credits for endpoints */
                )
                // List of all endpoints whose credits are different than the default value.
                .add_endpoint_amount(RpcRequest::GetAccountInfo, 2)
                .add_endpoint_amount(RpcRequest::GetBlockTime, 2)
                .add_endpoint_amount(RpcRequest::GetClusterNodes, 2)
                .add_endpoint_amount(RpcRequest::GetBlock, 23)
                .add_endpoint_amount(RpcRequest::GetEpochInfo, 2)
                .add_endpoint_amount(RpcRequest::GetFirstAvailableBlock, 3)
                .add_endpoint_amount(RpcRequest::GetHealth, 2)
                .add_endpoint_amount(RpcRequest::GetHighestSnapshotSlot, 2)
                .add_endpoint_amount(RpcRequest::GetInflationGovernor, 2)
                .add_endpoint_amount(RpcRequest::GetLatestBlockhash, 2)
                .add_endpoint_amount(RpcRequest::GetMinimumBalanceForRentExemption, 3)
                .add_endpoint_amount(RpcRequest::GetProgramAccounts, 35)
                .add_endpoint_amount(RpcRequest::GetRecentPerformanceSamples, 4)
                .add_endpoint_amount(RpcRequest::GetSignaturesForAddress, 3)
                .add_endpoint_amount(RpcRequest::GetTokenSupply, 2)
                .add_endpoint_amount(RpcRequest::GetTransaction, 3)
                .add_endpoint_amount(RpcRequest::GetVersion, 2)
                .add_endpoint_amount(RpcRequest::SimulateTransaction, 4)
                .add_endpoint_amount(RpcRequest::GetMultipleAccounts, 10)
                .add_endpoint_amount(RpcRequest::GetLargestAccounts, 259),
            )
            // Requests / second limit.
            .add_limit(SolanaClientRateLimit::new(
                1000, /* 1 second */
                25,   /* 25 requests per second */
                1,    /* all requests count the same */
            )),
    );

    let version = client.get_version().await.unwrap();

    println!("Cluster version: {}", version.solana_core);
}
