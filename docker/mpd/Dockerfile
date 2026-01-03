FROM tobi312/rpi-mpd:alpine

# Install gosu for clean step-down from root (lighter than tini + su-exec)
# su-exec is already in Alpine, but gosu is preferred by most
RUN apk add --no-cache gosu shadow

# Copy our entrypoint
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Keep the original non-daemon behaviour
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
USER mpd
CMD ["/usr/bin/mpd", "--no-daemon", "--stdout", "/etc/mpd.conf"]
