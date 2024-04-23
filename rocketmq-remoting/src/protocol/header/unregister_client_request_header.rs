/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    protocol::command_custom_header::{CommandCustomHeader, FromMap},
    rpc::rpc_request_header::RpcRequestHeader,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UnregisterClientRequestHeader {
    #[serde(rename = "clientID")]
    pub client_id: String,
    pub producer_group: Option<String>,
    pub consumer_group: Option<String>,
    #[serde(flatten)]
    pub rpc_request_header: RpcRequestHeader,
}

impl UnregisterClientRequestHeader {
    pub const CLIENT_ID: &'static str = "clientID";
    pub const PRODUCER_GROUP: &'static str = "producerGroup";
    pub const CONSUMER_GROUP: &'static str = "consumerGroup";
}

impl FromMap for UnregisterClientRequestHeader {
    type Target = Self;

    fn from(map: &HashMap<String, String>) -> Option<Self::Target> {
        Some(UnregisterClientRequestHeader {
            client_id: map
                .get(UnregisterClientRequestHeader::CLIENT_ID)
                .map(|s| s.to_string())
                .unwrap_or_default(),
            producer_group: map
                .get(UnregisterClientRequestHeader::PRODUCER_GROUP)
                .cloned(),
            consumer_group: map
                .get(UnregisterClientRequestHeader::CONSUMER_GROUP)
                .cloned(),
            rpc_request_header: <RpcRequestHeader as FromMap>::from(map).unwrap(),
        })
    }
}

impl CommandCustomHeader for UnregisterClientRequestHeader {
    fn to_map(&self) -> Option<HashMap<String, String>> {
        let mut map = self.rpc_request_header.to_map();
        map.as_mut()
            .unwrap()
            .insert(Self::CLIENT_ID.to_string(), self.client_id.clone());
        if let Some(ref producer_group) = self.producer_group {
            map.as_mut()
                .unwrap()
                .insert(Self::PRODUCER_GROUP.to_string(), producer_group.clone());
        }
        if let Some(ref consumer_group) = self.consumer_group {
            map.as_mut()
                .unwrap()
                .insert(Self::CONSUMER_GROUP.to_string(), consumer_group.clone());
        }
        map
    }
}
