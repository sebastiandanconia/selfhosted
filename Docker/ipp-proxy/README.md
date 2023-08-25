# ipps:// Reverse Proxy
For better access control over my HP printer, I run a reverse proxy.

## Certbot Configuration
As part of configuring `certbot`, create a deployment script:
```
/etc/letsencrypt/renewal-hooks/deploy/$HOSTNAME-deploy.sh
```
This script should contain, at a minimum, something like:
```
#!/bin/bash

docker exec ipp-proxy nginx -s reload
```

## Docker Container
### Building the Docker Container
```docker build -t nginx-ipp-proxy .```

### Deploying the Container
```
HOSTNAME=$(hostname)

docker run --name ipp-proxy -d -p 631:631 --restart unless-stopped \
-v /etc/letsencrypt/live/$HOSTNAME:/etc/nginx/ssl/live/$HOSTNAME:ro \
-v /etc/letsencrypt/archive/$HOSTNAME:/etc/nginx/ssl/archive/$HOSTNAME:ro \
nginx-ipp-proxy:latest
```
### Testing & Debugging
```docker exec -it ipp-proxy bash```

#### Without TLS (ipp://)
```
ipptool -tv ipp://$HOSTNAME.lan:631/ get-printer-attributes.test
```
#### With TLS (ipps://)
```
ipptool -tv ipps://$HOSTNAME.lan:631/ipp get-printer-attributes.test
```
Pathnames in the URL such as `/printer` or `/ipp` appear to have the same effect as no pathname.
