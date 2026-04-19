pub struct Cloudflare {
    auth: String,
    zone_id: String,
}

const HOST: &str = "api.cloudflare.com";

impl<A: crate::Address> super::HandleRecord<A> for Cloudflare {
    type RecordId = String;

    fn get_record_id(&self, record_name: &str) -> Option<String> {
        let path = format!(
            "/client/v4/zones/{}/dns_records?name={record_name}&type={}",
            self.zone_id,
            A::RECORD_TYPE
        );
        let resp = crate::req::get(HOST, &path, &[("Authorization", &self.auth)])?;

        if resp.status != 200 {
            return None;
        }

        let start = resp.body.find(r#""id":""#)? + 6;
        let end = start + resp.body[start..].find('"')?;
        Some(resp.body[start..end].to_string())
    }

    fn update_dns_record(&self, record_id: String, record_name: &str, ip: A) -> bool {
        let path = format!("/client/v4/zones/{}/dns_records/{record_id}", self.zone_id);
        let body = format!(
            r#"{{"type":"{}","name":"{record_name}","content":"{ip}","ttl":120,"proxied":false}}"#,
            A::RECORD_TYPE
        );

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
        let path = format!("/client/v4/zones/{}/dns_records", self.zone_id);
        let body = format!(
            r#"{{"type":"{}","name":"{record_name}","content":"{ip}","ttl":120,"proxied":false}}"#,
            A::RECORD_TYPE
        );

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
            auth: format!("Bearer {}", crate::env("CLOUDFLARE_API_TOKEN")),
            zone_id: crate::env("CLOUDFLARE_ZONE_ID"),
        }
    }
}
