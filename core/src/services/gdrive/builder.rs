// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use log::debug;

use super::backend::GdriveBackend;
use crate::raw::{normalize_root, HttpClient};
use crate::Scheme;
use crate::*;

/// [GoogleDrive](https://drive.google.com/) backend support.
///
/// # Capabilities
///
/// This service can be used to:
///
/// - [x] read
/// - [x] write
/// - [x] delete
/// - [ ] copy
/// - [ ] create
/// - [ ] list
/// - [ ] rename
///
/// # Notes
///
///
/// # Configuration
///
/// - `access_token`: set the access_token for google drive api
/// - `root`: Set the work directory for backend
///
/// You can refer to [`GoogleDriveBuilder`]'s docs for more information
///
/// # Example
///
/// ## Via Builder
///
/// ```no_run
/// use anyhow::Result;
/// use opendal::services::Gdrive;
/// use opendal::Operator;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // create backend builder
///     let mut builder = Gdrive::default();
///
///     builder.access_token("xxx").root("/path/to/root");
///
///     let op: Operator = Operator::new(builder)?.finish();
///
///     let write = op.write("abc.txt", "who are you").await?;
///     let read = op.read("abc.txt").await?;
///     let s = String::from_utf8(read).unwrap();
///     println!("{}", s);
///     let delete = op.delete("abc.txt").await?;
///     Ok(())
/// }
/// ```
#[derive(Default)]
pub struct GdriveBuilder {
    access_token: Option<String>,
    root: Option<String>,
    http_client: Option<HttpClient>,
}

impl Debug for GdriveBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend").field("root", &self.root).finish()
    }
}

impl GdriveBuilder {
    /// default: no access token, which leads to failure
    pub fn access_token(&mut self, access_token: &str) -> &mut Self {
        self.access_token = Some(access_token.to_string());
        self
    }

    /// Set root path of GoogleDrive folder.
    pub fn root(&mut self, root: &str) -> &mut Self {
        self.root = Some(root.to_string());
        self
    }

    /// Specify the http client that used by this service.
    ///
    /// # Notes
    ///
    /// This API is part of OpenDAL's Raw API. `HttpClient` could be changed
    /// during minor updates.
    pub fn http_client(&mut self, http_client: HttpClient) -> &mut Self {
        self.http_client = Some(http_client);
        self
    }
}

impl Builder for GdriveBuilder {
    const SCHEME: Scheme = Scheme::Gdrive;

    type Accessor = GdriveBackend;

    fn from_map(map: HashMap<String, String>) -> Self {
        let mut builder = Self::default();

        map.get("root").map(|v| builder.root(v));
        map.get("access_token").map(|v| builder.access_token(v));

        builder
    }

    fn build(&mut self) -> Result<Self::Accessor> {
        let root = normalize_root(&self.root.take().unwrap_or_default());
        debug!("backend use root {}", root);

        let client = if let Some(client) = self.http_client.take() {
            client
        } else {
            HttpClient::new().map_err(|err| {
                err.with_operation("Builder::build")
                    .with_context("service", Scheme::Gdrive)
            })?
        };

        match self.access_token.clone() {
            Some(access_token) => Ok(GdriveBackend::new(root, access_token, client)),
            None => Err(Error::new(ErrorKind::ConfigInvalid, "access_token not set")),
        }
    }
}
