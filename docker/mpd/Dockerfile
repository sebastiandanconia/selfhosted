FROM tobi312/rpi-mpd:alpine

LABEL org.opencontainers.image.authors="James Walker Crofts <20387783+sebastiandanconia@users.noreply.github.com>" \
    org.opencontainers.image.title="MPD with UID/GID Agility" \
    org.opencontainers.image.description="Music Player Daemon with PUID/PGID support for flexible volume permissions" \
    org.opencontainers.image.licenses="GPL-2.0" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${VCS_REF}" \
    org.opencontainers.image.url="https://hub.docker.com/r/sebastiandanconia/certbot" \
    org.opencontainers.image.source="https://github.com/sebastiandanconia/selfhosted"

# Install gosu for clean step-down from root (lighter than tini + su-exec)
# su-exec is already in Alpine, but gosu is preferred by most
RUN apk add --no-cache gosu shadow

# Copy our entrypoint
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Keep the original non-daemon behaviour
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["/usr/bin/mpd", "--no-daemon", "--stdout", "/etc/mpd.conf"]
