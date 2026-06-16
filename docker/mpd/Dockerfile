FROM tobi312/rpi-mpd:debian

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
