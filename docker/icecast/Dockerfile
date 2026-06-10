FROM debian:stable-slim

LABEL org.opencontainers.image.authors="James Walker Crofts <20387783+sebastiandanconia@users.noreply.github.com>" \
    org.opencontainers.image.title="Icecast" \
    org.opencontainers.image.description="Icecast image updated for current Debian" \
    org.opencontainers.image.licenses="GPL-2.0" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.url="https://hub.docker.com/r/sebastiandanconia/icecast" \
    org.opencontainers.image.source="https://github.com/sebastiandanconia/selfhosted"

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get -qq -y update; \
  apt-get -qq -y full-upgrade; \
  apt-get -qq -y install icecast2 sudo cron-apt; \
  apt-get -y autoclean; \
  apt-get clean; \
  chown -R icecast2 /etc/icecast2; \
  chmod 777 /var/log/icecast2/access.log; \
  chmod 777 /var/log/icecast2/error.log ;\
  sed -i 's/ -d//' /etc/cron-apt/action.d/3-download

CMD ["/start.sh"]
EXPOSE 8000
VOLUME ["/config", "/var/log/icecast2", "/etc/icecast2"]

ADD ./start.sh /start.sh
