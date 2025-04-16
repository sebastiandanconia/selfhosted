# ipps:// Proxy Service
For better security, flexibility, and compatibility, I run a proxy service that offers secure access to my HP printer from an untrusted network. I give specific users on my untrusted guest network a secret "Internet Printing Protocol, Secure" (IPPS) URL, for example
```
ipps://ipp.example.net/ipp/41QxT4Xtt9YFu46i4Gi9v
```
which points to a reverse proxy server on my trusted network. Because the connection between the user's computer and the proxy uses TLS, both the URL and the data they're printing are encrypted. The proxy in turn makes an unencrypted connection to the printer.

## Implementation Change
After trying implementations using Nginx, Apache2, and CUPS, I'm now using Squid as the proxy.

## Docker Container

You can generate secret authentication keys using:
```
dd if=/dev/random bs=1 count=16 | base58
```

### Building the Docker Container
```
docker build -t ipp-proxy .
```

### Create Docker Volumes
```
docker volume create ipp-config
docker create ipp-ssl
```

### Deploying the Container
```
docker run -d --name ipp-proxy \
--restart unless-stopped \
-p 631:631 \
-v ipp-config:/etc/squid \
-v ipp-ssl:/etc/squid/ssl \
--tmpfs /var/spool/squid \
ubuntu/squid
```

Copy your `squid.conf` to `/etc/squid/` in the container. You can find example(s) of `squid.conf` in the `EXAMPLES` folder. `squid.conf` should be readable by Squid within the container.

### Docker-Compose
As an alternative to running Docker directly, you can set up your `docker-compose.yaml` to something like this:
```
---
services:
  ipp-proxy:
    container_name: ipp-proxy
    image: ubuntu/squid
    environment:
      - TZ=Etc/UTC
    volumes:
    - ipp-config:/etc/squid/
    - ipp-ssl:/etc/squid/ssl/
    tmpfs:
      - /var/spool/squid
    ports:
      - "631:631"
```

## Certbot Configuration
Use Certbot to obtain a TLS certificate for the computer which will host the proxy service.

As part of configuring `certbot`, create a deployment script:
```
/etc/letsencrypt/renewal-hooks/deploy/ipp-example-deploy.sh
```
This script should contain something like:
```
#!/bin/bash

DOMAIN="ipp.example.net"

LIVE_CERT_DIR=/etc/letsencrypt/live/ipp
SERVER_CERT_DIR=/var/lib/docker/volumes/ipp-ssl/_data

# If the domain this script is managing is in the list
if [[ "${RENEWED_DOMAINS}" == *"${DOMAIN}"* ]]; then
  cp --dereference "${LIVE_CERT_DIR}"/fullchain.pem "${SERVER_CERT_DIR}"/ipp.crt
  cp --dereference "${LIVE_CERT_DIR}"/privkey.pem "${SERVER_CERT_DIR}"/ipp.key
  # Set ownership/permissions, e.g.:
  # chown -R root:root "${SERVER_CERT_DIR}"

  docker restart ipp-proxy
fi
```

### Testing & Debugging
```
docker exec -it ipp-proxy bash
```

#### Without TLS (ipp://)
```
ipptool -tv ipp://ipp.example.net:631/ipp/41QxT4Xtt9YFu46i4Gi9v get-printer-attributes.test
```
#### With TLS (ipps://)
Test your proxy service can receive connections and is using a valid certificate:
```
openssl s_client -connect ipp.example.net:631

ipptool -tv ipps://ipp.example.net:631/printer/41QxT4Xtt9YFu46i4Gi9v get-printer-attributes.test
```

(In a direct IPP connection to the printer, pathnames in the URL such as `/printer` or `/ipp` appear to have the same effect as no pathname.)
