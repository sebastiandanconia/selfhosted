FROM tobi312/rpi-mpd:debian

LABEL org.opencontainers.image.authors="James Walker Crofts <20387783+sebastiandanconia@users.noreply.github.com>" \
    org.opencontainers.image.title="MPD with UID/GID Agility" \
    org.opencontainers.image.description="Music Player Daemon with PUID/PGID support for flexible volume permissions" \
    org.opencontainers.image.licenses="GPL-2.0" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.url="https://hub.docker.com/r/sebastiandanconia/certbot" \
    org.opencontainers.image.source="https://github.com/sebastiandanconia/selfhosted"

# Install gosu for clean privilege dropping + ca-certificates for HTTPS if needed later
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        gosu \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*

# Copy entrypoint
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Entry point runs as root to handle user/group changes, then drops privileges with gosu
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["/usr/bin/mpd", "--no-daemon", "--stdout", "/etc/mpd.conf"]
