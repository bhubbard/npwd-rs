use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, PhoneApp};
use crate::error::{NpwdError, Result};
use crate::events::{GameEvent, Notification, PhoneEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// Category of financial transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    TransferIn,
    TransferOut,
    InvoicePayment,
    Salary,
}

/// Ledger transaction record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub tx_type: TransactionType,
    /// Amount in cents.
    pub amount: i64,
    pub timestamp: DateTime<Utc>,
    pub counterpart_account: Option<String>,
    pub description: String,
    pub balance_after: i64,
}

/// Status of a billing invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Cancelled,
}

/// In-game billing invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invoice {
    pub id: Uuid,
    pub issuer_account: String,
    pub recipient_account: String,
    pub amount: i64,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub status: InvoiceStatus,
}

/// Banking & Economy smartphone application.
#[derive(Debug, Clone)]
pub struct BankApp {
    metadata: AppMetadata,
    account_number: String,
    balance: i64,
    transactions: Vec<Transaction>,
    invoices: HashMap<Uuid, Invoice>,
}

impl BankApp {
    pub fn new(account_number: impl Into<String>, initial_balance: i64) -> Self {
        let metadata = AppMetadata::new("bank", "Maze Bank", "bank-icon")
            .with_permission(AppPermission::Banking)
            .system();

        Self {
            metadata,
            account_number: account_number.into(),
            balance: initial_balance,
            transactions: Vec::new(),
            invoices: HashMap::new(),
        }
    }

    /// Current balance in cents.
    pub fn balance(&self) -> i64 {
        self.balance
    }

    /// Account number identifier.
    pub fn account_number(&self) -> &str {
        &self.account_number
    }

    /// Direct wire transfer to another account.
    pub fn transfer(&mut self, to_account: &str, amount: i64, memo: &str) -> Result<Transaction> {
        if amount <= 0 {
            return Err(NpwdError::Internal("Transfer amount must be positive".to_string()));
        }
        if self.balance < amount {
            return Err(NpwdError::InsufficientFunds {
                balance: self.balance,
                required: amount,
            });
        }

        self.balance -= amount;
        let tx = Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::TransferOut,
            amount,
            timestamp: Utc::now(),
            counterpart_account: Some(to_account.to_string()),
            description: if memo.is_empty() {
                format!("Transfer to {}", to_account)
            } else {
                memo.to_string()
            },
            balance_after: self.balance,
        };

        self.transactions.push(tx.clone());
        Ok(tx)
    }

    /// Deposit incoming funds from another player or system transfer.
    pub fn receive_transfer(&mut self, from_account: &str, amount: i64, memo: &str) -> Result<Transaction> {
        if amount <= 0 {
            return Err(NpwdError::Internal("Received amount must be positive".to_string()));
        }

        self.balance += amount;
        let tx = Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::TransferIn,
            amount,
            timestamp: Utc::now(),
            counterpart_account: Some(from_account.to_string()),
            description: if memo.is_empty() {
                format!("Transfer from {}", from_account)
            } else {
                memo.to_string()
            },
            balance_after: self.balance,
        };

        self.transactions.push(tx.clone());
        Ok(tx)
    }

    /// Deposit paycheck / salary into account.
    pub fn deposit_salary(&mut self, employer: &str, amount: i64) -> Result<Transaction> {
        if amount <= 0 {
            return Err(NpwdError::Internal("Salary amount must be positive".to_string()));
        }

        self.balance += amount;
        let tx = Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::Salary,
            amount,
            timestamp: Utc::now(),
            counterpart_account: Some(employer.to_string()),
            description: format!("Paycheck from {}", employer),
            balance_after: self.balance,
        };

        self.transactions.push(tx.clone());
        Ok(tx)
    }

    /// Create an invoice billing another player.
    pub fn create_invoice(&mut self, to_account: &str, amount: i64, description: &str) -> Result<Invoice> {
        if amount <= 0 {
            return Err(NpwdError::Internal("Invoice amount must be positive".to_string()));
        }

        let invoice = Invoice {
            id: Uuid::new_v4(),
            issuer_account: self.account_number.clone(),
            recipient_account: to_account.to_string(),
            amount,
            description: description.to_string(),
            created_at: Utc::now(),
            status: InvoiceStatus::Pending,
        };

        self.invoices.insert(invoice.id, invoice.clone());
        Ok(invoice)
    }

    /// Register an inbound invoice to be paid.
    pub fn receive_invoice(&mut self, invoice: Invoice) {
        self.invoices.insert(invoice.id, invoice);
        self.sync_badge_count();
    }

    /// Pay an invoice by ID.
    pub fn pay_invoice(&mut self, invoice_id: Uuid) -> Result<Transaction> {
        let invoice = self
            .invoices
            .get_mut(&invoice_id)
            .ok_or_else(|| NpwdError::Internal(format!("Invoice {} not found", invoice_id)))?;

        if invoice.status != InvoiceStatus::Pending {
            return Err(NpwdError::Internal("Invoice is not pending".to_string()));
        }

        if self.balance < invoice.amount {
            return Err(NpwdError::InsufficientFunds {
                balance: self.balance,
                required: invoice.amount,
            });
        }

        self.balance -= invoice.amount;
        invoice.status = InvoiceStatus::Paid;

        let tx = Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::InvoicePayment,
            amount: invoice.amount,
            timestamp: Utc::now(),
            counterpart_account: Some(invoice.issuer_account.clone()),
            description: format!("Paid Invoice: {}", invoice.description),
            balance_after: self.balance,
        };

        self.transactions.push(tx.clone());
        self.sync_badge_count();
        Ok(tx)
    }

    /// Transaction history ordered most recent first.
    pub fn transactions(&self) -> Vec<&Transaction> {
        let mut list: Vec<&Transaction> = self.transactions.iter().collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        list
    }

    /// Invoices list.
    pub fn invoices(&self) -> Vec<&Invoice> {
        let mut list: Vec<&Invoice> = self.invoices.values().collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        list
    }

    /// Pending unpaid invoices.
    pub fn pending_invoices(&self) -> Vec<&Invoice> {
        self.invoices
            .values()
            .filter(|i| i.status == InvoiceStatus::Pending)
            .collect()
    }

    fn sync_badge_count(&mut self) {
        self.metadata.badge_count = self.pending_invoices().len() as u32;
    }
}

impl Default for BankApp {
    fn default() -> Self {
        Self::new("US-MAZE-004289", 500_000) // $5,000.00 default
    }
}

impl PhoneApp for BankApp {
    fn metadata(&self) -> &AppMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut AppMetadata {
        &mut self.metadata
    }

    fn on_mount(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.sync_badge_count();
        Ok(())
    }

    fn on_pause(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_resume(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.sync_badge_count();
        Ok(())
    }

    fn on_close(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppInputEvent,
        ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        match event {
            AppInputEvent::GameBridge(GameEvent::SalaryPaid { amount, employer }) => {
                let tx = self.deposit_salary(employer, *amount)?;
                ctx.notify(Notification::new(
                    "bank",
                    "Direct Deposit Received",
                    format!("+${:.2} from {}", (*amount as f64) / 100.0, employer),
                ));
                Ok(Some(AppOutputAction::Custom(serde_json::to_value(&tx).unwrap())))
            }
            AppInputEvent::UiAction { action, payload } if action == "transfer" => {
                let to = payload["to"].as_str().unwrap_or_default();
                let amount = payload["amount"].as_i64().unwrap_or(0);
                let memo = payload["memo"].as_str().unwrap_or_default();

                let tx = self.transfer(to, amount, memo)?;
                ctx.emit_phone_event(PhoneEvent::SendMoney {
                    to_account: to.to_string(),
                    amount,
                    memo: memo.to_string(),
                });
                Ok(Some(AppOutputAction::Custom(serde_json::to_value(&tx).unwrap())))
            }
            _ => Ok(None),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
