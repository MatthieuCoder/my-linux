#![deny(warnings)]

use std::{collections::BTreeMap, net::Ipv4Addr, sync::Arc};

use futures_util::{future::join_all, stream::SplitSink, SinkExt, StreamExt};
use pdu::EthernetPdu;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use tokio_tun::Tun;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mac_mapping = Arc::new(RwLock::new(BTreeMap::<
        [u8; 6],
        Arc<Mutex<SplitSink<WebSocket, Message>>>,
    >::new()));

    let tun = Tun::builder()
        .name("")
        .tap(true)
        .packet_info(false)
        .up()
        .mtu(1500)
        .address(Ipv4Addr::new(10, 10, 0, 1))
        .netmask(Ipv4Addr::new(255, 255, 0, 0))
        .try_build()
        .unwrap();

    let (mut reader, writer) = {
        let (read, write) = tokio::io::split(tun);

        (read, Arc::new(Mutex::new(write)))
    };

    {
        let mappings = mac_mapping.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1500];
            loop {
                let bytes = reader.read(&mut buf).await.unwrap();
                let bytes = &buf[0..bytes];

                match EthernetPdu::new(bytes) {
                    Ok(tramme) => {
                        if tramme.destination_address() == [0xff, 0xff, 0xff, 0xff, 0xff, 0xff] {
                            // send to all clients
                            println!("BROADCAST from tun.");
                            join_all(mappings.read().await.iter().map(|(_, writer)| async move {
                                let mut sink = writer.lock().await;
                                sink.send(Message::binary(&*bytes)).await.unwrap();
                            }))
                            .await;
                        } else if let Some(l) =
                            mappings.read().await.get(&tramme.destination_address())
                        {
                            println!("send from tun.");
                            l.lock().await.send(Message::binary(&*bytes)).await.unwrap();
                        }
                    }
                    Err(_) => {
                        println!("invalid data");
                    }
                }
            }
        });
    }
    let mac_mapping = mac_mapping.clone();
    let routes = warp::path("router")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let mappings = mac_mapping.clone();
            let writer = writer.clone();
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, mut rx) = websocket.split();
                let txx = Arc::new(Mutex::new(tx));
                async move {
                    let mut user_mac: Option<[u8; 6]> = None;
                    while let Some(Ok(message)) = rx.next().await {
                        let bytes = message.as_bytes();

                        match EthernetPdu::new(bytes) {
                            Ok(tramme) => {
                                let our_address = tramme.source_address();
                                if let Some(address) = user_mac {
                                    if our_address != address {
                                        continue;
                                    }
                                } else {
                                    if !mappings.read().await.contains_key(&our_address) {
                                        user_mac = Some(our_address);
                                        mappings
                                            .write()
                                            .await
                                            .insert(user_mac.unwrap(), txx.clone());
                                    } else {
                                        continue;
                                    }
                                }

                                // broadcast
                                if tramme.destination_address()
                                    == [0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
                                {
                                    // send to all clients
                                    join_all(mappings.read().await.iter().map(
                                        |(_, writer)| async move {
                                            let mut sink = writer.lock().await;
                                            sink.send(Message::binary(&*bytes)).await.unwrap();
                                        },
                                    ))
                                    .await;
                                    writer.lock().await.write(&bytes).await.unwrap();
                                } else if let Some(l) =
                                    mappings.read().await.get(&tramme.destination_address())
                                {
                                    l.lock().await.send(Message::binary(&*bytes)).await.unwrap();
                                } else {
                                    writer.lock().await.write(&bytes).await.unwrap();
                                }
                            }
                            Err(_) => {
                                println!("invalid data")
                            }
                        }
                    }
                    if let Some(mac) = user_mac {
                        mappings.write().await.remove(&mac);
                    }
                }
            })
        });
    
    let cert = std::env::var("SSL_CERT").unwrap();
    let key = std::env::var("SSL_KEY").unwrap();

    warp::serve(routes)
        .tls()
        .cert_path(cert)
        .key_path(key)
        .run(([0, 0, 0, 0], 443))
        .await;
}
