//! Benchmark comparing `npwd-rs` (Rust) vs original TypeScript/React FiveM NPWD (CEF/NUI).

use npwd_rs::apps::bank::BankApp;
use npwd_rs::prelude::*;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("     npwd-rs (Rust) vs TypeScript/React NPWD (CEF / NUI)    ");
    println!("============================================================");

    // 1. Phone OS Heartbeat Tick (Battery, Call state, Event bus)
    println!("\n--- 1. Phone OS Heartbeat Tick (Clock, Battery, Subsystems) ---");
    {
        let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();
        let iterations = 5_000_000;
        let dt = 0.016;
        let start = Instant::now();

        for _ in 0..iterations {
            phone.tick(dt);
        }

        let elapsed = start.elapsed();
        let ns_per_tick = elapsed.as_nanos() as f64 / iterations as f64;
        let ticks_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "OS Ticks: {} | Time: {:.2?} | Latency: {:.2} ns/tick | {:>10.0} ticks/s | Battery: {:.1}%",
            iterations, elapsed, ns_per_tick, ticks_per_sec, phone.battery.percentage()
        );
    }

    // 2. App Lifecycle & Multitasking Context Switches
    println!("\n--- 2. App Lifecycle & Multitasking Context Switches ---");
    {
        let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();
        let iterations = 1_000_000;
        let start = Instant::now();
        let apps = ["contacts", "messages", "bank", "camera", "marketplace", "settings"];

        for i in 0..iterations {
            let app_id = apps[i % apps.len()];
            let _ = phone.launch_app(app_id);
            if i % 3 == 0 {
                let _ = phone.press_home_button();
            }
        }

        let elapsed = start.elapsed();
        let ns_per_switch = elapsed.as_nanos() as f64 / iterations as f64;
        let switches_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "App Transitions: {} | Time: {:.2?} | Latency: {:.2} ns/switch | {:>10.0} switches/s",
            iterations, elapsed, ns_per_switch, switches_per_sec
        );
    }

    // 3. Maze Bank In-Memory Transaction Engine
    println!("\n--- 3. Maze Bank In-Memory Transaction Engine ---");
    {
        let mut bank = BankApp::new("US-MAZE-1122", 50_000_000); // $500,000.00
        let iterations = 1_000_000;
        let start = Instant::now();
        let mut successful_txs = 0;

        for i in 0..iterations {
            let amount = 1000 + (i % 500) as i64;
            if i % 2 == 0 {
                if bank.deposit_salary("Los Santos Customs", amount).is_ok() {
                    successful_txs += 1;
                }
            } else {
                if bank.transfer("US-MAZE-9988", amount, "Split rent payment").is_ok() {
                    successful_txs += 1;
                }
            }
        }

        std::hint::black_box(successful_txs);
        let elapsed = start.elapsed();
        let ns_per_tx = elapsed.as_nanos() as f64 / iterations as f64;
        let txs_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Bank Transactions: {} | Time: {:.2?} | Latency: {:.2} ns/tx | {:>10.0} tx/s | Balance: ${:.2}",
            iterations, elapsed, ns_per_tx, txs_per_sec, (bank.balance() as f64) / 100.0
        );
    }

    // 4. Notification & Event Bus Dispatch
    println!("\n--- 4. Event Bus Inbound/Outbound Message Routing ---");
    {
        let mut bus = EventBus::new();
        let iterations = 2_000_000;
        let start = Instant::now();
        let mut routed_count = 0;

        for i in 0..iterations {
            bus.push_incoming(GameEvent::CellularSignalChanged {
                bars: (i % 5) as u8 + 1,
            });

            if let Some(event) = bus.pop_incoming() {
                if let GameEvent::CellularSignalChanged { bars } = event {
                    routed_count += bars as usize;
                }
            }
        }

        std::hint::black_box(routed_count);
        let elapsed = start.elapsed();
        let ns_per_event = elapsed.as_nanos() as f64 / iterations as f64;
        let events_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Events Routed: {} | Time: {:.2?} | Latency: {:.2} ns/event | {:>10.0} events/s",
            iterations, elapsed, ns_per_event, events_per_sec
        );
    }

    println!("\n============================================================");
    println!("                      Benchmark Complete                    ");
    println!("============================================================");
}
