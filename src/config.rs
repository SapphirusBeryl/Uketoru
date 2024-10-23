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

use lettre::transport::smtp::authentication::Credentials;
use serde::Deserialize;
use std::{fs::File, io::Read, sync::LazyLock};

pub static CONFIG: LazyLock<Config> = LazyLock::new(load);

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug, Clone, Default)]
pub enum TransportType {
    #[default]
    TLS,
    StartTLS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Smtp {
    #[serde(default = "TransportType::default")]
    transport: TransportType,
    address: String,
    to: Option<String>,
    name: Option<String>,
    username: Option<String>,
    #[serde(default)]
    password: String,
    server: String,
    port: Option<u16>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Relay {
    listen: Option<String>,
    token: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    relay: Relay,
    smtp: Smtp,
}

impl Config {
    pub fn client(&self) -> &Smtp {
        &self.smtp
    }

    pub fn relay(&self) -> &Relay {
        &self.relay
    }
}

impl Relay {
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn listen_addr(&self) -> &str {
        self.listen.as_deref().unwrap_or("0.0.0.0:3000")
    }
}

impl Smtp {
    pub fn transport(&self) -> &TransportType {
        &self.transport
    }

    pub fn credentials(&self) -> Credentials {
        match &self.username {
            Some(username) => Credentials::new(username.to_owned(), self.password.to_owned()),
            None => Credentials::new(self.address.to_owned(), self.password.to_owned()),
        }
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(25)
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn to(&self) -> &str {
        self.to.as_deref().unwrap_or(&self.address)
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("Uketoru")
    }
}

fn load() -> Config {
    let mut string = String::new();
    let mut config = File::open("config.toml").expect("Opening config.toml");

    config.read_to_string(&mut string).expect("Reading config.toml");
    toml::from_str::<Config>(&string).expect("Invalid config.toml")
}
