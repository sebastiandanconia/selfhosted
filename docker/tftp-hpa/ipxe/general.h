/*
 * Local iPXE configuration overrides for this image.
 *
 * Upstream disables HTTPS (and some related features) on BIOS builds for
 * historical size reasons. Re-enable a modern TLS client so both UEFI and
 * BIOS iPXE can chain to high-security HTTPS endpoints that require ECDHE,
 * ECDSA, supported_groups/ec_point_formats, and a current cipher suite list.
 *
 * Included last via config/local/general.h, after platform #undefs.
 */

/* HTTPS download protocol (also on BIOS; already default on EFI). */
#define DOWNLOAD_PROTO_HTTPS

/* Certificate management commands (debug TLS trust / load extra CAs). */
#define CERT_CMD

/* Image digest commands (verify downloaded kernels/initrds). */
#define DIGEST_CMD
