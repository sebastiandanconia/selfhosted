# Certbot Docker Image

The official Certbot Docker images are generally intended to be invoked as needed for certificate renewal. By contrast, this image is intended to stay running continuously, calling `certbot` and related scripts at least daily via `cron` jobs. This is useful for running within a NAS or similar environment.

You will normally want to share `certbot-config` (or whatever you're calling the volume/directory where Certbot will store certificates on your system) with the containers/apps that use the certificates.

### Docker-Compose
Example `docker-compose.yaml`:
```yaml
---
services:
  certbot:
    container_name: certbot
    image: sebastiandanconia/certbot
    restart: always
    user: "0:0" # Run container processes as root
    volumes:
      - certbot-config:/etc/letsencrypt
      - certbot-state:/var/lib/letsencrypt
      - certbot-log:/var/log/letsencrypt
```