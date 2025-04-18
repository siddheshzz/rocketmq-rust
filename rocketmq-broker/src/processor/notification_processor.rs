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

use cheetah_string::CheetahString;
use rocketmq_remoting::runtime::connection_handler_context::ConnectionHandlerContext;
use tracing::error;
use tracing::warn;

#[derive(Default)]
pub struct NotificationProcessor {}

impl NotificationProcessor {
    pub fn start(&mut self) {
        warn!("NotificationProcessor started unimplemented, need to implement it");
    }

    fn process_request(
        &self,
        _ctx: ConnectionHandlerContext,
        _request: rocketmq_remoting::protocol::remoting_command::RemotingCommand,
    ) -> rocketmq_remoting::protocol::remoting_command::RemotingCommand {
        todo!()
    }

    pub fn shutdown(&mut self) {
        warn!("NotificationProcessor shutdown unimplemented, need to implement it");
    }

    #[allow(unused_variables)]
    pub fn notify_message_arriving(&self, topic: &CheetahString, queue_id: i32) {
        error!("notify_message_arriving unimplemented, need to implement it");
    }

    #[allow(unused_variables)]
    pub fn notify_message_arriving_full(
        &self,
        topic: CheetahString,
        queue_id: i32,
        tags_code: Option<i64>,
        msg_store_time: i64,
        filter_bit_map: Option<Vec<u8>>,
        properties: Option<&HashMap<CheetahString, CheetahString>>,
    ) {
        error!("notify_message_arriving_full unimplemented, need to implement it");
    }
}
