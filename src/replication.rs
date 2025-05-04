use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

#[derive(Debug, PartialEq)]
pub enum Role {
    Master,
    Replica,
}

#[derive(Debug, PartialEq)]
pub struct ReplicationInfo {
    pub role: Role,
    pub master_replid: String,
    pub master_repl_offset: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MasterConfig {
    pub host: String,
    pub port: u16,
}

pub struct ReplicationConfig {
    pub master: Option<MasterConfig>,
    pub master_replid: String,
    pub master_repl_offset: usize,
}

impl ReplicationConfig {
    pub fn new_master() -> Self {
        ReplicationConfig {
            master: None,
            master_replid: Alphanumeric.sample_string(&mut rng(), 40),
            master_repl_offset: 0,
        }
    }

    pub fn new_replica(master_host: String, master_port: u16) -> Self {
        ReplicationConfig {
            master: Some(MasterConfig {
                host: master_host,
                port: master_port,
            }),
            master_replid: Alphanumeric.sample_string(&mut rng(), 40),
            master_repl_offset: 0,
        }
    }

    pub fn info(&self) -> ReplicationInfo {
        ReplicationInfo {
            role: match &self.master {
                Some(_) => Role::Replica,
                None => Role::Master,
            },
            master_replid: self.master_replid.clone(),
            master_repl_offset: self.master_repl_offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_config_master() {
        let config = ReplicationConfig::new_master();

        assert_eq!(config.master, None);
        assert_eq!(config.master_replid.len(), 40);
        assert_eq!(config.master_repl_offset, 0);
    }

    #[test]
    fn test_replication_config_replica() {
        let config = ReplicationConfig::new_replica(String::from("other"), 1234);

        assert_eq!(
            config.master,
            Some(MasterConfig {
                host: String::from("other"),
                port: 1234
            })
        );
        assert_eq!(config.master_replid.len(), 40);
        assert_eq!(config.master_repl_offset, 0);
    }

    #[test]
    fn test_replication_info_master() {
        let config = ReplicationConfig::new_master();

        assert_eq!(config.info().role, Role::Master);
        assert_eq!(config.info().master_replid.len(), 40);
        assert_eq!(config.info().master_repl_offset, 0);
    }

    #[test]
    fn test_replication_info_replica() {
        let config = ReplicationConfig::new_replica(String::from("other"), 1234);

        assert_eq!(config.info().role, Role::Replica);
        assert_eq!(config.info().master_replid.len(), 40);
        assert_eq!(config.info().master_repl_offset, 0);
    }
}
