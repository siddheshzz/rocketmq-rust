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
use cheetah_string::CheetahString;
use rocketmq_macros::RequestHeaderCodec;
use serde::Deserialize;
use serde::Serialize;

use crate::rpc::topic_request_header::TopicRequestHeader;

#[derive(Clone, Debug, Serialize, Deserialize, RequestHeaderCodec)]
#[serde(rename_all = "camelCase")]
pub struct QuerySubscriptionByConsumerRequestHeader {
    #[required]
    pub group: CheetahString,

    #[required]
    pub topic: CheetahString,

    #[serde(flatten)]
    pub topic_request_header: Option<TopicRequestHeader>,
}

#[cfg(test)]
mod tests {
    use cheetah_string::CheetahString;

    use super::*;

    #[test]
    fn query_subscription_by_consumer_request_header_serializes_correctly() {
        let header = QuerySubscriptionByConsumerRequestHeader {
            group: CheetahString::from_static_str("test_group"),
            topic: CheetahString::from_static_str("test_topic"),
            topic_request_header: None,
        };
        let serialized = serde_json::to_string(&header).unwrap();
        let expected = r#"{"group":"test_group","topic":"test_topic"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn query_subscription_by_consumer_request_header_deserializes_correctly() {
        let data = r#"{"group":"test_group","topic":"test_topic"}"#;
        let header: QuerySubscriptionByConsumerRequestHeader = serde_json::from_str(data).unwrap();
        assert_eq!(header.group, CheetahString::from_static_str("test_group"));
        assert_eq!(header.topic, CheetahString::from_static_str("test_topic"));
        assert!(!header.topic_request_header.is_none());
    }

    #[test]
    fn query_subscription_by_consumer_request_header_handles_missing_optional_fields() {
        let data = r#"{"group":"test_group","topic":"test_topic"}"#;
        let header: QuerySubscriptionByConsumerRequestHeader = serde_json::from_str(data).unwrap();
        assert_eq!(header.group, CheetahString::from_static_str("test_group"));
        assert_eq!(header.topic, CheetahString::from_static_str("test_topic"));
        assert!(!header.topic_request_header.is_none());
    }

    #[test]
    fn query_subscription_by_consumer_request_header_handles_invalid_data() {
        let data = r#"{"group":12345,"topic":"test_topic"}"#;
        let result: Result<QuerySubscriptionByConsumerRequestHeader, _> =
            serde_json::from_str(data);
        assert!(result.is_err());
    }
}
