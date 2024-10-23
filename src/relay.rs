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

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};
use serde::Deserialize;

use crate::config::{TransportType, CONFIG};

#[derive(Deserialize)]
pub struct RelayMessage {
    from: String,
    text: String,
}

pub async fn default() -> (StatusCode, Body) {
    (StatusCode::NOT_FOUND, Body::from("Not Found"))
}

pub async fn validate(req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = match CONFIG.relay().token() {
        Some(token) => token,
        None => return Ok(next.run(req).await),
    };
    let auth = match req.headers().get(header::AUTHORIZATION) {
        Some(header) => header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?,
        None => Err(StatusCode::UNAUTHORIZED)?,
    };

    if !auth.starts_with("Bearer") || auth.len() < 7 {
        Err(StatusCode::BAD_REQUEST)?
    } else if &auth[7 ..] != token {
        Err(StatusCode::UNAUTHORIZED)?
    }

    Ok(next.run(req).await)
}

pub async fn message(Json(msg): Json<RelayMessage>) -> (StatusCode, Body) {
    let client = CONFIG.client();
    let message = Message::builder()
        .from(format!("{} <{}>", msg.from, client.address()).parse().expect("Invalid address"))
        .to(format!("{} <{}>", client.name(), client.to()).parse().expect("Invalid address"))
        .subject(format!("Message received from {} via Uketoru", msg.from))
        .header(ContentType::TEXT_PLAIN)
        .body(msg.text)
        .expect("Message builder");
    let transport = match CONFIG.client().transport() {
        TransportType::TLS => SmtpTransport::relay(CONFIG.client().server()),
        TransportType::StartTLS => SmtpTransport::starttls_relay(CONFIG.client().server()),
    };
    let transport = match transport {
        Ok(transport) => transport,
        Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Internal Error: {err}"))),
    };

    match transport
        .port(CONFIG.client().port())
        .credentials(CONFIG.client().credentials())
        .build()
        .send(&message)
    {
        Ok(_) => (StatusCode::OK, Body::empty()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Internal Error: {err}"))),
    }
}
