use crate::util::cat;

pub struct Cloudflare {
    auth: String,
    zone_id: String,
}

const HOST: &str = "api.cloudflare.com";

impl<A: crate::addr::Address> super::HandleRecord<A> for Cloudflare {
    type RecordId = String;

    fn get_record_id(&self, record_name: &str) -> Option<String> {
        let path = cat(&[
            "/client/v4/zones/", &self.zone_id,
            "/dns_records?name=", record_name,
            "&type=", A::RECORD_TYPE,
        ]);
        let resp = crate::req::get(HOST, &path, &[("Authorization", &self.auth)])?;

        if resp.status != 200 {
            return None;
        }

        let start = resp.body.find(r#""id":""#)? + 6;
        let end = start + resp.body[start..].find('"')?;
        Some(resp.body[start..end].to_string())
    }

    fn update_dns_record(&self, record_id: String, record_name: &str, ip: A) -> bool {
        let path = cat(&["/client/v4/zones/", &self.zone_id, "/dns_records/", &record_id]);
        let mut ip_buf = [0u8; 64];
        let ip_str = ip.to_str(&mut ip_buf);
        let body = cat(&[
            r#"{"type":""#, A::RECORD_TYPE,
            r#"","name":""#, record_name,
            r#"","content":""#, ip_str,
            r#"","ttl":120,"proxied":false}"#,
        ]);

        crate::req::put(
            HOST,
            &path,
            &[
                ("Authorization", &self.auth),
                ("Content-Type", "application/json"),
            ],
            &body,
        )
        .is_some_and(|r| r.status == 200)
    }

    fn create_dns_record(&self, record_name: &str, ip: A) -> bool {
        let path = cat(&["/client/v4/zones/", &self.zone_id, "/dns_records"]);
        let mut ip_buf = [0u8; 64];
        let ip_str = ip.to_str(&mut ip_buf);
        let body = cat(&[
            r#"{"type":""#, A::RECORD_TYPE,
            r#"","name":""#, record_name,
            r#"","content":""#, ip_str,
            r#"","ttl":120,"proxied":false}"#,
        ]);

        crate::req::post(
            HOST,
            &path,
            &[
                ("Authorization", &self.auth),
                ("Content-Type", "application/json"),
            ],
            &body,
        )
        .is_some_and(|r| r.status == 200)
    }
}

impl super::DnsProvider for Cloudflare {
    #[inline(always)]
    fn new() -> Self {
        Self {
            auth: cat(&["Bearer ", &crate::util::env("CLOUDFLARE_API_TOKEN")]),
            zone_id: crate::util::env("CLOUDFLARE_ZONE_ID"),
        }
    }
}
