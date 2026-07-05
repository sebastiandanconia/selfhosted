#!/bin/sh
# ============================================================
# tftp-hpa container entrypoint
#
#   * Forwards tftpd syslog to stdout  -> `docker logs`
#   * Seeds an empty TFTP root with shipped PXE bootloaders
#   * Drops privileges via tftpd-hpa's native -u (root binds
#     UDP/69, then transfers run as the configured user)
# ============================================================
set -e

SEED_DIR=/usr/share/tftpboot-assets
TFTPBOOT_DIR="${TFTPBOOT_DIR:-/tftpboot}"
PUID=$(echo "${PUID:-0}" | tr -d '"'\' | xargs)
PGID=$(echo "${PGID:-0}" | tr -d '"'\' | xargs)

# --- One-shot "seed" subcommand -------------------------------------------
# Populate a host directory once (e.g. before mounting it read-only).
# Existing files are kept.
if [ "${1:-}" = "seed" ]; then
    echo "[entrypoint] Seeding ${TFTPBOOT_DIR} from ${SEED_DIR} (existing files kept)"
    mkdir -p "${TFTPBOOT_DIR}"
    cp -a --update=none "${SEED_DIR}"/. "${TFTPBOOT_DIR}"/
    echo "[entrypoint] Done."
    exit 0
fi

# --- syslog -> stdout ------------------------------------------------------
# tftpd-hpa logs to syslog; without a syslog daemon nothing is captured.
# busybox syslogd -O - writes every syslog line to stdout (docker logs).
if [ "${DISABLE_SYSLOG_FORWARD:-0}" != "1" ]; then
    rm -f /dev/log 2>/dev/null || true
    busybox syslogd -n -O - &
fi

# --- Seed empty, writable TFTP root ----------------------------------------
if [ "${SEED_ON_EMPTY:-1}" = "1" ]; then
    if [ -z "$(ls -A "${TFTPBOOT_DIR}" 2>/dev/null)" ]; then
        if touch "${TFTPBOOT_DIR}/.seed-probe" 2>/dev/null; then
            rm -f "${TFTPBOOT_DIR}/.seed-probe"
            echo "[entrypoint] ${TFTPBOOT_DIR} is empty; seeding PXE bootloaders"
            cp -a "${SEED_DIR}"/. "${TFTPBOOT_DIR}"/
        else
            echo "[entrypoint] ${TFTPBOOT_DIR} is empty and read-only; not seeding." >&2
            echo "[entrypoint] Populate it yourself, or run: docker run --rm -v <host>:/tftpboot <image> seed" >&2
        fi
    fi
fi

# --- Privilege model -------------------------------------------------------
# tftpd must bind UDP/69 as root, then drops to -u for transfers, so we use
# tftpd-hpa's native -u rather than gosu (gosu would drop before the bind).
# Default (PUID=1000) runs transfers as an unprivileged 'tftp' user;
# PUID=0 runs as root; any other PUID creates 'tftp' with that UID.
if [ "${PUID}" = "0" ]; then
    TFTP_USER=root
else
    if ! getent group tftp >/dev/null; then
        groupadd -o -g "${PGID}" tftp
    fi
    if ! getent passwd tftp >/dev/null; then
        useradd -o -u "${PUID}" -g tftp -M -s /usr/sbin/nologin tftp
    fi
    TFTP_USER=tftp
fi

echo "[entrypoint] Serving ${TFTPBOOT_DIR} as user ${TFTP_USER} (uid=${PUID} gid=${PGID})"

# "$@" lets you append tftpd flags, e.g. command: ["--blocksize", "1428"].
# The served directory is always last.
exec /usr/sbin/in.tftpd -L --verbose -u "${TFTP_USER}" --secure "$@" "${TFTPBOOT_DIR}"
