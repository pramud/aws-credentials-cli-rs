use derive_builder::Builder;

#[derive(Debug, Builder)]
pub struct RoleInfo {
    pub aws_partition: String,
    pub role_name: String,
    pub account_id: String,
    pub region: String,
    pub duration: i32,
}

impl RoleInfo {
    pub fn role_arn(&self) -> String {
        format!("arn:{}:iam::{}:role/{}", self.aws_partition, self.account_id, self.role_name)
    }
}
