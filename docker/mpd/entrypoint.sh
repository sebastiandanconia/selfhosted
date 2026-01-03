#!/bin/sh

# ------------------------------------------------------------
# MPD dynamic user/group setup script (PUID/PGID support)
# Works on Alpine Linux (tobi312/rpi-mpd:alpine and similar)
# ------------------------------------------------------------

set -e

# Default values (change if you prefer different defaults)
PUID=${PUID:-1000}
PGID=${PGID:-1000}

echo "Setting up MPD user/group -> UID=${PUID} GID=${PGID}"

# Create or modify group "mpd" with desired PGID
if ! getent group mpd >/dev/null; then
    # Group doesn't exist → create it
    addgroup -g "${PGID}" mpd
else
    # Group exists → change its GID if it's wrong
    OLD_GID=$(getent group mpd | cut -d: -f3)
    if [ "${OLD_GID}" != "${PGID}" ]; then
        groupmod -g "${PGID}" mpd
        echo "Changed group mpd GID ${OLD_GID} → ${PGID}"
    fi
fi

# Create or modify user "mpd" with desired PUID
if ! getent passwd mpd >/dev/null; then
    adduser -u "${PUID}" -G mpd -D -H -s /bin/false mpd
    echo "Created user mpd (UID ${PUID})"
else
    OLD_UID=$(getent passwd mpd | cut -d: -f3)
    if [ "${OLD_UID}" != "${PUID}" ]; then
        usermod -u "${PUID}" mpd
        echo "Changed user mpd UID ${OLD_UID} → ${PUID}"
    fi
fi

# Ensure primary group is correctly set to "mpd" (GID = PGID)
# This fixes the core issue where the base image has primary group "audio" (GID 18)
CURRENT_PGID=$(getent passwd mpd | cut -d: -f4)
if [ "${CURRENT_PGID}" != "${PGID}" ]; then
    usermod -g mpd mpd
    echo "Changed mpd primary GID from ${CURRENT_PGID} to ${PGID} (group mpd)"
fi

# Fix ownership of all MPD paths (safe even if some dirs don't exist)
chown -R "${PUID}:${PGID}" \
    /var/lib/mpd \
    /var/log/mpd \
    /run/mpd \
    /etc/mpd.conf \
    2>/dev/null || true

# IMPORTANT: Disable MPD's built-in privilege drop since gosu handles it
sed -i '/^[[:blank:]]*user[[:blank:]]\+.*$/ s/^/#/' /etc/mpd.conf 2>/dev/null || true
sed -i '/^[[:blank:]]*group[[:blank:]]\+.*$/ s/^/#/' /etc/mpd.conf 2>/dev/null || true


echo ""
echo "User/group setup complete."
echo "NOTE: For direct ALSA audio output (/dev/snd access):"
echo "  - Pass devices in docker run/compose:"
echo "      devices:"
echo "        - /dev/snd:/dev/snd"
echo "  - Add the host's audio GID as supplementary group:"
echo "      group_add:"
echo "        - \$(getent group audio | cut -d: -f3)  # OS dependent"
echo "Starting MPD as UID:${PUID} GID:${PGID} ..."

# Execute the original CMD as the configured user/group
if command -v gosu >/dev/null 2>&1; then
    exec gosu "${PUID}:${PGID}" "$@"
elif command -v su-exec >/dev/null 2>&1; then
    exec su-exec "${PUID}:${PGID}" "$@"
else
    # last resort: run as root (not recommended but works)
    echo "Warning: neither gosu nor su-exec found – running MPD as root!"
    exec "$@"
fi
