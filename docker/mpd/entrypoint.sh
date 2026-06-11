#!/bin/sh
# ================================================================
# MPD Docker Entrypoint
# Supports dynamic PUID/PGID and supplementary groups (ADD_GIDS)
# ================================================================

set -e

# User/Group IDs for MPD (default values)
# Sanitize inputs to remove any surrounding quotes or extra whitespace.
# This makes the script robust whether values are quoted or unquoted in docker-compose.yml.
PUID=$(echo "${PUID:-1000}" | tr -d '"'\' | xargs)
PGID=$(echo "${PGID:-1000}" | tr -d '"'\' | xargs)
# ADD_GIDS: optional comma-separated list of additional GIDs for mpd user
ADD_GIDS=$(echo "${ADD_GIDS:-}" | tr -d '"'\' | xargs)

echo "MPD Docker - Setting up mpd user: UID=${PUID} GID=${PGID} ADD_GIDS=${ADD_GIDS:-none}"

# Ensure mpd group exists with correct GID
if ! getent group mpd >/dev/null; then
    groupadd -g "${PGID}" mpd
else
    OLD_GID=$(getent group mpd | cut -d: -f3)
    if [ "${OLD_GID}" != "${PGID}" ]; then
        groupmod -o -g "${PGID}" mpd
        echo "Updated mpd group GID ${OLD_GID} → ${PGID}"
    fi
fi

# Ensure mpd user exists with correct UID
if ! getent passwd mpd >/dev/null; then
    useradd -u "${PUID}" -g mpd -o -M -s /usr/sbin/nologin mpd
    echo "Created mpd user (UID ${PUID})"
else
    OLD_UID=$(getent passwd mpd | cut -d: -f3)
    if [ "${OLD_UID}" != "${PUID}" ]; then
        usermod -o -u "${PUID}" mpd
        echo "Updated mpd user UID ${OLD_UID} → ${PUID}"
    fi
fi

# Ensure primary group is mpd (important on some base images)
CURRENT_GID=$(id -g mpd)
if [ "${CURRENT_GID}" != "${PGID}" ]; then
    usermod -g mpd mpd
    echo "Updated mpd primary group to GID ${PGID}"
fi

# Handle additional supplementary groups (e.g. audio group on host)
if [ -n "${ADD_GIDS}" ]; then
    echo "Adding supplementary groups to mpd user: ${ADD_GIDS}"
    OLDIFS="$IFS"
    IFS=','
    for gid_part in ${ADD_GIDS}; do
        gid=$(echo "$gid_part" | xargs)
        if [ -n "$gid" ] && echo "$gid" | grep -q '^[0-9]\+$'; then
            if ! getent group "$gid" >/dev/null 2>&1; then
                groupadd -g "$gid" "supp${gid}" 2>/dev/null || true
            fi
            group_name=$(getent group "$gid" | cut -d: -f1)
            if [ -n "$group_name" ]; then
                usermod -aG "$group_name" mpd && \
                    echo "Added mpd to group ${group_name} (GID=${gid})" || \
                    echo "Warning: Failed to add mpd to group ${group_name}"
            fi
        else
            echo "Warning: Invalid GID '${gid_part}' in ADD_GIDS"
        fi
    done
    IFS="$OLDIFS"
fi

# Fix permissions on key directories
chown -R "${PUID}:${PGID}" \
    /var/lib/mpd \
    /var/log/mpd \
    /run/mpd \
    /etc/mpd.conf 2>/dev/null || true

# Disable MPD's internal privilege dropping (we use gosu instead)
sed -i 's/^\s*user\s\+.*/#&/' /etc/mpd.conf 2>/dev/null || true
sed -i 's/^\s*group\s\+.*/#&/' /etc/mpd.conf 2>/dev/null || true

echo "User/group setup completed. Starting MPD as UID:${PUID} GID:${PGID}..."

# Run MPD as the target user using gosu
exec gosu "${PUID}:${PGID}" "$@"
