# tftp-hpa Docker Image (PXE-enabled)

Multi-architecture Docker image running [tftp-hpa](https://git.kernel.org/pub/scm/network/tftp/tftp-hpa.git/) (the H. Peter Anvin TFTP daemon) with a curated set of PXE bootloaders for BIOS and UEFI x86_64 network booting, including iPXE for HTTP-boot chains.

It stays current by pulling every bootloader from Ubuntu apt packages at build time.

## Design

You own the whole TFTP root. Bind-mount a host directory at `/tftpboot` and put whatever you want in it. The image ships PXE bootloaders and safe default menus as **seed assets** under `/usr/share/tftpboot-assets`; on first start, if `/tftpboot` is empty and writable, the entrypoint copies them in so a fresh volume is immediately usable. If you pre-populate your mount, nothing is touched.

## Bootloaders included

Seed assets copied into an empty `/tftpboot`:

| Path | Source package | Purpose |
|------|----------------|---------|
| `pxelinux.0`, `lpxelinux.0` | `pxelinux` | BIOS PXE loader (`lpxelinux.0` also supports HTTP/FTP) |
| `ldlinux.c32`, `menu.c32`, `vesamenu.c32`, `libutil.c32`, `libcom32.c32`, `reboot.c32`, `poweroff.c32` | `syslinux-common` | Syslinux modules for menus |
| `memdisk` | `syslinux-common` | Boot floppy/ISO images from RAM |
| `grub/grubx64.efi` | `grub-efi-amd64-bin` (built via `grub-mkimage`, TFTP prefix `(tftp)/grub`) | UEFI x86_64 net boot; fetches `grub/grub.cfg` |
| `ipxe/ipxe.efi`, `ipxe/undionly.kpxe`, `ipxe/ipxe.lkrn` | `ipxe` | iPXE (UEFI + BIOS), HTTP-boot capable |
| `pxelinux.cfg/default`, `grub/grub.cfg`, `ipxe/boot.ipxe` | this repo | Safe defaults that boot nothing |

## Quick start

```bash
docker run -d --name tftp \
  -p 69:69/udp \
  -v /srv/tftpboot:/tftpboot \
  sebastiandanconia/tftp-hpa
```

Point your DHCP server at this host as the next-server (option 66) with a bootfile (option 67) such as `pxelinux.0` (BIOS) or `grub/grubx64.efi` (UEFI x86_64).

### docker compose

```yaml
services:
  tftp:
    image: sebastiandanconia/tftp-hpa
    restart: unless-stopped
    environment:
      # Match your host UID/GID so the unprivileged tftp user can read your files.
      # Defaults: 1000. Set to 0 to run transfers as root.
      - PUID=1000
      - PGID=1000
    volumes:
      - /srv/tftpboot:/tftpboot
    ports:
      - 69:69/udp
```

## Configuration

### Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PUID` | `1000` | UID for tftpd transfers. `0` = run as root. Non-zero creates a `tftp` user with this UID; tftpd becomes this user after binding UDP/69. Match your host UID so the unprivileged daemon can read your mounted files. |
| `PGID` | `1000` | GID for the `tftp` user (ignored when `PUID=0`). |
| `TFTPBOOT_DIR` | `/tftpboot` | Directory served over TFTP. |
| `SEED_ON_EMPTY` | `1` | Copy seed assets into an empty, writable `TFTPBOOT_DIR` on start. |
| `DISABLE_SYSLOG_FORWARD` | `0` | Set `1` to skip the in-container syslog→stdout forwarder. |

### Privilege model

By default tftpd runs its transfers as an unprivileged `tftp` user (UID/GID 1000), not as root. tftpd must still bind UDP/69 (<1024), so the container starts as root, binds the port, and then drops privileges using tftpd-hpa's native `-u` flag. Set `PUID=0` to run transfers as root, or set `PUID`/`PGID` to your host UID/GID so the unprivileged daemon can read your bind-mounted files. The seed assets are world-readable (0444) so the unprivileged user can always serve them. This differs from the `gosu` pattern used in the mpd image because gosu would drop privileges before the port-69 bind and the daemon could not listen.

### Logging

tftpd-hpa logs via syslog. The entrypoint starts `busybox syslogd -O -` so all syslog output appears in `docker logs <container>`. tftpd is started with `--verbose`, which logs every transfer.

### One-shot seeding

To populate a host directory once (for example before mounting it read-only):

```bash
docker run --rm -v /srv/tftpboot:/tftpboot sebastiandanconia/tftp-hpa seed
```

Existing files are not overwritten.

## Boot configuration examples

The shipped defaults boot nothing (a menu offering only Reboot / Power off, with no timeout). The examples below show two netbooted hosts, each with its own kernel command line. Place your kernel/initrd files under `/tftpboot/images/`.

### BIOS — pxelinux per-host

pxelinux searches `pxelinux.cfg/` in order: `01-<lowercase mac with dashes>` → `<hex-ip>` → `default`.

`pxelinux.cfg/01-11-22-33-44-55-01` (host A):
```
ui vesamenu.c32
menu title host-a
prompt 0
timeout 50
default linux
label linux
  menu label host-a Linux
  kernel images/vmlinuz-a
  append initrd=images/initrd-a.img root=/dev/nfs rw console=tty0 console=ttyS0,115200 host=a special-flag-for-a
```

`pxelinux.cfg/01-11-22-33-44-55-02` (host B) is the same shape with `images/vmlinuz-b`, `images/initrd-b.img`, and `host=b special-flag-for-b`.

### UEFI x86_64 — GRUB per-host

The shipped `grubx64.efi` fetches `grub/grub.cfg` over TFTP. Branch on the client MAC, which GRUB exposes as `net_default_mac`:

`grub/grub.cfg`:
```
set timeout=50
if [ "${net_default_mac}" = "11:22:33:44:55:01" ]; then
  source /grub/grub.cfg-host-a
elif [ "${net_default_mac}" = "11:22:33:44:55:02" ]; then
  source /grub/grub.cfg-host-b
else
  menuentry "Reboot" { reboot }
  menuentry "Power off" { halt }
fi
```

`grub/grub.cfg-host-a`:
```
menuentry "host-a Linux" {
  linux /images/vmlinuz-a root=/dev/nfs rw console=tty0 console=ttyS0,115200 host=a special-flag-for-a
  initrd /images/initrd-a.img
}
```

### HTTP boot — iPXE

For HTTP-based booting, TFTP serves only the iPXE binary; iPXE then chains a script fetched over HTTP. Minimal `ipxe/boot.ipxe`:
```
#!ipxe
dhcp
chain http://boot.example.com/${mac:hex}.ipxe
```

Set the DHCP bootfile to `ipxe/ipxe.efi` (UEFI) or `ipxe/undionly.kpxe` (BIOS), then have iPXE chain the script above (via an embedded script or DHCP option 175).

## Known limitations

- **Secure Boot not supported.** The shipped `grubx64.efi` is unsigned (built from `grub-efi-amd64-bin`). For Secure Boot, use `grub-efi-amd64-signed` + `shim-signed`.
- **No UEFI arm64.** Only x86_64 UEFI (`grubx64.efi`) and x86 BIOS (`pxelinux.0`) bootloaders are seeded. arm64 UEFI (`grubaa64.efi` / `shimaa64.efi`) is a possible future enhancement.
- **No DHCP / proxyDHCP.** This image is pure TFTP. Per-host PXE segmentation is handled at the DHCP layer.

## Supported architectures

The image is built for `linux/amd64` and `linux/arm64`. The tftpd-hpa daemon is native to each platform; the seeded PXE bootloaders are x86/x86_64-only and boot x86 clients regardless of which architecture the server runs on.

## Building

```bash
docker build -t sebastiandanconia/tftp-hpa .
```

Multi-arch (requires buildx):
```bash
docker buildx build --platform linux/amd64,linux/arm64 --tag sebastiandanconia/tftp-hpa:latest .
```

The bootloader-builder stage is pinned to the build host platform (`$BUILDPLATFORM`) so the x86/x86_64 bootloaders are always produced natively; only the final stage is built per target platform.

## CI/CD

GitHub Actions builds and pushes multi-arch images on pushes to main/master and on tags. Required secrets: `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN`.

## License

GPL-2.0-or-later for this image's own files. Bundled bootloaders retain their upstream licenses (syslinux GPL-2, GRUB GPL-3, iPXE GPL-2).
