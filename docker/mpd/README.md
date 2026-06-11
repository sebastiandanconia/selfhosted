# MPD with UID/GID Agility

A fork of [Tob1as/docker-mpd](https://github.com/Tob1as/docker-mpd) by Tobias Hargesheimer that adds support for user-configurable UID/GID via environment variables.

## Why This Fork?

The upstream image runs MPD as the `mpd` user created inside the container, whose UID/GID is determined by the Alpine package manager (typically UID 100, GID 101). This creates permission issues when bind-mounting host directories:

- **Host directories must match the container's internal IDs** — not the user's preferred UID/GID
- **The upstream workaround is `chmod 777`** — functional but insecure
- **No `PUID`/`PGID` mechanism exists upstream** — a common pattern in self-hosted images (e.g., LinuxServer.io)

This fork adds an entrypoint script that adjusts the `mpd` user's UID/GID at container startup, allowing seamless integration with host filesystems.

## Usage

If you want the machine on which you're running Docker to be able to output audio locally (for example to directly-connected speakers), you may need to set `ADD_GIDS` to the Group ID of the `audio` group on your host computer.

### Docker Compose

```yaml
services:
  mpd:
    image: sebastiandanconia/mpd
    restart: unless-stopped
    environment:
      - TZ=${TZ:-UTC}
      - PUID="1000"  # Match your host user
      - PGID="1000"  # Match your host group
      # Optional comma-separated list of groups of which mpd should be a member
      # - ADD_GIDS="18,29"
      # For ALSA audio, set ADD_GIDS=$(getent group audio | cut -d: -f3)
      # - ADD_GIDS="${HOST_AUDIO_GID}"
    cap_add:
      - SYS_NICE
    volumes:
      - mpd-data:/var/lib/mpd:rw,delegated
      - mpd-music:/var/lib/mpd/music:ro
      - mpd-playlists:/var/lib/mpd/playlists:rw
      - path/to/mpd.conf:/etc/mpd.conf
    ports:
      - 6600:6600  # MPD client
      - 8000:8000  # HTTP stream
    devices:
      - /dev/snd:/dev/snd
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PUID`   | `1000`  | User ID for the `mpd` process |
| `PGID`   | `1000`  | Group ID for the `mpd` process |
|  ADD_GIDS|  ""     | Additional Group IDs for the `mpd` process |

## How It Works

The `entrypoint.sh` script:

1. Modifies the `mpd` user/group to match `PUID`/`PGID`
2. Fixes ownership of MPD directories (`/var/lib/mpd`, etc.)
3. Disables MPD's built-in privilege dropping (handled by `gosu` instead)
4. Drops to the configured UID/GID via `gosu` before exec'ing MPD

## Credits

- **Original image**: [Tob1as/docker-mpd](https://github.com/Tob1as/docker-mpd)
- **Original author**: Tobias Hargesheimer <docker@ison.ws>
- **License**: GPL-2.0 (inherited from upstream)

## See Also

- [MPD Documentation](https://www.musicpd.org/)
- [MPD Clients](https://www.musicpd.org/clients/)
