#[cfg(test)]
mod handle_connection_tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::super::super::mock_stream::{request, send_data};
    use cleyto_coin::chain::utxo::{TransactionInput, TransactionOutput};
    use cleyto_coin::chain::Chain;
    use cleyto_coin::node::Message;
    use cleyto_coin::{
        chain::{transaction::Transaction, wallet::Wallet},
        node::NodeState,
    };

    // --- Test helpers -------------------------------------------------

    fn test_state() -> Arc<Mutex<NodeState>> {
        Arc::new(Mutex::new(NodeState::default()))
    }

    /// A node state whose genesis gives 1000 to a sender, and a validly
    /// signed transaction sending those 1000 to a receiver.
    fn funded_state_and_transaction() -> (Arc<Mutex<NodeState>>, Transaction)
    {
        let (state, transaction, _) = funded_state_and_signer(1000);
        (state, transaction)
    }

    /// Same as above, sending `amount` (<= 1000), and also returns a
    /// function that builds more signed transactions spending the same coin
    fn funded_state_and_signer(
        amount: u64,
    ) -> (
        Arc<Mutex<NodeState>>,
        Transaction,
        impl Fn(u64) -> Transaction,
    ) {
        let sender = Wallet::new();
        let receiver = Wallet::new();
        let chain = Chain::new(vec![TransactionOutput::new(1000, &sender.0)]);
        let coins = chain.utxos_owned_by(&sender.0.to_public_key());

        let spend = move |amount: u64| {
            let outputs = vec![TransactionOutput::new(amount, &receiver.0)];
            let mut transaction =
                Transaction::new(TransactionInput::from_utxos(&coins), outputs)
                    .unwrap();
            sender
                .1
                .sign_all_owned_inputs_in_transaction(&mut transaction, &coins)
                .unwrap();
            transaction
        };

        let state = Arc::new(Mutex::new(NodeState {
            status: true,
            chain,
            transactions_pool: Vec::new(),
            connected_nodes: HashSet::new(),
        }));

        (state, spend(amount), spend)
    }

    /// Asserts the request failed and that its log message contains
    /// `expected`, so a test can't pass by failing for an unrelated reason
    fn assert_err_contains(
        result: &Result<Option<String>, Option<String>>,
        expected: &str,
    ) {
        match result {
            Err(Some(message)) => assert!(
                message.contains(expected),
                "expected error containing {expected:?}, got {message:?}"
            ),
            other => panic!(
                "expected error containing {expected:?}, got {other:?}"
            ),
        }
    }

    // --- Happy path: known endpoints -----------------------------------

    #[tokio::test]
    async fn test_get_index_success() {
        let result = request("GET", "/", None, test_state()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_get_status_success() {
        let result = request("GET", "/status", None, test_state()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_get_transaction_pool_success() {
        let result =
            request("GET", "/get-transaction-pool", None, test_state()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_favicon_success_and_suppressed_log() {
        let result = request("GET", "/favicon.ico", None, test_state()).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            None,
            "favicon should never produce a log message"
        );
    }

    #[tokio::test]
    async fn test_post_submit_transaction_success() {
        let (state, tx) = funded_state_and_transaction();
        let message = Message::Transaction(tx);
        let body = serde_json::to_string(&message)
            .expect("failed to serialize transaction");
        let result =
            request("POST", "/messages", Some(&body), state.clone()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert_eq!(state.lock().unwrap().transactions_pool.len(), 1);
    }

    #[tokio::test]
    async fn test_post_submit_transaction_unknown_utxo_returns_err() {
        // Valid transaction, but the node's chain doesn't have its coins
        let (_, tx) = funded_state_and_transaction();
        let body = serde_json::to_string(&Message::Transaction(tx)).unwrap();
        let result =
            request("POST", "/messages", Some(&body), test_state()).await;
        assert_err_contains(&result, "doesn't exist or was already spent");
    }

    #[tokio::test]
    async fn test_post_submit_double_spend_returns_err() {
        // A second, validly signed, transaction spending the same coin as
        // one already in the pool
        let (state, tx, spend) = funded_state_and_signer(1000);
        let double_spend = spend(999);

        let body = serde_json::to_string(&Message::Transaction(tx)).unwrap();
        request("POST", "/messages", Some(&body), state.clone())
            .await
            .unwrap();

        let body =
            serde_json::to_string(&Message::Transaction(double_spend)).unwrap();
        let result =
            request("POST", "/messages", Some(&body), state.clone()).await;
        assert_err_contains(&result, "already spent by a transaction in the pool");
        assert_eq!(state.lock().unwrap().transactions_pool.len(), 1);
    }

    #[ignore = "KeyRefresh not implemented"]
    #[tokio::test]
    async fn test_post_messages_success() {
        // TODO: replace with real message payload shape if not this simple.
        let body =
            serde_json::to_string(&cleyto_coin::node::Message::KeyRefresh)
                .unwrap();
        let result =
            request("POST", "/messages", Some(&body), test_state()).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    // --- Routing failures ------------------------------------------------

    #[tokio::test]
    async fn test_unknown_path_returns_err() {
        let result =
            request("GET", "/does-not-exist", None, test_state()).await;
        assert_err_contains(&result, "Path /does-not-exist was not found");
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_get_only_path() {
        let result = request("POST", "/", Some("{}"), test_state()).await;
        assert_err_contains(&result, "with wrong method");
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_post_only_path() {
        // /messages only accepts POST
        let result = request("GET", "/messages", None, test_state()).await;
        assert_err_contains(&result, "with wrong method");
    }

    // --- Malformed / missing content-length -------------------------------

    #[tokio::test]
    async fn test_post_missing_content_length_returns_err() {
        let raw = b"POST /messages HTTP/1.1\r\nhost: localhost\r\n\r\n{}";
        let result = send_data(raw, test_state()).await;
        assert_err_contains(&result, "Missing content-length");
    }

    #[tokio::test]
    async fn test_post_invalid_content_length_returns_err() {
        let raw = b"POST /messages HTTP/1.1\r\nhost: localhost\r\ncontent-length: not-a-number\r\n\r\n{}";
        let result = send_data(raw, test_state()).await;
        // An unparseable content-length is reported as a missing one
        assert_err_contains(&result, "Missing content-length");
    }

    // --- Malformed status line / headers -----------------------------------

    #[tokio::test]
    async fn test_empty_request_returns_err() {
        let result = send_data(b"", test_state()).await;
        assert_err_contains(&result, "Invalid status line");
    }

    #[tokio::test]
    async fn test_incomplete_request_line_returns_err() {
        let raw = b"GET /\r\n\r\n";
        let result = send_data(raw, test_state()).await;
        assert_err_contains(&result, "Invalid request line");
    }

    #[tokio::test]
    async fn test_malformed_header_returns_err() {
        let raw = b"GET / HTTP/1.1\r\nnotaheader\r\n\r\n";
        let result = send_data(raw, test_state()).await;
        assert_err_contains(&result, "Invalid request line");
    }

    // --- Bad request bodies ------------------------------------------------

    #[tokio::test]
    async fn test_post_submit_transaction_invalid_body_returns_err() {
        let bad_body = "not a valid transaction";
        let result =
            request("POST", "/messages", Some(bad_body), test_state()).await;
        // A body that isn't a Message is rejected (InvalidBody) without a log
        assert_eq!(result, Err(None));
    }

    #[tokio::test]
    async fn test_post_submit_transaction_tampered_signature_returns_err() {
        // Valid structure but signature won't verify against a different
        // transaction_info — exercises the deeper business-logic validation
        // path (not just JSON parsing), assuming submit_transaction checks
        // signature validity before accepting into the pool.
        let (state, mut tx) = funded_state_and_transaction();
        // Transaction::default is signed by its own random wallet, so its
        // signature is a valid signature from someone who isn't the owner
        let bogus = Transaction::default();
        let signature = bogus.inputs.first().unwrap().signature.clone();

        tx.inputs
            .iter_mut()
            .for_each(|i| i.signature = signature.clone());

        let message = Message::Transaction(tx);
        let body = serde_json::to_string(&message).unwrap();
        let result =
            request("POST", "/messages", Some(&body), state.clone()).await;

        assert_err_contains(&result, "signature did not match");
        assert!(state.lock().unwrap().transactions_pool.is_empty());
    }
}
