FROM debian:stable-slim

LABEL org.opencontainers.image.authors="James Walker Crofts <20387783+sebastiandanconia@users.noreply.github.com>" \
    org.opencontainers.image.title="Certbot (Continuous Run)" \
    org.opencontainers.image.description="Certbot image designed to stay running continuously" \
    org.opencontainers.image.licenses="GPL-2.0" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.url="https://hub.docker.com/r/sebastiandanconia/certbot" \
    org.opencontainers.image.source="https://github.com/sebastiandanconia/selfhosted"

RUN apt update \
&& apt install -y \
certbot \
cron \
curl

VOLUME /etc/letsencrypt
VOLUME /var/lib/letsencrypt

CMD ["cron", "-f", "-P"]

