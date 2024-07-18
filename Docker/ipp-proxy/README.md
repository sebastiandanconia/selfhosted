# ipps:// Proxy Service
For better security, flexibility, and compatibility, I run a proxy service that offers secure access to my HP printer from an untrusted network. I give specific users on my untrusted guest network a secret "Internet Printing Protocol, Secure" (IPPS) URL, for example
```
ipps://ipp.example.net/printers/41QxT4Xtt9YFu46i4Gi9v
```
which points to a CUPS server on my trusted network. Because the connection between the user's computer and the proxy uses TLS, both the URL and the data they're printing are encrypted. The CUPS server in turn makes an unencrypted connection to the printer.

## Implementation Change
An earlier version of this service used a regular reverse proxy in Nginx or Apache2, but this only worked for trivially small print jobs (if at all), due to the special HTTP dialect IPP uses (HTTP `Continue`, `KeepAlive`, etc.).

The connection between the proxy service and and printer doesn't necessarily have to be IPP; I found doing so was causing a PostScript error, and that this was fixed by configuring it instead as an AppSocket or LPD connection. Alternatively, [https://wiki.debian.org/CUPSDebugging](https://wiki.debian.org/CUPSDebugging) offers hints for digging into print engine/interpreter bugs.

## Docker Container

You can generate secret authentication keys (which you will use as printer names in CUPS) using:
```
dd if=/dev/random bs=1 count=20 | base58
```

### Building the Docker Container
```
docker build -t ipp-proxy-cups .
```

### Create Docker Volumes
```
docker volume create ipp-config
docker create ipp-ssl
```

### Deploying the Container
```
docker run -d --name ipp-proxy \
--hostname ipp \
--restart unless-stopped \
-p 631:631 \
-v ipp-config:/etc/cups \
-v ipp-ssl:/etc/cups/ssl \
--tmpfs /run/cups \
--tmpfs /var/spool/cups \
--tmpfs /var/cache/cups \
ipp-proxy-cups
```

After starting the container for the first time to populate the otherwise-empty `ipp-config` Docker volume, copy `cupsd.conf` to `/etc/cups` in the container. You can find examples of `cupsd.conf` in the `EXAMPLES` folder. `cupsd.conf` should be owned by `root:lp` within the container.

### Setting Root Passphrase (Optional)
```
docker exec -it ipp-proxy bash
root@ipp:/# passwd
New password:
Retype new password:
passwd: password updated successfully
root@ipp:/# exit
exit
```

### Configuring Print Queues
You'll need to tell CUPS about your printers. The easiest way to do this is to set temporarily relaxed security settings (e.g. `WebInterface Yes` in `cupsd.conf`) and use the web interface at (for example) `https://ipp.example.net:631/`. You might, in this example, name a printer/queue `41QxT4Xtt9YFu46i4Gi9v`. Alternatively, you can use `lpadmin`.

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
CUPS_CERT_DIR=/var/lib/docker/volumes/ipp-ssl/_data

# If the domain this script is managing is in the list
if [[ "${RENEWED_DOMAINS}" == *"${DOMAIN}"* ]]; then
  # NOTE: Certificate and key MUST be of the form $HOSTNAME.crt & $HOSTNAME.key,
  # respectively, as seen by cupsd (i.e. inside the container).
  cp --dereference "${LIVE_CERT_DIR}"/fullchain.pem "${CUPS_CERT_DIR}"/ipp.crt
  cp --dereference "${LIVE_CERT_DIR}"/privkey.pem "${CUPS_CERT_DIR}"/ipp.key
  # NOOP: chown -R root:root "${CUPS_CERT_DIR}"

  docker restart ipp-proxy
fi
```

### Testing & Debugging
```
docker exec -it ipp-proxy bash
```

#### Without TLS (ipp://)
```
ipptool -tv ipp://ipp.example.net:631/printer/41QxT4Xtt9YFu46i4Gi9v get-printer-attributes.test
```
#### With TLS (ipps://)
Test your proxy service can receive connections and is using a valid certificate:
```
openssl s_client -connect ipp.example.net:631

ipptool -tv ipps://ipp.example.net:631/printer/41QxT4Xtt9YFu46i4Gi9v get-printer-attributes.test
```

(In a direct IPP connection to the printer, pathnames in the URL such as `/printer` or `/ipp` appear to have the same effect as no pathname.)
