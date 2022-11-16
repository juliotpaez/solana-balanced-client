use solana_balanced_client::{SolanaClient, SolanaClientRateLimit, SolanaClientRpc};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use std::sync::Arc;

/// Example of creating a client with the GenesysGo's free-tier configuration.
///
/// Info at:
/// - https://docs.genesysgo.com/shadow/rpc-subscribers/our-offerings
#[tokio::main]
async fn main() {
    let default_rpc = Arc::new(RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));
    let genesysgo_rpc = Arc::new(RpcClient::new("<your-genesysgo-rpc-url>".to_string()));
    let client = SolanaClient::new_with_default(default_rpc).add_rpc(
        SolanaClientRpc::new(genesysgo_rpc)
            // Requests per second limit.
            .add_limit(
                SolanaClientRateLimit::new(
                    1000, /* 1 second */
                    1,    /* 1 request per second */
                    1,    /* all requests count the same */
                )
                // Ignore GetMultipleAccounts requests here to track them in the next limit.
                .ignore_endpoint(RpcRequest::GetMultipleAccounts),
            )
            // Requests per second limit for GetMultipleAccounts.
            .add_limit(
                SolanaClientRateLimit::new(
                    60 * 1000, /* 1 minute */
                    6,         /* 6 request per minute */
                    1,         /* all requests count the same */
                )
                // Ignore all endpoints and include only GetMultipleAccounts requests here.
                .ignores_all_endpoints()
                .add_endpoint_amount(RpcRequest::GetMultipleAccounts, 1),
            ),
    );

    let version = client.get_version().await.unwrap();

    println!("Cluster version: {}", version.solana_core);
}
