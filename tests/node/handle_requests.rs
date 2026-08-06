#[cfg(test)]
mod handle_connection_tests {
    use super::super::send_data;
    use cleyto_coin::{
        chain::{
            transaction::{Transaction, TransactionInfo},
            utxo::UTXO,
            wallet::Wallet,
        },
        node::NodeState,
    };

    // --- Test helpers -------------------------------------------------

    fn test_state() -> NodeState {
        NodeState::default()
    }

    /// Builds a single, validly-signed transaction, mirroring the pattern
    /// used in chain::testing::test_chain().
    fn test_transaction() -> Transaction {
        let sender = Wallet::new();
        let receiver = Wallet::new();
        let inputs = vec![UTXO::new(1000, sender.0.clone())];
        let outputs = vec![UTXO::new(1000, receiver.0.clone())];
        let info = TransactionInfo::new(inputs, outputs);
        let signature = sender.1.sign_transaction(&info).unwrap();
        Transaction::new(sender.0.clone(), receiver.0.clone(), info, signature)
            .unwrap()
    }

    fn raw_request(method: &str, path: &str, body: Option<&str>) -> Vec<u8> {
        match body {
            Some(b) => format!(
                "{method} {path} HTTP/1.1\r\nhost: localhost\r\ncontent-length: {}\r\n\r\n{}",
                b.len(),
                b
            )
            .into_bytes(),
            None => format!("{method} {path} HTTP/1.1\r\nhost: localhost\r\n\r\n").into_bytes(),
        }
    }

    async fn request(
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<Option<String>, Option<String>> {
        send_data(&raw_request(method, path, body), test_state()).await
    }

    // --- Happy path: known endpoints -----------------------------------

    #[tokio::test]
    async fn test_get_index_success() {
        let result = request("GET", "/", None).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_get_status_success() {
        let result = request("GET", "/status", None).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_get_transaction_pool_success() {
        let result = request("GET", "/get-transaction-pool", None).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_favicon_success_and_suppressed_log() {
        let result = request("GET", "/favicon.ico", None).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            None,
            "favicon should never produce a log message"
        );
    }

    #[tokio::test]
    async fn test_post_submit_transaction_success() {
        let tx = test_transaction();
        let body = serde_json::to_string(&tx)
            .expect("failed to serialize transaction");
        let result = request("POST", "/submit-transaction", Some(&body)).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn test_post_messages_success() {
        // TODO: replace with real message payload shape if not this simple.
        let body =
            serde_json::to_string(&cleyto_coin::node::Message::KeyRefresh)
                .unwrap();
        let result = request("POST", "/messages", Some(&body)).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    // --- Routing failures ------------------------------------------------

    #[tokio::test]
    async fn test_unknown_path_returns_err() {
        let result = request("GET", "/does-not-exist", None).await;
        assert!(
            result.is_err(),
            "expected Err for unknown path, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_get_only_path() {
        let result = request("POST", "/", Some("{}")).await;
        assert!(
            result.is_err(),
            "expected Err for disallowed method, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_method_not_allowed_on_post_only_path() {
        let result = request("GET", "/submit-transaction", None).await;
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
            request("POST", "/submit-transaction", Some(bad_body)).await;
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
        let bogus_info = TransactionInfo::new(
            vec![UTXO::new(1, bogus.0.clone())],
            vec![UTXO::new(1, bogus.0.clone())],
        );
        tx.signature = bogus.1.sign_transaction(&bogus_info).unwrap();
        let body = serde_json::to_string(&tx).unwrap();
        let result = request("POST", "/submit-transaction", Some(&body)).await;
        assert!(
            result.is_err(),
            "expected Err for tampered/invalid signature, got {:?}",
            result
        );
    }
}
