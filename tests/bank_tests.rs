use npwd_rs::prelude::*;

#[test]
fn test_bank_transfers_and_insufficient_funds() {
    let mut bank = BankApp::new("US-MAZE-1122", 100_000); // $1,000.00
    assert_eq!(bank.balance(), 100_000);

    // Transfer $350.00 (35,000 cents)
    let tx = bank
        .transfer("US-MAZE-9988", 35_000, "Split rent payment")
        .unwrap();
    assert_eq!(tx.tx_type, TransactionType::TransferOut);
    assert_eq!(tx.amount, 35_000);
    assert_eq!(bank.balance(), 65_000);

    // Insufficient funds attempt: try transferring $1,000.00 when only $650.00 remaining
    let fail_tx = bank.transfer("US-MAZE-9988", 100_000, "Too much money");
    assert!(matches!(
        fail_tx,
        Err(NpwdError::InsufficientFunds {
            balance: 65_000,
            required: 100_000
        })
    ));
}

#[test]
fn test_salary_deposit_game_event() {
    let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();

    let initial_balance = {
        let bank = phone.app_registry.get_concrete::<BankApp>("bank").unwrap();
        bank.balance()
    };

    phone
        .handle_game_event(GameEvent::SalaryPaid {
            amount: 250_000, // $2,500.00
            employer: "Los Santos Customs".to_string(),
        })
        .unwrap();

    let bank = phone.app_registry.get_concrete::<BankApp>("bank").unwrap();
    assert_eq!(bank.balance(), initial_balance + 250_000);

    let banners = phone.event_bus.active_banners();
    assert!(banners.iter().any(|b| b.title == "Direct Deposit Received"));
}

#[test]
fn test_invoice_creation_and_payment() {
    let mut bank = BankApp::new("US-MAZE-1122", 50_000); // $500.00

    let invoice = Invoice {
        id: uuid::Uuid::new_v4(),
        issuer_account: "US-HOSPITAL-01".to_string(),
        recipient_account: "US-MAZE-1122".to_string(),
        amount: 15_000, // $150.00
        description: "Medical Treatment ER".to_string(),
        created_at: chrono::Utc::now(),
        status: InvoiceStatus::Pending,
    };

    let invoice_id = invoice.id;
    bank.receive_invoice(invoice);
    assert_eq!(bank.pending_invoices().len(), 1);
    assert_eq!(bank.metadata().badge_count, 1);

    // Pay invoice
    let tx = bank.pay_invoice(invoice_id).unwrap();
    assert_eq!(tx.tx_type, TransactionType::InvoicePayment);
    assert_eq!(bank.balance(), 35_000);
    assert_eq!(bank.pending_invoices().len(), 0);
    assert_eq!(bank.metadata().badge_count, 0);
}
