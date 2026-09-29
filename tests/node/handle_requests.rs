#[cfg(test)]
mod handle_connection_tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::super::super::mock_stream::{request, send_data};
    use cleyto_coin::chain::utxo::TransactionInput;
    use cleyto_coin::chain::utxo::TransactionOutput;
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

    /// Builds a single, validly-signed transaction, mirroring the pattern
    /// used in chain::testing::test_chain().
    fn test_transaction() -> Transaction {
        let sender = Wallet::new();
        let receiver = Wallet::new();
        let inputs = vec![TransactionInput::new(1000, sender.0.clone())];
        let outputs = vec![TransactionOutput::new(1000, receiver.0.clone())];
        let mut transaction = Transaction::new(inputs, outputs).unwrap();
        sender
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction)
            .unwrap();

        transaction
    }

    /// Returns the transaction along with the sender wallet and the genesis
    /// UTXOs it spends, so the caller can seed a `Chain` that actually knows
    /// about them (`Wallet::available_utxos` is crate-private, so these can't
    /// be recovered from the wallet after the fact).
    fn test_transaction_and_wallets(
    ) -> (Transaction, Wallet, Vec<TransactionOutput>) {
        let mut sender = Wallet::new();
        let receiver = Wallet::new();
        let original_outputs =
            vec![TransactionOutput::new(1000, sender.0.clone())];
        sender.0.add_utxos(original_outputs.clone());
        let inputs = TransactionInput::from_outputs(original_outputs.clone());
        let outputs = vec![TransactionOutput::new(1000, receiver.0.clone())];
        let mut transaction = Transaction::new(inputs, outputs).unwrap();
        sender
            .1
            .sign_all_owned_inputs_in_transaction(&mut transaction)
            .unwrap();

        (transaction, sender.0, original_outputs)
    }

    fn test_state_wallets(
        first_receiver: Wallet,
        utxo_original: Vec<TransactionOutput>,
    ) -> Arc<Mutex<NodeState>> {
        Arc::new(Mutex::new(NodeState {
            status: true,
            chain: Chain::new(first_receiver, utxo_original),
            transactions_pool: Vec::new(),
            connected_nodes: HashSet::new(),
        }))
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
        let (tx, sender, original_outputs) = test_transaction_and_wallets();
        let message = Message::Transaction(tx);
        let body = serde_json::to_string(&message)
            .expect("failed to serialize transaction");
        let state = test_state_wallets(sender, original_outputs);
        let result = request("POST", "/messages", Some(&body), state).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
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
        assert!(
            result.is_err(),
            "expected Err for unknown path, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_get_only_path() {
        let result = request("POST", "/", Some("{}"), test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for disallowed method, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_post_only_path() {
        let result =
            request("GET", "/submit-transaction", None, test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for disallowed method, got {:?}",
            result
        );
    }

    // --- Malformed / missing content-length -------------------------------

    #[tokio::test]
    async fn test_post_missing_content_length_returns_err() {
        let raw = b"POST /messages HTTP/1.1\r\nhost: localhost\r\n\r\n{}";
        let result = send_data(raw, test_state()).await;
        assert!(
            result.is_err(),
            "expected Err due to MissingContentLength, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_post_invalid_content_length_returns_err() {
        let raw = b"POST /messages HTTP/1.1\r\nhost: localhost\r\ncontent-length: not-a-number\r\n\r\n{}";
        let result = send_data(raw, test_state()).await;
        assert!(
            result.is_err(),
            "expected Err due to invalid content-length, got {:?}",
            result
        );
    }

    // --- Malformed status line / headers -----------------------------------

    #[tokio::test]
    async fn test_empty_request_returns_err() {
        let result = send_data(b"", test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for empty request, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_incomplete_request_line_returns_err() {
        let raw = b"GET /\r\n\r\n";
        let result = send_data(raw, test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for incomplete request line, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_malformed_header_returns_err() {
        let raw = b"GET / HTTP/1.1\r\nnotaheader\r\n\r\n";
        let result = send_data(raw, test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for malformed header, got {:?}",
            result
        );
    }

    // --- Bad request bodies ------------------------------------------------

    #[tokio::test]
    async fn test_post_submit_transaction_invalid_body_returns_err() {
        let bad_body = "not a valid transaction";
        let result =
            request("POST", "/messages", Some(bad_body), test_state()).await;
        assert!(
            result.is_err(),
            "expected Err for malformed transaction body, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_post_submit_transaction_tampered_signature_returns_err() {
        // Valid structure but signature won't verify against a different
        // transaction_info — exercises the deeper business-logic validation
        // path (not just JSON parsing), assuming submit_transaction checks
        // signature validity before accepting into the pool.
        let mut tx = test_transaction();
        let bogus = Wallet::new();
        let mut bogus_info = Transaction::new(
            vec![TransactionInput::new(1, bogus.0.clone())],
            vec![TransactionOutput::new(1, bogus.0.clone())],
        )
        .unwrap();
        bogus
            .1
            .sign_all_owned_inputs_in_transaction(&mut bogus_info)
            .unwrap();

        let signature = bogus_info.inputs.first().unwrap().signature.clone();

        tx.inputs
            .iter_mut()
            .for_each(|i| i.signature = signature.clone());

        let message = Message::Transaction(tx);
        let body = serde_json::to_string(&message).unwrap();
        let result =
            request("POST", "/messages", Some(&body), test_state()).await;

        assert!(
            result.is_err(),
            "expected Err for tampered/invalid signature, got {:?}",
            result
        );
    }
}
