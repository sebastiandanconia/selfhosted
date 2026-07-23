# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Stage 1: Build/collect PXE bootloaders for x86 clients.
#
# These loaders are x86/x86_64 code that runs on the PXE client, not on the
# TFTP server. Build them on amd64 (native gcc). When the image itself is
# built for arm64, Buildx runs this stage under QEMU (CI sets that up).
# ---------------------------------------------------------------------------
FROM --platform=linux/amd64 ubuntu:24.04 AS bootloader-builder

ENV DEBIAN_FRONTEND=noninteractive

# Pin iPXE releases for reproducible TLS/crypto capability. v2.0.0 ships a
# full TLS 1.2 client (ECDHE, ECDSA, AES-GCM, named curves / supported_groups).
ARG IPXE_REF=v2.0.0

RUN apt-get update && apt-get install -y --no-install-recommends \
        pxelinux \
        syslinux-common \
        grub-common \
        grub-efi-amd64-bin \
        ca-certificates \
        git \
        make \
        gcc \
        g++ \
        binutils \
        perl \
        liblzma-dev \
        libc6-dev \
        gcc-multilib \
    && rm -rf /var/lib/apt/lists/*

# iPXE: build from upstream with improved HTTPS support (see ipxe/general.h).
# Ubuntu's packaged iPXE is an older/minimal compile whose TLS ClientHello
# lacks supported_groups/ec_point_formats and a modern cipher list,
# so high-security HTTPS boot servers reject it.
COPY ipxe/general.h /tmp/ipxe-local/general.h
RUN set -eux; \
    git clone --depth 1 --branch "${IPXE_REF}" https://github.com/ipxe/ipxe.git /tmp/ipxe; \
    cp /tmp/ipxe-local/general.h /tmp/ipxe/src/config/local/general.h; \
    make -C /tmp/ipxe/src -j"$(nproc)" \
        bin-x86_64-efi/ipxe.efi \
        bin/undionly.kpxe \
        bin/ipxe.lkrn

RUN set -eux; \
    mkdir -p /assets/pxelinux.cfg /assets/grub /assets/ipxe; \
    # BIOS: pxelinux loaders, memdisk, and the menu/loop modules they need.
    cp /usr/lib/PXELINUX/pxelinux.0  /assets/pxelinux.0; \
    cp /usr/lib/PXELINUX/lpxelinux.0 /assets/lpxelinux.0; \
    cp /usr/lib/syslinux/memdisk    /assets/memdisk; \
    for m in ldlinux menu vesamenu libutil libcom32 reboot poweroff; do \
        cp /usr/lib/syslinux/modules/bios/${m}.c32 /assets/${m}.c32; \
    done; \
    # UEFI x86_64: a self-contained grubx64.efi that fetches /grub/grub.cfg
    # over TFTP from the DHCP next-server. (Unsigned; no Secure Boot.)
    grub-mkimage -O x86_64-efi -o /assets/grub/grubx64.efi \
        -p '(tftp)/grub' \
        efinet tftp http net linux normal configfile ls cat echo reboot halt \
        search search_fs_uuid search_fs_file search_label test \
        part_gpt part_msdos fat ext2 btrfs xfs loopback chain \
        videoinfo font gfxterm gettext terminal serial; \
    # iPXE: HTTPS-capable network bootloaders for UEFI and BIOS.
    cp /tmp/ipxe/src/bin-x86_64-efi/ipxe.efi /assets/ipxe/ipxe.efi; \
    cp /tmp/ipxe/src/bin/undionly.kpxe       /assets/ipxe/undionly.kpxe; \
    cp /tmp/ipxe/src/bin/ipxe.lkrn           /assets/ipxe/ipxe.lkrn

# Safe default configs that boot nothing (no kernel, no local disk boot).
COPY defaults/pxelinux.cfg.default /assets/pxelinux.cfg/default
COPY defaults/grub.cfg             /assets/grub/grub.cfg
COPY defaults/boot.ipxe            /assets/ipxe/boot.ipxe

# Read-only assets: files 0444, dirs 0555 (traversable).
RUN find /assets -type f -exec chmod 0444 {} + && \
    find /assets -type d -exec chmod 0555 {} +

# ---------------------------------------------------------------------------
# Stage 2: Runtime. Pure TFTP server + a tiny syslog forwarder.
# ---------------------------------------------------------------------------
FROM ubuntu:24.04

ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

LABEL org.opencontainers.image.authors="James Walker Crofts <20387783+sebastiandanconia@users.noreply.github.com>" \
      org.opencontainers.image.title="tftp-hpa (PXE-enabled)" \
      org.opencontainers.image.description="tftp-hpa TFTP server with BIOS pxelinux and UEFI x86_64 GRUB/iPXE bootloaders" \
      org.opencontainers.image.licenses="GPL-2.0-or-later" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.url="https://hub.docker.com/r/sebastiandanconia/tftp-hpa" \
      org.opencontainers.image.source="https://github.com/sebastiandanconia/selfhosted"

ENV DEBIAN_FRONTEND=noninteractive \
    PUID=1000 \
    PGID=1000 \
    TFTPBOOT_DIR=/tftpboot \
    SEED_ON_EMPTY=1 \
    DISABLE_SYSLOG_FORWARD=0

RUN apt-get update && apt-get install -y --no-install-recommends \
        tftpd-hpa \
        busybox \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /tftpboot

COPY --from=bootloader-builder /assets /usr/share/tftpboot-assets
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

VOLUME /tftpboot
EXPOSE 69/udp

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD []
