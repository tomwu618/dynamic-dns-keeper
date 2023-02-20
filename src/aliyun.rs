use std::net::IpAddr;
use aliyun_openapi_core_rust_sdk::RPClient;
use crate::function::Function;

pub struct AliYun {
    key_id: String,
    key_secret: String,
    record_id: String,
    rr: String,
    record_type: String,
    ttl: String,
    line: String,
}

impl AliYun {
    pub(crate) fn new(key_id: String, key_secret: String, record_id: String, rr: String, record_type: String, ttl: String, line: String) -> Self {
        AliYun {
            key_id,
            key_secret,
            record_id,
            rr,
            record_type,
            ttl,
            line,
        }
    }
}

impl Function for AliYun {
    fn update(&self, record: IpAddr) -> bool {
        let api_url = "https://alidns.aliyuncs.com/";
        let apt_version = "2015-01-09";

        let aliyun_openapi_client = RPClient::new(
            String::from(self.key_id.clone()),
            String::from(self.key_secret.clone()),
            String::from(api_url),
            String::from(apt_version),
        );

        let response = aliyun_openapi_client
            .get("UpdateDomainRecord")
            .query(&[("DomainName", "next53.xyz"),
                ("RecordId", self.record_id.as_str()),
                ("RR", self.rr.as_str()),
                ("Type", self.record_type.as_str()),
                ("TTL", self.ttl.as_str()),
                ("Line", self.line.as_str()),
                ("Value", record.to_string().as_str()),
            ])
            .send();

        if response.is_err() {
            println!("DescribeRegions error: {}", response.err().unwrap());
        } else {
            println!("DescribeRegions response: {}", response.ok().unwrap());
        }

        return true;
    }
}
