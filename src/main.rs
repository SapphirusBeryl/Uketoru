/*
 * Uketoru
 *
 * Copyright (C) 2024 Xavier Moffett <sapphirus@azorium.net>
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use axum::{middleware, routing::post, serve, Router};
use std::io::Error;
use tokio::net::TcpListener;

use crate::{config::CONFIG, relay::*};

mod config;
mod relay;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = Router::new()
        .route("/api", post(message).get(default))
        .fallback(default)
        .layer(middleware::from_fn(validate));
    let address = CONFIG.relay().listen_addr();
    let listen = TcpListener::bind(address).await?;

    println!("Uketoru listening on {address}");
    serve(listen, router).await
}
