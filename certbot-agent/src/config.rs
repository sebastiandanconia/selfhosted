/* Configuration settings. Code designed for general reuse would put these
   in a configuration file or similar. In this case, defining compile-time
   constants is simpler since this is a niche custom script, and because
   we don't want lots of CLI options in a cron job, thereby moving complexity
   and risk to command line option propagation and parsing. */

// Local config:
pub const LIVE_CERT_DIR: &str = "/etc/letsencrypt/live/phaethon";
pub const CACHE_FILE: &str = "/var/cache/certbot-agent.yaml";

// Server config:
pub const SERVER_SSH_ACCOUNT: &str = "root@phaethon.example.net";
pub const SSH_PORT: &str = "2222";
pub const SERVER_CERT_DIR: &str = "/var/lib/docker/volumes/minio-certs/_data";

