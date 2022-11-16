# Solana Balanced Client

This is a wrapper over the `RpcClient` of solana-client crate. It provides the ability to balance the requests between
multiple RPC endpoints taking into account their limits.

## Usage

```rust
fn main() {
    let default_rpc = Arc::new(RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));
    let your_rpc = Arc::new(RpcClient::new("<your-rpc-url>".to_string()));
    let client = SolanaClient::new_with_default(your_rpc).add_rpc(
        SolanaClientRpc::new(quicknode_rpc)
            // Credits / month limits.
            .add_limit(
                SolanaClientRateLimit::new(
                    30 * 24 * 60 * 60 * 1000, /* 30 days */
                    1_000,                    /* credits per month */
                    1,                        /* default cost in credits for endpoints */
                )
                    // Ignore those endpoints that should be handled in this limit.
                    .ignore_endpoint(RpcRequest::GetHealth)
                    // List of all endpoints whose credits are different than the default value.
                    .add_endpoint_amount(RpcRequest::GetAccountInfo, 2)
                    .add_endpoint_amount(RpcRequest::GetMultipleAccounts, 10)
            )
            // Requests / second limit.
            .add_limit(SolanaClientRateLimit::new(
                1000, /* 1 second */
                10,   /* 10 requests per second */
                1,    /* all requests count the same */
            )),
    );
    
    // ...
}
```

You can see more examples in the [`examples`](examples) directory.

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.