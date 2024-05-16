# ipps:// Reverse Proxy
For better security, flexibility, and compatibility, I run a reverse proxy that can access my HP printer. I give specific users on my untrusted guest network a secret "Internet Printing Protocol, Secure" (IPPS) URL, for example
```
ipps://argentina.example.net:631/41QxT4Xtt9YFu46i4Gi9v/ipp
```
which points to a reverse proxy server on my trusted network. Because the connection between the user's computer and the reverse proxy uses TLS, both the URL and the data they're printing are encrypted. The reverse proxy in turn makes an unencrypted IPP connection to the printer.

This example assumes you're using `$HOSTNAME` as the name of the certificate. Also, `argentina.example.net` and `argentina.lan` are the same machine.

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
### Changing the certificate name and password
Substitute your hostname (certificate name) in `default.conf`.

You can generate secret authentication keys using
```
dd if=/dev/random bs=1 count=20 | base58
```

Now, in `default.conf`, edit the `location /` statement to reflect the secret authentication key you generated. You may copy and paste `location` blocks as many times as you wish for different users and groups. If you leave it as is, all users will be able to send print jobs to your printer.

#### Example `default.conf`
```
server {
	listen 631 ssl;
	listen  [::]:631 ssl;
	server_name     $hostname;

	ssl_certificate /etc/nginx/ssl/live/argentina/fullchain.pem;
	ssl_certificate_key /etc/nginx/ssl/live/argentina/privkey.pem;

	# Substitute your own secret authentication key here
	location /41QxT4Xtt9YFu46i4Gi9v/ {
		proxy_pass http://printer.lan:631/;

		# HTTP 1.1 is REQUIRED by the printer
		proxy_http_version 1.1;
		...
	}

}
```

### Building the Docker Container
```docker build -t nginx-ipp-proxy .```

### Deploying the Container
```
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
Test your reverse proxy can receive connections and is using a valid certificate:
```
openssl s_client -connect $HOSTNAME.lan:631
```

Test you can talk to your printer through your reverse proxy:
```
ipptool -tv ipps://$HOSTNAME.lan:631/ipp get-printer-attributes.test
```
```
ipptool -tv ipps://$HOSTNAME.lan:631/$SECRET_AUTH_KEY/ipp get-printer-attributes.test
```
Pathnames in the URL such as `/printer` or `/ipp` appear to have the same effect as no pathname.
