extern crate actix;
extern crate actix_web;
extern crate althea_types;
extern crate bytes;
extern crate clarity;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate guac_core;

extern crate num256;
extern crate qutex;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate web3;

mod blockchain_client;
mod config;
mod counterparty_client;
mod counterparty_server;

use actix::System;
use blockchain_client::BlockchainClient;
use clarity::utils::hex_str_to_bytes;
use clarity::{Address, PrivateKey};
use config::CONFIG;
use counterparty_client::CounterpartyClient;
use failure::Error;
use futures::{future, Future};
use guac_core::types::Counterparty;
use guac_core::UserApi;
use guac_core::{Crypto, Guac, Storage};
use std::env;
use std::sync::Arc;

#[macro_export]
macro_rules! try_future_box {
    ($expression:expr) => {
        match $expression {
            Err(err) => {
                return Box::new(future::err(err.into())) as Box<Future<Item = _, Error = Error>>;
            }
            Ok(value) => value,
        }
    };
}

pub fn init_guac(
    port: u16,
    contract_address: Address,
    own_address: Address,
    secret: PrivateKey,
    full_node_url: String,
) -> Guac {
    let guac = Guac {
        blockchain_client: Arc::new(Box::new(BlockchainClient::new(
            contract_address,
            own_address,
            secret,
            &full_node_url,
        ))),
        counterparty_client: Arc::new(Box::new(CounterpartyClient {})),
        storage: Arc::new(Box::new(Storage::new())),
        crypto: Arc::new(Box::new(Crypto {
            contract_address,
            own_address,
            secret,
        })),
    };

    counterparty_server::init_server(port, guac.clone());

    guac
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes() -> (Guac, Guac) {
        let contract_addr: Address = CONFIG.contract_address.parse().unwrap();

        let pk_1: PrivateKey = CONFIG.private_key_0.parse().unwrap();
        let addr_1 = pk_1.to_public_key().unwrap();

        let pk_2: PrivateKey = CONFIG.private_key_1.parse().unwrap();
        let addr_2 = pk_2.to_public_key().unwrap();

        let guac_1 = init_guac(
            8881,
            contract_addr,
            addr_1,
            pk_1,
            "http://127.0.0.1:8545".to_string(),
        );
        let guac_2 = init_guac(
            8882,
            contract_addr,
            addr_2,
            pk_2,
            "http://127.0.0.1:8545".to_string(),
        );

        (guac_1, guac_2)
    }

    #[test]
    fn test_register_counterparty() {
        let system = actix::System::new("test");

        let (guac_1, guac_2) = make_nodes();

        let storage_1 = guac_1.storage.clone();

        actix::spawn(
            guac_1
                .register_counterparty(guac_2.crypto.own_address, "example.com".to_string())
                .then(move |res| {
                    res.unwrap();

                    assert_eq!(
                        storage_1
                            .get_counterparty(guac_2.crypto.own_address)
                            .wait()
                            .unwrap()
                            .clone(),
                        Counterparty::New {
                            i_am_0: true,
                            url: "example.com".to_string()
                        }
                    );

                    System::current().stop();
                    Box::new(future::ok(()))
                }),
        );

        system.run();
    }

    #[test]
    fn test_fill_channel() {
        let system = actix::System::new("test");

        let (guac_1, guac_2) = make_nodes();

        let storage_1 = guac_1.storage.clone();

        actix::spawn(
            guac_1
                .register_counterparty(guac_2.crypto.own_address, "[::1]:8882".to_string())
                .and_then(move |_| {
                    guac_2
                        .register_counterparty(guac_1.crypto.own_address, "[::1]:8881".to_string())
                        .and_then(move |_| {
                            guac_1.fill_channel(guac_2.crypto.own_address, 64u64.into())
                        })
                })
                .then(|res| {
                    res.unwrap();
                    System::current().stop();
                    Box::new(future::ok(()))
                }),
        );

        system.run();
    }
}
