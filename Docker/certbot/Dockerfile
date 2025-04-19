FROM ubuntu

RUN apt update \
&& apt install -y \
certbot \
curl

VOLUME /etc/letsencrypt
VOLUME /var/lib/letsencrypt

CMD ["/bin/bash"]

