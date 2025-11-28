FROM debian:stable-slim

RUN apt update \
&& apt install -y \
certbot \
cron \
curl

VOLUME /etc/letsencrypt
VOLUME /var/lib/letsencrypt

CMD ["cron", "-f", "-P"]

